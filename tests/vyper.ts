import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createMintAndVault, getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import assert from "assert";
import { Layout, f32, f64 } from "buffer-layout";
import { Vyper } from "../target/types/vyper";
import { bn, findAssociatedTokenAddress, to_bps } from "./utils";

describe.only("vyper", () => {
  const program = anchor.workspace.Vyper as Program<Vyper>;
  // console.log("Vyper program.programId: " + program.programId);

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  it("creates tranche", async () => {
    const inputData = getTrancheInputData();

    const [depositMint, depositGod] = await createMintAndVault(
      program.provider,
      inputData.quantity,
      program.provider.wallet.publicKey,
      0
    );
    console.log("depositMint: " + depositMint);

    const depositFromAccount = await findAssociatedTokenAddress(program.provider.wallet.publicKey, depositMint);

    const createDepositFromAccountTx = new anchor.web3.Transaction();
    createDepositFromAccountTx.add(
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        depositMint,
        depositFromAccount,
        program.provider.wallet.publicKey,
        program.provider.wallet.publicKey
      ),
      Token.createTransferInstruction(
        TOKEN_PROGRAM_ID,
        depositGod,
        depositFromAccount,
        program.provider.wallet.publicKey,
        [],
        inputData.quantity.toNumber()
      )
    );
    await program.provider.send(createDepositFromAccountTx);

    console.log("depositFromAccount: " + depositFromAccount);

    const depositFromAccountInfo = await getTokenAccount(program.provider, depositFromAccount);
    console.log("depositFromAccountInfo.owner: " + depositFromAccountInfo.owner);
    console.log("depositFromAccountInfo.mint: " + depositFromAccountInfo.mint);

    const [seniorTrancheMint, seniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("senior"), depositMint.toBuffer()],
      program.programId
    );
    const seniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, seniorTrancheMint);
    console.log("seniorTrancheMint: " + seniorTrancheMint);
    console.log("seniorTrancheVault: " + seniorTrancheVault);

    const [juniorTrancheMint, juniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("junior"), depositMint.toBuffer()],
      program.programId
    );
    const juniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, juniorTrancheMint);
    console.log("juniorTrancheMint: " + juniorTrancheMint);
    console.log("juniorTrancheVault: " + juniorTrancheVault);

    const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
      [depositMint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
      program.programId
    );
    console.log("trancheConfig:" + trancheConfig);
    console.log("trancheConfigBump:" + trancheConfigBump);

    const trancheVault = await findAssociatedTokenAddress(trancheConfig, depositMint);
    console.log("depositToAccount (tranche config owned account): " + trancheVault);

    // * * * * * * * * * * * * * * * * * * * * * * *
    // create tranche

    const tx = await program.rpc.createTranche(inputData, trancheConfigBump, seniorTrancheMintBump, juniorTrancheMintBump, {
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
    });
    console.log("tx", tx);

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

    const seniorTrancheMintInfo = await getMintInfo(program.provider, seniorTrancheMint);
    assert.equal(seniorTrancheMintInfo.decimals, 0);
    assert.equal(seniorTrancheMintInfo.supply.toNumber(), inputData.mintCount[0].toNumber());

    const seniorTokenAccountInfo = await getTokenAccount(program.provider, seniorTrancheVault);
    assert.equal(seniorTokenAccountInfo.amount.toNumber(), inputData.mintCount[0].toNumber());
    assert.deepEqual(seniorTokenAccountInfo.mint, seniorTrancheMint);
    assert.deepEqual(seniorTokenAccountInfo.owner, program.provider.wallet.publicKey);

    const juniorTrancheMintInfo = await getMintInfo(program.provider, juniorTrancheMint);
    assert.equal(juniorTrancheMintInfo.decimals, 0);
    assert.equal(juniorTrancheMintInfo.supply.toNumber(), inputData.mintCount[1].toNumber());

    const juniorTokenAccountInfo = await getTokenAccount(program.provider, juniorTrancheVault);
    assert.equal(juniorTokenAccountInfo.amount.toNumber(), inputData.mintCount[1].toNumber());
    assert.deepEqual(juniorTokenAccountInfo.mint, juniorTrancheMint);
    assert.deepEqual(juniorTokenAccountInfo.owner, program.provider.wallet.publicKey);

    const trancheVaultInfo = await getTokenAccount(program.provider, trancheVault);
    assert.equal(trancheVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
    assert.deepEqual(trancheVaultInfo.mint, depositMint);
  });

  it("creates tranche and redeem after end date", async () => {
    const createTrancheInputData = getTrancheInputData();

    const [depositMint, depositGod] = await createMintAndVault(
      program.provider,
      createTrancheInputData.quantity,
      program.provider.wallet.publicKey,
      0
    );
    const depositFromAccount = await findAssociatedTokenAddress(program.provider.wallet.publicKey, depositMint);

    const createDepositFromAccountTx = new anchor.web3.Transaction();
    createDepositFromAccountTx.add(
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        depositMint,
        depositFromAccount,
        program.provider.wallet.publicKey,
        program.provider.wallet.publicKey
      ),
      Token.createTransferInstruction(
        TOKEN_PROGRAM_ID,
        depositGod,
        depositFromAccount,
        program.provider.wallet.publicKey,
        [],
        createTrancheInputData.quantity.toNumber()
      )
    );
    await program.provider.send(createDepositFromAccountTx);

    const depositFromAccountInfo = await getTokenAccount(program.provider, depositFromAccount);

    const [seniorTrancheMint, seniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("senior"), depositMint.toBuffer()],
      program.programId
    );
    const seniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, seniorTrancheMint);

    const [juniorTrancheMint, juniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from("junior"), depositMint.toBuffer()],
      program.programId
    );
    const juniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, juniorTrancheMint);

    const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
      [depositMint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
      program.programId
    );

    const trancheVault = await findAssociatedTokenAddress(trancheConfig, depositMint);

    // create tranche

    await program.rpc.createTranche(createTrancheInputData, trancheConfigBump, seniorTrancheMintBump, juniorTrancheMintBump, {
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
    });

    const redeemInputData = {
      quantity: createTrancheInputData.quantity,
    };

    const tx = await program.rpc.redeem(redeemInputData, {
      accounts: {
        authority: program.provider.wallet.publicKey,
        trancheConfig,
        depositMint,
        trancheVault,
        depositToAccount: depositFromAccount,

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
  });
});
function getTrancheInputData() {
  return {
    quantity: bn(1000),
    capitalSplit: [to_bps(0.5), to_bps(1)],
    interestSplit: [to_bps(0.85), to_bps(1)],
    mintCount: [bn(4), bn(10)],
    startDate: bn(new Date("2022-01-01T10:00:00Z").getTime() / 1000),
    endDate: bn(new Date("2022-01-31T10:00:00Z").getTime() / 1000),
    createSerum: false,
    canMintMore: false,
  };
}
