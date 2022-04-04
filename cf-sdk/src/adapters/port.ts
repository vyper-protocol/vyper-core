import Big from "big.js";

import {
  Cluster,
  Keypair,
  PublicKey,
  SystemProgram,
  Transaction,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, Token } from "@solana/spl-token";
import { ENV } from "@solana/spl-token-registry";
import * as anchor from "@project-serum/anchor";

import {
  AssetConfig,
  AssetDepositConfig,
  AssetDisplayConfig,
  AssetPrice,
  AssetPriceConfig,
  DEFAULT_PORT_LENDING_MARKET,
  Environment,
  initLendingMarketInstruction,
  initReserveInstruction,
  MintId,
  Port,
  PORT_STAKING,
  ReserveConfigProto,
  ReserveId,
} from "@port.finance/port-sdk";

import { Asset } from "./asset";

interface PortAccounts {
  program: PublicKey;
  market: PublicKey;
  marketAuthority: PublicKey;
  reserve: PublicKey;
  collateralMint: PublicKey;
  oracle: PublicKey;
  liquiditySupply: PublicKey;
}

// TODO use constant from port sdk
// WF port team to make it public
// https://github.com/port-finance/port-sdk/blob/v2/src/utils/AssetConfigs.ts
const DEVNET_ASSETS = [
  new AssetConfig(
    MintId.fromBase58("So11111111111111111111111111111111111111112"),
    new AssetDisplayConfig("Solana", "SOL", "#BC57C4"),
    AssetPriceConfig.fromDecimals(4),
    new AssetDepositConfig(
      ReserveId.fromBase58("6FeVStQAGPWvfWijDHF7cTWRCi7He6vTT3ubfNhe9SPt"),
      {
        min: 100_000_000, // min 0.1 SOL
        remain: 20_000_000, // remain 0.02 SOL
      }
    )
  ),
];

export class PortReserveAsset extends Asset {
  private constructor(
    public provider: anchor.Provider,
    public accounts: PortAccounts,
    public client: Port
  ) {
    super();
  }

  static async load(
    provider: anchor.Provider,
    cluster: Cluster,
    reserveMint: PublicKey
  ): Promise<PortReserveAsset> {
    let env: Environment;
    let market: PublicKey;
    if (cluster == "devnet") {
      env = new Environment(
        ENV.Devnet,
        DEVNET_LENDING_PROGRAM_ID,
        PORT_STAKING,
        TOKEN_PROGRAM_ID,
        DEVNET_ASSETS
      );
      market = new PublicKey("H27Quk3DSbu55T4dCr1NddTTSAezXwHU67FPCZVKLhSW");
    } else if (cluster == "mainnet-beta") {
      env = Environment.forMainNet();
      market = DEFAULT_PORT_LENDING_MARKET;
    } else {
      throw new Error("Cluster ${cluster} not supported");
    }
    const client = new Port(provider.connection, env, market);
    const reserveContext = await client.getReserveContext();
    const reserve = reserveContext.getByAssetMintId(MintId.of(reserveMint));
    const [authority, _] = await PublicKey.findProgramAddress(
      [market.toBuffer()],
      env.getLendingProgramPk()
    );
    const accounts: PortAccounts = {
      program: env.getLendingProgramPk(),
      market: market,
      marketAuthority: authority,
      reserve: reserve.getReserveId(),
      collateralMint: reserve.getShareMintId(),
      oracle: reserve.getOracleId(),
      liquiditySupply: reserve.getAssetBalanceId(),
    };

    return new PortReserveAsset(provider, accounts, client);
  }

  static async initialize(
    provider: anchor.Provider,
    owner: Keypair,
    reserveTokenMint: PublicKey,
    pythPrice: PublicKey,
    ownerReserveTokenAccount: PublicKey,
    initialReserveAmount: number
  ): Promise<PortReserveAsset> {
    const market = await createLendingMarket(provider);
    const accounts = await createDefaultReserve(
      provider,
      initialReserveAmount,
      reserveTokenMint,
      ownerReserveTokenAccount,
      market.publicKey,
      pythPrice,
      owner
    );
    const env = new Environment(
      ENV.Devnet,
      DEVNET_LENDING_PROGRAM_ID,
      null,
      TOKEN_PROGRAM_ID,
      []
    );
    const client = new Port(provider.connection, env, market.publicKey);
    return new PortReserveAsset(provider, accounts, client);
  }

  async getLpTokenAccountValue(address: PublicKey): Promise<Big> {
    const reserve = await this.client.getReserve(this.accounts.reserve);
    const exchangeRate = reserve.getExchangeRatio();

    const mint = reserve.getShareMintId();
    const lpToken = new Token(
      this.provider.connection,
      mint,
      TOKEN_PROGRAM_ID,
      Keypair.generate() // dummy signer since we aren't making any txs
    );

    const lpTokenAmount = AssetPrice.of(
      mint,
      (await lpToken.getAccountInfo(address)).amount.toNumber()
    );

    return lpTokenAmount.divide(exchangeRate.getUnchecked()).getRaw();
  }

  /**
   * @todo make this the same as program's calculation
   *
   * Continuously compounded APY
   *
   * @returns
   */
  async getApy(): Promise<Big> {
    const reserve = await this.client.getReserve(this.accounts.reserve);
    const apr = reserve.getSupplyApy().getUnchecked();
    const apy = Math.expm1(apr.toNumber());

    return new Big(apy);
  }
}

const DEVNET_LENDING_PROGRAM_ID = new PublicKey(
  "pdQ2rQQU5zH2rDgZ7xH2azMBJegUzUyunJ5Jd637hC4"
);
const TOKEN_ACCOUNT_LEN = 165;
const TOKEN_MINT_LEN = 82;
const RESERVE_LEN = 575;
const LENDING_MARKET_LEN = 258;

const DEFAULT_RESERVE_CONFIG: ReserveConfigProto = {
  optimalUtilizationRate: 80,
  loanToValueRatio: 80,
  liquidationBonus: 5,
  liquidationThreshold: 85,
  minBorrowRate: 0,
  optimalBorrowRate: 40,
  maxBorrowRate: 90,
  fees: {
    borrowFeeWad: new anchor.BN(10000000000000),
    flashLoanFeeWad: new anchor.BN(30000000000000),
    hostFeePercentage: 0,
  },
  stakingPoolOption: 0,
  stakingPool: TOKEN_PROGRAM_ID, // dummy
};

// TODO move to common utils
const createAccount = async (
  provider: anchor.Provider,
  space: number,
  owner: PublicKey
): Promise<Keypair> => {
  const newAccount = Keypair.generate();
  const createTx = new Transaction().add(
    SystemProgram.createAccount({
      fromPubkey: provider.wallet.publicKey,
      newAccountPubkey: newAccount.publicKey,
      programId: owner,
      lamports: await provider.connection.getMinimumBalanceForRentExemption(space),
      space,
    })
  );
  await provider.send(createTx, [newAccount]);
  return newAccount;
};

async function createLendingMarket(provider: anchor.Provider): Promise<Keypair> {
  const lendingMarket = await createAccount(
    provider,
    LENDING_MARKET_LEN,
    DEVNET_LENDING_PROGRAM_ID
  );
  await provider.send(
    (() => {
      const tx = new Transaction();
      tx.add(
        initLendingMarketInstruction(
          provider.wallet.publicKey,
          Buffer.from(
            "USD\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
            "ascii"
          ),
          lendingMarket.publicKey,
          DEVNET_LENDING_PROGRAM_ID
        )
      );
      return tx;
    })(),
    []
  );
  return lendingMarket;
}

async function createDefaultReserve(
  provider: anchor.Provider,
  initialLiquidity: number | anchor.BN,
  liquidityMint: PublicKey,
  sourceTokenWallet: PublicKey,
  lendingMarket: PublicKey,
  oracle: PublicKey,
  owner: Keypair
): Promise<PortAccounts> {
  const reserve = await createAccount(provider, RESERVE_LEN, DEVNET_LENDING_PROGRAM_ID);

  const collateralMintAccount = await createAccount(
    provider,
    TOKEN_MINT_LEN,
    TOKEN_PROGRAM_ID
  );

  const liquiditySupplyTokenAccount = await createAccount(
    provider,
    TOKEN_ACCOUNT_LEN,
    TOKEN_PROGRAM_ID
  );

  const collateralSupplyTokenAccount = await createAccount(
    provider,
    TOKEN_ACCOUNT_LEN,
    TOKEN_PROGRAM_ID
  );

  const userCollateralTokenAccount = await createAccount(
    provider,
    TOKEN_ACCOUNT_LEN,
    TOKEN_PROGRAM_ID
  );

  const liquidityFeeReceiver = await createAccount(
    provider,
    TOKEN_ACCOUNT_LEN,
    TOKEN_PROGRAM_ID
  );

  const [lendingMarketAuthority] = await PublicKey.findProgramAddress(
    [lendingMarket.toBuffer()],
    DEVNET_LENDING_PROGRAM_ID
  );

  const tx = new Transaction();

  tx.add(
    Token.createApproveInstruction(
      TOKEN_PROGRAM_ID,
      sourceTokenWallet,
      provider.wallet.publicKey,
      owner.publicKey,
      [],
      initialLiquidity
    )
  );
  tx.add(
    initReserveInstruction(
      initialLiquidity,
      0,
      new anchor.BN(0),
      DEFAULT_RESERVE_CONFIG,
      sourceTokenWallet,
      userCollateralTokenAccount.publicKey,
      reserve.publicKey,
      liquidityMint,
      liquiditySupplyTokenAccount.publicKey,
      liquidityFeeReceiver.publicKey,
      oracle,
      collateralMintAccount.publicKey,
      collateralSupplyTokenAccount.publicKey,
      lendingMarket,
      lendingMarketAuthority,
      provider.wallet.publicKey,
      provider.wallet.publicKey,
      DEVNET_LENDING_PROGRAM_ID
    )
  );

  await provider.send(tx, [owner]);

  return {
    program: DEVNET_LENDING_PROGRAM_ID,
    market: lendingMarket,
    marketAuthority: lendingMarketAuthority,
    reserve: reserve.publicKey,
    oracle: oracle,
    collateralMint: collateralMintAccount.publicKey,
    liquiditySupply: liquiditySupplyTokenAccount.publicKey,
  };
}
