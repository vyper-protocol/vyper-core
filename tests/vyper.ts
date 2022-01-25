import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
  createMintAndVault,
  getMintInfo,
  getTokenAccount,
  createAccountRentExempt,
} from "@project-serum/common";
import { Market } from "@project-serum/serum";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import assert from "assert";
import { Layout, f32, f64 } from "buffer-layout";
import { Vyper } from "../target/types/vyper";
import {
  bn,
  createDepositConfiguration,
  createTranchesConfiguration,
  findAssociatedTokenAddress,
  to_bps,
} from "./utils";
import { DEX_PID, createSerumAccounts } from "./serum/utils";

describe.only("vyper", () => {
  const program = anchor.workspace.Vyper as Program<Vyper>;
  // console.log("Vyper program.programId: " + program.programId);

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it("creates tranche", async () => {
    const inputData = getTrancheInputData();

    const [depositMint, depositFromAccount] = await createDepositConfiguration(
      inputData.quantity.toNumber(),
      program
    );

    const {
      seniorTrancheMint,
      seniorTrancheMintBump,
      seniorTrancheVault,
      juniorTrancheMint,
      juniorTrancheMintBump,
      juniorTrancheVault,
    } = await createTranchesConfiguration(depositMint, program);

    const [trancheConfig, trancheConfigBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          depositMint.toBuffer(),
          seniorTrancheMint.toBuffer(),
          juniorTrancheMint.toBuffer(),
        ],
        program.programId
      );
    console.log("trancheConfig:" + trancheConfig);
    console.log("trancheConfigBump:" + trancheConfigBump);

    const trancheVault = await findAssociatedTokenAddress(
      trancheConfig,
      depositMint
    );
    console.log(
      "depositToAccount (tranche config owned account): " + trancheVault
    );

    console.log("dexProgramId: " + DEX_PID);
    // * * * * * * * * * * * * * * * * * * * * * * *
    // create tranche

    const tx = await program.rpc.createTranche(
      inputData,
      trancheConfigBump,
      seniorTrancheMintBump,
      juniorTrancheMintBump,
      {
        accounts: {
          authority: program.provider.wallet.publicKey,

          trancheConfig,

          depositMint,
          depositFromAccount,
          trancheVault,

          seniorTrancheMint: seniorTrancheMint,
          seniorTrancheVault: seniorTrancheVault,
          juniorTrancheMint: juniorTrancheMint,
          juniorTrancheVault: juniorTrancheVault,

          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      }
    );
    console.log("tx", tx);

    const {
      market,
      requestQueue,
      eventQueue,
      asks,
      bids,
      vaultOwnerNonce,
      trancheSerumVault,
      usdcSerumVault,
    } = await createSerumAccounts(juniorTrancheMint, depositMint, program);

    const tx2 = await program.rpc.createSerumMarket(
      vaultOwnerNonce,
      {
        accounts: {
          authority: program.provider.wallet.publicKey,

          seniorTrancheMint: seniorTrancheMint,
          seniorTrancheSerumVault: seniorTrancheVault,
          juniorTrancheMint: juniorTrancheMint,
          juniorTrancheSerumVault: trancheSerumVault,

          market: market.publicKey,
          requestQueue: requestQueue.publicKey,
          eventQueue: eventQueue.publicKey,
          asks: asks.publicKey,
          bids: bids.publicKey,

          usdcMint: depositMint,
          usdcSerumVault,
          serumDex: DEX_PID,

          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
        signers: [market, requestQueue, eventQueue, asks, bids],
      }
    );
    console.log("tx2", tx2);

    // * * * * * * * * * * * * * * * * * * * * * * *
    // fetch tranche config

    const account = await program.account.trancheConfig.fetch(trancheConfig);

    assert.equal(account.quantity.toNumber(), inputData.quantity.toNumber());
    assert.deepEqual(account.interestSplit, inputData.interestSplit);
    assert.deepEqual(account.capitalSplit, inputData.capitalSplit);
    assert.deepEqual(
      account.mintCount.map((c) => c.toNumber()),
      inputData.mintCount.map((c) => c.toNumber())
    );
    assert.equal(account.startDate.toNumber(), inputData.startDate.toNumber());
    assert.equal(account.endDate.toNumber(), inputData.endDate.toNumber());
    assert.equal(account.canMintMore, inputData.canMintMore);
    assert.equal(account.createSerum, inputData.createSerum);
    assert.ok(account.createdAt.toNumber() > 0);

    const seniorTrancheMintInfo = await getMintInfo(
      program.provider,
      seniorTrancheMint
    );
    assert.equal(seniorTrancheMintInfo.decimals, 0);
    assert.equal(
      seniorTrancheMintInfo.supply.toNumber(),
      inputData.mintCount[0].toNumber()
    );

    const seniorTokenAccountInfo = await getTokenAccount(
      program.provider,
      seniorTrancheVault
    );
    assert.equal(
      seniorTokenAccountInfo.amount.toNumber(),
      inputData.mintCount[0].toNumber()
    );
    assert.deepEqual(seniorTokenAccountInfo.mint, seniorTrancheMint);
    assert.deepEqual(
      seniorTokenAccountInfo.owner,
      program.provider.wallet.publicKey
    );

    const juniorTrancheMintInfo = await getMintInfo(
      program.provider,
      juniorTrancheMint
    );
    assert.equal(juniorTrancheMintInfo.decimals, 0);
    assert.equal(
      juniorTrancheMintInfo.supply.toNumber(),
      inputData.mintCount[1].toNumber()
    );

    const juniorTokenAccountInfo = await getTokenAccount(
      program.provider,
      juniorTrancheVault
    );
    assert.equal(
      juniorTokenAccountInfo.amount.toNumber(),
      inputData.mintCount[1].toNumber()
    );
    assert.deepEqual(juniorTokenAccountInfo.mint, juniorTrancheMint);
    assert.deepEqual(
      juniorTokenAccountInfo.owner,
      program.provider.wallet.publicKey
    );

    const trancheVaultInfo = await getTokenAccount(
      program.provider,
      trancheVault
    );
    assert.equal(
      trancheVaultInfo.amount.toNumber(),
      inputData.quantity.toNumber()
    );
    assert.deepEqual(trancheVaultInfo.mint, depositMint);

    const loadedMarket = await Market.load(program.provider.connection, market.publicKey, {}, DEX_PID);
    assert.deepEqual(juniorTrancheMint, loadedMarket.baseMintAddress);
    assert.deepEqual(depositMint, loadedMarket.quoteMintAddress);
  });

  it("creates tranche and redeem everything", async () => {
    const createTrancheInputData = getTrancheInputData();

    const [depositMint, depositFromAccount] = await createDepositConfiguration(
      createTrancheInputData.quantity.toNumber(),
      program
    );

    const {
      seniorTrancheMint,
      seniorTrancheMintBump,
      seniorTrancheVault,
      juniorTrancheMint,
      juniorTrancheMintBump,
      juniorTrancheVault,
    } = await createTranchesConfiguration(depositMint, program);

    const [trancheConfig, trancheConfigBump] =
      await anchor.web3.PublicKey.findProgramAddress(
        [
          depositMint.toBuffer(),
          seniorTrancheMint.toBuffer(),
          juniorTrancheMint.toBuffer(),
        ],
        program.programId
      );

    const trancheVault = await findAssociatedTokenAddress(
      trancheConfig,
      depositMint
    );

    const marketAccount = await createAccountRentExempt(
      program.provider,
      program.programId,
      64
    );

    // create tranche

    await program.rpc.createTranche(
      createTrancheInputData,
      trancheConfigBump,
      seniorTrancheMintBump,
      juniorTrancheMintBump,
      {
        accounts: {
          authority: program.provider.wallet.publicKey,

          trancheConfig,

          depositMint,
          depositFromAccount,
          trancheVault,

          seniorTrancheMint: seniorTrancheMint,
          seniorTrancheVault: seniorTrancheVault,
          juniorTrancheMint: juniorTrancheMint,
          juniorTrancheVault: juniorTrancheVault,

          systemProgram: anchor.web3.SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        },
      }
    );

    const depositToAccount = depositFromAccount;

    const tx = await program.rpc.redeem({
      accounts: {
        authority: program.provider.wallet.publicKey,
        trancheConfig,
        depositMint,
        trancheVault,
        depositToAccount,

        seniorTrancheMint: seniorTrancheMint,
        seniorTrancheVault: seniorTrancheVault,
        juniorTrancheMint: juniorTrancheMint,
        juniorTrancheVault: juniorTrancheVault,

        systemProgram: anchor.web3.SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
      },
    });
    console.log("tx", tx);

    const depositToAccountInfo = await getTokenAccount(
      program.provider,
      depositToAccount
    );
    assert.equal(
      depositToAccountInfo.amount.toNumber(),
      createTrancheInputData.quantity.toNumber()
    );
    assert.deepEqual(depositToAccountInfo.mint, depositMint);

    const seniorTokenAccountInfo = await getTokenAccount(
      program.provider,
      seniorTrancheVault
    );
    assert.equal(seniorTokenAccountInfo.amount.toNumber(), 0);
    assert.deepEqual(seniorTokenAccountInfo.mint, seniorTrancheMint);
    assert.deepEqual(
      seniorTokenAccountInfo.owner,
      program.provider.wallet.publicKey
    );

    const juniorTokenAccountInfo = await getTokenAccount(
      program.provider,
      juniorTrancheVault
    );
    assert.equal(juniorTokenAccountInfo.amount.toNumber(), 0);
    assert.deepEqual(juniorTokenAccountInfo.mint, juniorTrancheMint);
    assert.deepEqual(
      juniorTokenAccountInfo.owner,
      program.provider.wallet.publicKey
    );
  });
});
function getTrancheInputData() {
  return {
    quantity: bn(1000),
    capitalSplit: [to_bps(0.5), to_bps(0.5)],
    interestSplit: [to_bps(0.15), to_bps(0.85)],
    mintCount: [bn(4), bn(10)],
    startDate: bn(new Date("2022-01-01T10:00:00Z").getTime() / 1000),
    endDate: bn(new Date("2022-01-31T10:00:00Z").getTime() / 1000),
    createSerum: false,
    canMintMore: false,
  };
}
