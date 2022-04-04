import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createMintAndVault, createTokenAccount, getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import assert from "assert";
import { bn } from "./utils";
import { createTrancheConfigInput, createTranchesConfiguration, findTrancheConfig } from "./vyper-core-utils";
import { SolendReserveAsset, DEVNET_SOLEND_PROGRAM_ID, pythPrice, switchboardFeed } from "./solend/solend";

describe.only("vyper-core-lending-solend", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  //@ts-ignore
  const programVyperCoreLending = anchor.workspace.VyperCoreLending as Program<VyperCoreLending>;
  //@ts-ignore
  const programProxyLendingSolend = anchor.workspace.ProxyLendingSolend as Program<ProxyLendingSolend>;

  it("deposit as senior on lending protocol", async () => {
    // define input data
    const inputData = createTrancheConfigInput();
    const quantityToDeposit = 1000;

    // init SOLEND
    const solendInit = await initLendingMarkets();

    // mint reserve token to user wallet
    var userReserveTokenAccount = await createTokenAccount(
      programVyperCoreLending.provider,
      solendInit.reserveToken,
      programVyperCoreLending.provider.wallet.publicKey
    );

    const mintToTx = new anchor.web3.Transaction();
    mintToTx.add(
      Token.createMintToInstruction(
        TOKEN_PROGRAM_ID,
        solendInit.reserveToken,
        userReserveTokenAccount,
        programVyperCoreLending.provider.wallet.publicKey,
        [solendInit.owner],
        quantityToDeposit
      )
    );
    await programVyperCoreLending.provider.send(mintToTx, [solendInit.owner]);

    const userReserveTokenAccountInfo = await getTokenAccount(programVyperCoreLending.provider, userReserveTokenAccount);
    assert.equal(userReserveTokenAccountInfo.amount, quantityToDeposit);

    // initialize tranche config

    const { seniorTrancheMint, seniorTrancheMintBump, juniorTrancheMint, juniorTrancheMintBump } =
      await createTranchesConfiguration(programProxyLendingSolend.programId, solendInit.reserveToken, programVyperCoreLending);

    const [trancheConfig, trancheConfigBump] = await findTrancheConfig(
      solendInit.reserveToken,
      seniorTrancheMint,
      juniorTrancheMint,
      programVyperCoreLending.programId
    );

    // vyper-core rpc: create tranche

    const tx = await programVyperCoreLending.rpc.createTranche(
      inputData,
      trancheConfigBump,
      seniorTrancheMintBump,
      juniorTrancheMintBump,
      {
        accounts: {
          authority: programVyperCoreLending.provider.wallet.publicKey,
          trancheConfig,
          mint: solendInit.reserveToken,
          seniorTrancheMint: seniorTrancheMint,
          juniorTrancheMint: juniorTrancheMint,
          proxyProtocolProgram: programProxyLendingSolend.programId,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      }
    );

    const seniorTrancheVault = await createTokenAccount(
      programVyperCoreLending.provider,
      seniorTrancheMint,
      programVyperCoreLending.provider.wallet.publicKey
    );
    const juniorTrancheVault = await createTokenAccount(
      programVyperCoreLending.provider,
      juniorTrancheMint,
      programVyperCoreLending.provider.wallet.publicKey
    );

    const vyperCollateralTokenAccount = await createTokenAccount(
      programVyperCoreLending.provider,
      new anchor.web3.PublicKey(solendInit.reserve.reserve.config.collateralMintAddress),
      trancheConfig
    );

    // deposit on lending protocol

    const seniorTrancheMintQuantity = 150;
    const juniorTrancheMintQuantity = 50;

    const tx2 = await programVyperCoreLending.rpc.deposit(
      bn(quantityToDeposit),
      [bn(seniorTrancheMintQuantity), bn(juniorTrancheMintQuantity)],
      {
        accounts: {
          authority: programVyperCoreLending.provider.wallet.publicKey,
          trancheConfig,
          reserveToken: solendInit.reserveToken,
          sourceLiquidity: userReserveTokenAccount,

          depositToProtocolReserve: solendInit.reserve.accounts.liquiditySupply,
          collateralTokenAccount: vyperCollateralTokenAccount,
          collateralMint: solendInit.reserve.reserve.config.collateralMintAddress,
          protocolState: new anchor.web3.PublicKey(solendInit.reserve.reserve.config.address),
          lendingMarketAccount: solendInit.marketKeypair.publicKey,
          lendingMarketAuthority: solendInit.reserve.accounts.marketAuthority,
          pythReserveLiquidityOracle: pythPrice,
          switchboardReserveLiquidityOracle: switchboardFeed,

          seniorTrancheMint,
          seniorTrancheVault,

          juniorTrancheMint,
          juniorTrancheVault,

          proxyProtocolProgram: programProxyLendingSolend.programId,
          protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      }
    );
    console.log("tx: " + tx2);

    const account = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);

    assert.equal(
      account.depositedQuantiy
        .map((c) => c.toNumber())
        .reduce((a: number, b: number): number => {
          return a + b;
        }, 0),
      quantityToDeposit
    );
    assert.deepEqual(account.interestSplit, inputData.interestSplit);
    assert.deepEqual(account.capitalSplit, inputData.capitalSplit);

    const seniorTrancheMintInfo = await getMintInfo(programVyperCoreLending.provider, seniorTrancheMint);
    assert.equal(seniorTrancheMintInfo.decimals, 0);
    assert.equal(seniorTrancheMintInfo.supply.toNumber(), seniorTrancheMintQuantity);

    const seniorTrancheVaultInto = await getTokenAccount(programVyperCoreLending.provider, seniorTrancheVault);
    assert.equal(seniorTrancheVaultInto.amount, seniorTrancheMintQuantity);

    const juniorTrancheMintInfo = await getMintInfo(programVyperCoreLending.provider, juniorTrancheMint);
    assert.equal(juniorTrancheMintInfo.decimals, 0);
    assert.equal(juniorTrancheMintInfo.supply.toNumber(), juniorTrancheMintQuantity);

    const juniorTrancheVaultInto = await getTokenAccount(programVyperCoreLending.provider, juniorTrancheVault);
    assert.equal(juniorTrancheVaultInto.amount, juniorTrancheMintQuantity);
  });

  interface InitLendingMarketResult {
    reserve: SolendReserveAsset;
    marketKeypair: anchor.web3.Keypair;
    owner: anchor.web3.Keypair;
    reserveToken: anchor.web3.PublicKey;
    ownerReserveTokenAccount: anchor.web3.PublicKey;
  }

  async function initLendingMarkets(): Promise<InitLendingMarketResult> {
    // console.log("init lending markets (castle-finance)");

    // const sig = await programVyperCoreLending.provider.connection.requestAirdrop(solendOwner.publicKey, 1000000000);
    // const supplSig = await programVyperCoreLending.provider.connection.requestAirdrop(referralFeeOwner, 1000000000);
    // await programVyperCoreLending.provider.connection.confirmTransaction(sig, "singleGossip");
    // await programVyperCoreLending.provider.connection.confirmTransaction(supplSig, "singleGossip");

    const initialReserveAmount = 100;
    const solendOwner = anchor.web3.Keypair.generate();
    const [reserveToken, ownerReserveTokenAccount] = await createMintAndVault(
      programVyperCoreLending.provider,
      bn(3 * initialReserveAmount),
      solendOwner.publicKey,
      2
    );

    const pythProduct = new anchor.web3.PublicKey("ALP8SdU9oARYVLgLR7LrqMNCYBnhtnQz1cj6bwgwQmgj");

    const pythProgram = new anchor.web3.PublicKey("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH");
    const switchboardProgram = new anchor.web3.PublicKey("DtmE9D2CSB4L5D6A15mraeEjrGMm6auWVzgaD8hK2tZM");

    // console.log("init lending markets:");
    // console.log("pyth product: " + pythProduct);
    // console.log("pyth price: " + pythPrice);
    // console.log("switchboard feed: " + switchboardFeed);
    // console.log("pyth program: " + pythProgram);
    // console.log("switchboard program: " + switchboardProgram);

    const [solendReserve, marketKeypair] = await SolendReserveAsset.initialize(
      programVyperCoreLending.provider,
      solendOwner,
      // @ts-ignore
      programVyperCoreLending.provider.wallet,
      reserveToken,
      pythProgram,
      switchboardProgram,
      pythProduct,
      pythPrice,
      switchboardFeed,
      ownerReserveTokenAccount,
      initialReserveAmount
    );

    return {
      owner: solendOwner,
      marketKeypair,
      ownerReserveTokenAccount,
      reserveToken,
      reserve: solendReserve,
    };
  }
});
