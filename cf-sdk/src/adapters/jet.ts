import Big from "big.js";

import {
  Cluster,
  Keypair,
  PublicKey,
  Signer,
  SystemProgram,
  SYSVAR_CLOCK_PUBKEY,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import { Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import * as anchor from "@project-serum/anchor";

import {
  Amount,
  DEX_ID,
  JetClient,
  JetMarket,
  JetReserve,
  JetUser,
  JET_ID,
  JET_MARKET_ADDRESS,
  JET_MARKET_ADDRESS_DEVNET,
  ReserveConfig,
} from "@jet-lab/jet-engine";

import { Asset } from "./asset";

export interface JetAccounts {
  program: PublicKey;
  reserve: PublicKey;
  market: PublicKey;
  marketAuthority: PublicKey;
  feeNoteVault: PublicKey;
  depositNoteMint: PublicKey;
  liquiditySupply: PublicKey;
  pythPrice: PublicKey;
}

export class JetReserveAsset extends Asset {
  private constructor(
    public provider: anchor.Provider,
    public accounts: JetAccounts,
    public market: JetMarket,
    public reserve: JetReserve
  ) {
    super();
  }

  static async load(
    provider: anchor.Provider,
    cluster: Cluster,
    reserveMint: PublicKey
  ): Promise<JetReserveAsset> {
    let client: JetClient;
    let market: JetMarket;
    if (cluster == "devnet") {
      client = await JetClient.connect(provider, true);
      market = await JetMarket.load(client, JET_MARKET_ADDRESS_DEVNET);
    } else if (cluster == "mainnet-beta") {
      client = await JetClient.connect(provider, false);
      market = await JetMarket.load(client, JET_MARKET_ADDRESS);
    } else {
      throw new Error("Cluster ${cluster} not supported");
    }
    const reserves = await JetReserve.loadMultiple(client, market);
    const reserve = reserves.find((res) => res.data.tokenMint.equals(reserveMint));

    const accounts: JetAccounts = {
      program: JET_ID,
      reserve: reserve.data.address,
      market: market.address,
      marketAuthority: market.marketAuthority,
      feeNoteVault: reserve.data.feeNoteVault,
      depositNoteMint: reserve.data.depositNoteMint,
      liquiditySupply: reserve.data.vault,
      pythPrice: reserve.data.pythOraclePrice,
    };
    return new JetReserveAsset(provider, accounts, market, reserve);
  }

  /**
   * Creates a market, reserves, and adds initial liquidity
   *
   * TODO Split into create market adding reserves to it
   *
   * @param provider
   * @param owner
   * @param marketQuoteTokenMint
   * @param reserveToken
   * @param pythPrice
   * @param pythProduct
   * @param ownerReserveTokenAccount
   * @param initialReserveAmount
   * @returns
   */
  static async initialize(
    provider: anchor.Provider,
    wallet: anchor.Wallet,
    owner: Signer,
    marketQuoteTokenMint: PublicKey,
    reserveToken: Token,
    pythPrice: PublicKey,
    pythProduct: PublicKey,
    ownerReserveTokenAccount: PublicKey,
    initialReserveAmount: number
  ): Promise<JetReserveAsset> {
    const client = await JetClient.connect(provider, true);
    const market = await createLendingMarket(client, wallet, marketQuoteTokenMint);

    const accounts = await createReserve(
      wallet,
      client,
      market.address,
      marketQuoteTokenMint,
      reserveToken,
      TOKEN_PROGRAM_ID, // dummy dex market addr
      pythPrice,
      pythProduct
    );

    const reserve = await JetReserve.load(client, accounts.reserve);
    const jetUser = await JetUser.load(client, market, [reserve], owner.publicKey);
    const depositTx = await jetUser.makeDepositTx(
      reserve,
      ownerReserveTokenAccount,
      Amount.tokens(initialReserveAmount)
    );
    await provider.send(depositTx, [owner]);

    return new JetReserveAsset(provider, accounts, market, reserve);
  }

  async getLpTokenAccountValue(address: PublicKey): Promise<Big> {
    await this.market.refresh();

    const reserveInfo = this.market.reserves[this.reserve.data.index];
    const exchangeRate = new Big(reserveInfo.depositNoteExchangeRate.toString()).div(
      new Big(1e15)
    );

    const lpToken = new Token(
      this.provider.connection,
      this.reserve.data.depositNoteMint,
      TOKEN_PROGRAM_ID,
      Keypair.generate() // dummy signer since we aren't making any txs
    );

    const lpTokenAccountInfo = await lpToken.getAccountInfo(address);
    const lpTokenAmount = new Big(lpTokenAccountInfo.amount.toString());

    return exchangeRate.mul(lpTokenAmount);
  }

  async getApy(): Promise<Big> {
    await this.reserve.refresh();
    return new Big(this.reserve.data.depositApy);
  }
}

async function createLendingMarket(
  client: JetClient,
  wallet: anchor.Wallet,
  quoteCurrencyMint: PublicKey
): Promise<JetMarket> {
  const account = Keypair.generate();

  await client.program.rpc.initMarket(wallet.publicKey, "USD", quoteCurrencyMint, {
    accounts: {
      market: account.publicKey,
    },
    signers: [account],
    instructions: [await client.program.account.market.createInstruction(account)],
  });

  return JetMarket.load(client, account.publicKey);
}

async function createReserve(
  wallet: anchor.Wallet,
  client: JetClient,
  market: PublicKey,
  quoteTokenMint: PublicKey,
  tokenMint: Token,
  dexMarket: PublicKey,
  pythPrice: PublicKey,
  pythProduct: PublicKey
): Promise<JetAccounts> {
  const reserve = Keypair.generate();
  const [depositNoteMint, depositNoteMintBump] = await findProgramAddress(
    client.program.programId,
    ["deposits", reserve, tokenMint]
  );
  const [loanNoteMint, loanNoteMintBump] = await findProgramAddress(
    client.program.programId,
    ["loans", reserve, tokenMint]
  );
  const [vault, vaultBump] = await findProgramAddress(client.program.programId, [
    "vault",
    reserve,
  ]);
  const [feeNoteVault, feeNoteVaultBump] = await findProgramAddress(
    client.program.programId,
    ["fee-vault", reserve]
  );
  const [dexSwapTokens, dexSwapTokensBump] = await findProgramAddress(
    client.program.programId,
    ["dex-swap-tokens", reserve]
  );
  const [dexOpenOrders, dexOpenOrdersBump] = await findProgramAddress(
    client.program.programId,
    ["dex-open-orders", reserve]
  );
  const [marketAuthority] = await findProgramAddress(client.program.programId, [
    market,
  ]);

  const reserveAccounts = {
    accounts: {
      reserve,
      vault,
      feeNoteVault,
      dexOpenOrders,
      dexSwapTokens,
      tokenMint,

      dexMarket,
      pythPrice,
      pythProduct,

      depositNoteMint,
      loanNoteMint,
    },

    bump: {
      vault: vaultBump,
      feeNoteVault: feeNoteVaultBump,
      dexOpenOrders: dexOpenOrdersBump,
      dexSwapTokens: dexSwapTokensBump,
      depositNoteMint: depositNoteMintBump,
      loanNoteMint: loanNoteMintBump,
    },
  };

  const reserveConfig: ReserveConfig = {
    utilizationRate1: 8500,
    utilizationRate2: 9500,
    borrowRate0: 50,
    borrowRate1: 600,
    borrowRate2: 4000,
    borrowRate3: 1600,
    minCollateralRatio: 12500,
    liquidationPremium: 300,
    manageFeeRate: 0,
    manageFeeCollectionThreshold: new anchor.BN(10),
    loanOriginationFee: 0,
    liquidationSlippage: 300,
    liquidationDexTradeMax: new anchor.BN(100),
    reserved0: 0,
    reserved1: Array(24).fill(0),
  };

  await client.program.rpc.initReserve(reserveAccounts.bump, reserveConfig, {
    accounts: toPublicKeys({
      market,
      marketAuthority,
      owner: wallet.publicKey,

      oracleProduct: reserveAccounts.accounts.pythProduct,
      oraclePrice: reserveAccounts.accounts.pythPrice,

      quoteTokenMint,

      tokenProgram: TOKEN_PROGRAM_ID,
      dexProgram: DEX_ID,
      clock: SYSVAR_CLOCK_PUBKEY,
      rent: SYSVAR_RENT_PUBKEY,
      systemProgram: SystemProgram.programId,

      ...reserveAccounts.accounts,
    }),
    signers: [reserveAccounts.accounts.reserve, wallet.payer],
    instructions: [
      await client.program.account.reserve.createInstruction(
        reserveAccounts.accounts.reserve
      ),
    ],
  });

  return {
    program: JET_ID,
    reserve: reserve.publicKey,
    market: market,
    marketAuthority: marketAuthority,
    feeNoteVault: feeNoteVault,
    depositNoteMint: depositNoteMint,
    liquiditySupply: vault,
    pythPrice: pythPrice,
  };
}

/**
 * Find a program derived address
 * @param programId The program the address is being derived for
 * @param seeds The seeds to find the address
 * @returns The address found and the bump seed required
 */
async function findProgramAddress(
  programId: PublicKey,
  seeds: (HasPublicKey | ToBytes | Uint8Array | string)[]
): Promise<[PublicKey, number]> {
  const seed_bytes = seeds.map((s) => {
    if (typeof s == "string") {
      return Buffer.from(s);
    } else if ("publicKey" in s) {
      return s.publicKey.toBytes();
    } else if ("toBytes" in s) {
      return s.toBytes();
    } else {
      return s;
    }
  });
  return await PublicKey.findProgramAddress(seed_bytes, programId);
}

interface ToBytes {
  toBytes(): Uint8Array;
}

interface HasPublicKey {
  publicKey: PublicKey;
}

/**
 * Convert some object of fields with address-like values,
 * such that the values are converted to their `PublicKey` form.
 * @param obj The object to convert
 */
function toPublicKeys(
  obj: Record<string, string | PublicKey | HasPublicKey | any>
): any {
  const newObj = {};

  for (const key in obj) {
    const value = obj[key];

    if (typeof value == "string") {
      newObj[key] = new PublicKey(value);
    } else if (typeof value == "object" && "publicKey" in value) {
      newObj[key] = value.publicKey;
    } else {
      newObj[key] = value;
    }
  }

  return newObj;
}
