import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createMint, createMintAndVault, createTokenAccount, getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import assert from "assert";
import { bn, printProgramShortDetails, to_bps } from "./utils";
import { VyperCoreLending } from "../target/types/vyper_core_lending";
import { ProxyLendingSolend } from "../target/types/proxy_lending_solend";
import { createTrancheConfigInput, createTranchesConfiguration, findTrancheConfig } from "./vyper-core-utils";
import { VAULT_AUTHORITY } from "./constants";

describe("vyper-core-lending", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  //@ts-ignore
  const programVyperCoreLending = anchor.workspace.VyperCoreLending as Program<VyperCoreLending>;
  //@ts-ignore
  const programProxyLendingSolend = anchor.workspace.ProxyLendingSolend as Program<ProxyLendingSolend>;

  console.log("########################");
  printProgramShortDetails(programVyperCoreLending as Program);
  printProgramShortDetails(programProxyLendingSolend as Program);
  console.log("########################");

  it("creates tranche", async () => {
    // define input data
    const inputData = createTrancheConfigInput();
    const mint = await createMint(programVyperCoreLending.provider);

    // initialize tranche config

    const { seniorTrancheMint, seniorTrancheMintBump, juniorTrancheMint, juniorTrancheMintBump } =
      await createTranchesConfiguration(programProxyLendingSolend.programId, mint, programVyperCoreLending);

    const [trancheConfig, trancheConfigBump] = await findTrancheConfig(
      mint,
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
          mint,
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

    // * * * * * * * * * * * * * * * * * * * * * * *
    // fetch tranche config

    const account = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);

    assert.equal(account.depositedQuantiy[0].toNumber(), 0);
    assert.equal(account.depositedQuantiy[1].toNumber(), 0);
    assert.deepEqual(account.interestSplit, inputData.interestSplit);
    assert.deepEqual(account.capitalSplit, inputData.capitalSplit);
    assert.equal(account.createSerum, inputData.createSerum);
    assert.ok(account.createdAt.toNumber() > 0);

    const seniorTrancheMintInfo = await getMintInfo(programVyperCoreLending.provider, seniorTrancheMint);
    assert.equal(seniorTrancheMintInfo.decimals, 0);
    assert.equal(seniorTrancheMintInfo.supply.toNumber(), 0);

    const juniorTrancheMintInfo = await getMintInfo(programVyperCoreLending.provider, juniorTrancheMint);
    assert.equal(juniorTrancheMintInfo.decimals, 0);
    assert.equal(juniorTrancheMintInfo.supply.toNumber(), 0);
  });

  describe("update tranche config", async () => {
    const inputData = createTrancheConfigInput();
    let trancheConfig: anchor.web3.PublicKey;
    let trancheConfigBump: number;

    before(async () => {
      // define input data
      const mint = await createMint(programVyperCoreLending.provider);

      // initialize tranche config

      const { seniorTrancheMint, seniorTrancheMintBump, juniorTrancheMint, juniorTrancheMintBump } =
        await createTranchesConfiguration(programProxyLendingSolend.programId, mint, programVyperCoreLending);

      [trancheConfig, trancheConfigBump] = await findTrancheConfig(
        mint,
        seniorTrancheMint,
        juniorTrancheMint,
        programVyperCoreLending.programId
      );

      // vyper-core rpc: create tranche

      await programVyperCoreLending.rpc.createTranche(
        inputData,
        trancheConfigBump,
        seniorTrancheMintBump,
        juniorTrancheMintBump,
        {
          accounts: {
            authority: programVyperCoreLending.provider.wallet.publicKey,
            trancheConfig,
            mint,
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
    });

    it("updates interest split", async () => {
      // fetch tranche config

      var trancheConfigAccount = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);

      assert.deepEqual(trancheConfigAccount.interestSplit, inputData.interestSplit);

      // update interest split
      var newInterestSplit = [to_bps(0.5), to_bps(1)];

      await programVyperCoreLending.rpc.updateInterestSplit(newInterestSplit, {
        accounts: {
          authority: programVyperCoreLending.provider.wallet.publicKey,
          trancheConfig,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      });

      trancheConfigAccount = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);
      assert.deepEqual(trancheConfigAccount.interestSplit, newInterestSplit);
    });

    it("can't update interst split with wrong authority", async () => {
      var newInterestSplit = [to_bps(0.15), to_bps(1)];

      assert.rejects(async () => {
        await programVyperCoreLending.rpc.updateInterestSplit(newInterestSplit, {
          accounts: {
            authority: programVyperCoreLending.provider.wallet.publicKey,
            trancheConfig,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [anchor.web3.Keypair.generate()],
        });
      }, Error);
    });

    it("updates capital split", async () => {
      // fetch tranche config

      var trancheConfigAccount = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);

      assert.deepEqual(trancheConfigAccount.capitalSplit, inputData.capitalSplit);

      // update interest split
      var newCapitalSplit = [to_bps(0.85), to_bps(0.15)];

      await programVyperCoreLending.rpc.updateCapitalSplit(newCapitalSplit, {
        accounts: {
          authority: programVyperCoreLending.provider.wallet.publicKey,
          trancheConfig,
          systemProgram: anchor.web3.SystemProgram.programId,
        },
      });

      trancheConfigAccount = await programVyperCoreLending.account.trancheConfig.fetch(trancheConfig);
      assert.deepEqual(trancheConfigAccount.capitalSplit, newCapitalSplit);
    });

    it("can't update interst split with wrong authority", async () => {
      var newInterestSplit = [to_bps(0.75), to_bps(0.25)];

      assert.rejects(async () => {
        await programVyperCoreLending.rpc.updateCapitalSplit(newInterestSplit, {
          accounts: {
            authority: programVyperCoreLending.provider.wallet.publicKey,
            trancheConfig,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [anchor.web3.Keypair.generate()],
        });
      }, Error);
    });

    it("can't update capital split with wrong parameters", async () => {
      var newInterestSplit = [to_bps(0.75), to_bps(0.75)];

      assert.rejects(async () => {
        await programVyperCoreLending.rpc.updateCapitalSplit(newInterestSplit, {
          accounts: {
            authority: programVyperCoreLending.provider.wallet.publicKey,
            trancheConfig,
            systemProgram: anchor.web3.SystemProgram.programId,
          },
          signers: [anchor.web3.Keypair.generate()],
        });
      }, Error);
    });
  });

  // it("redeem with no profit and no loss and all tranches", async () => {
  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // define input data

  //   const inputData = {
  //     quantity: bn(1000),
  //     capitalSplit: [to_bps(0.85), to_bps(0.15)],
  //     interestSplit: [to_bps(0.85), to_bps(1)],
  //     mintCount: [bn(4), bn(10)],
  //     startDate: bn(new Date("2022-01-01T10:00:00Z").getTime() / 1000),
  //     endDate: bn(new Date("2022-01-31T10:00:00Z").getTime() / 1000),
  //     createSerum: false,
  //     canMintMore: false,
  //     protocolBump: 0,
  //   };

  //   const [mint, depositSourceAccount] = await createMintAndDepositSource(programVyper.provider, inputData.quantity.toNumber());

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: initialize protocol

  //   console.log("initialize mock protocol");
  //   const [protocolVault, protocolVaultBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
  //     programMockProtocol.programId
  //   );
  //   await programMockProtocol.rpc.initialize(protocolVaultBump, {
  //     accounts: {
  //       vault: protocolVault,
  //       mint,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });
  //   console.log("protocolVault: " + protocolVault);
  //   inputData.protocolBump = protocolVaultBump;

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // initialize tranche config

  //   console.log("creating tranche configs...");
  //   const {
  //     seniorTrancheMint,
  //     seniorTrancheMintBump,
  //     seniorTrancheVault,
  //     juniorTrancheMint,
  //     juniorTrancheMintBump,
  //     juniorTrancheVault,
  //   } = await createTranchesConfiguration(mint, programVyper);

  //   const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
  //     programVyper.programId
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: create tranche

  //   console.log("calling vyper createTranche...");
  //   const tx = await programVyper.rpc.createTranche(
  //     inputData,
  //     trancheConfigBump,
  //     seniorTrancheMintBump,
  //     juniorTrancheMintBump,
  //     {
  //       accounts: {
  //         authority: programVyper.provider.wallet.publicKey,

  //         trancheConfig,

  //         mint,
  //         depositSourceAccount,
  //         protocolVault,

  //         seniorTrancheMint: seniorTrancheMint,
  //         seniorTrancheVault: seniorTrancheVault,
  //         juniorTrancheMint: juniorTrancheMint,
  //         juniorTrancheVault: juniorTrancheVault,

  //         protocolProgram: programMockProtocol.programId,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       },
  //     }
  //   );
  //   console.log("tx", tx);

  //   let protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
  //   assert.deepEqual(protocolVaultInfo.mint, mint);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: Redeem

  //   await programVyper.rpc.redeem({
  //     accounts: {
  //       authority: programVyper.provider.wallet.publicKey,
  //       trancheConfig,
  //       mint,
  //       protocolVault,
  //       depositDestAccount: depositSourceAccount,

  //       seniorTrancheMint: seniorTrancheMint,
  //       seniorTrancheVault: seniorTrancheVault,
  //       juniorTrancheMint: juniorTrancheMint,
  //       juniorTrancheVault: juniorTrancheVault,

  //       protocolProgram: programMockProtocol.programId,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     },
  //   });

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), 0);
  //   const depositSourceAccountVaultInfo = await getTokenAccount(programVyper.provider, depositSourceAccount);
  //   assert.equal(depositSourceAccountVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
  // });

  // it("redeem with profit and all tranches", async () => {
  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // define input data
  //   const profitQuantity = 200;
  //   const inputData = {
  //     quantity: bn(1000),
  //     capitalSplit: [to_bps(0.85), to_bps(0.15)],
  //     interestSplit: [to_bps(0.85), to_bps(1)],
  //     mintCount: [bn(4), bn(10)],
  //     startDate: bn(new Date("2022-01-01T10:00:00Z").getTime() / 1000),
  //     endDate: bn(new Date("2022-01-31T10:00:00Z").getTime() / 1000),
  //     createSerum: false,
  //     canMintMore: false,
  //     protocolBump: 0,
  //   };

  //   const [mint, depositSourceAccount] = await createMintAndDepositSource(
  //     programVyper.provider,
  //     profitQuantity + inputData.quantity.toNumber()
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: initialize protocol

  //   console.log("initialize mock protocol");
  //   const [protocolVault, protocolVaultBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
  //     programMockProtocol.programId
  //   );
  //   await programMockProtocol.rpc.initialize(protocolVaultBump, {
  //     accounts: {
  //       vault: protocolVault,
  //       mint,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });
  //   console.log("protocolVault: " + protocolVault);
  //   inputData.protocolBump = protocolVaultBump;

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // initialize tranche config

  //   console.log("creating tranche configs...");
  //   const {
  //     seniorTrancheMint,
  //     seniorTrancheMintBump,
  //     seniorTrancheVault,
  //     juniorTrancheMint,
  //     juniorTrancheMintBump,
  //     juniorTrancheVault,
  //   } = await createTranchesConfiguration(mint, programVyper);

  //   const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
  //     programVyper.programId
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: create tranche

  //   console.log("calling vyper createTranche...");
  //   const tx = await programVyper.rpc.createTranche(
  //     inputData,
  //     trancheConfigBump,
  //     seniorTrancheMintBump,
  //     juniorTrancheMintBump,
  //     {
  //       accounts: {
  //         authority: programVyper.provider.wallet.publicKey,

  //         trancheConfig,

  //         mint,
  //         depositSourceAccount,
  //         protocolVault,

  //         seniorTrancheMint: seniorTrancheMint,
  //         seniorTrancheVault: seniorTrancheVault,
  //         juniorTrancheMint: juniorTrancheMint,
  //         juniorTrancheVault: juniorTrancheVault,

  //         protocolProgram: programMockProtocol.programId,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       },
  //     }
  //   );
  //   console.log("tx", tx);

  //   let protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
  //   assert.deepEqual(protocolVaultInfo.mint, mint);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: simulate interest -> deposit

  //   await programMockProtocol.rpc.deposit(bn(profitQuantity), protocolVaultBump, {
  //     accounts: {
  //       mint,
  //       vault: protocolVault,
  //       srcAccount: depositSourceAccount,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), profitQuantity + inputData.quantity.toNumber());

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: Redeem

  //   await programVyper.rpc.redeem({
  //     accounts: {
  //       authority: programVyper.provider.wallet.publicKey,
  //       trancheConfig,
  //       mint,
  //       protocolVault,
  //       depositDestAccount: depositSourceAccount,

  //       seniorTrancheMint: seniorTrancheMint,
  //       seniorTrancheVault: seniorTrancheVault,
  //       juniorTrancheMint: juniorTrancheMint,
  //       juniorTrancheVault: juniorTrancheVault,

  //       protocolProgram: programMockProtocol.programId,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     },
  //   });

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), 0);

  //   const depositSourceAccountVaultInfo = await getTokenAccount(programVyper.provider, depositSourceAccount);
  //   assert.equal(depositSourceAccountVaultInfo.amount.toNumber(), profitQuantity + inputData.quantity.toNumber());
  // });

  // it("redeem with loss and all tranches", async () => {
  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // define input data
  //   const lossQuantity = 200;
  //   const inputData = {
  //     quantity: bn(1000),
  //     capitalSplit: [to_bps(0.85), to_bps(0.15)],
  //     interestSplit: [to_bps(0.85), to_bps(1)],
  //     mintCount: [bn(4), bn(10)],
  //     startDate: bn(new Date("2022-01-01T10:00:00Z").getTime() / 1000),
  //     endDate: bn(new Date("2022-01-31T10:00:00Z").getTime() / 1000),
  //     createSerum: false,
  //     canMintMore: false,
  //     protocolBump: 0,
  //   };

  //   const [mint, depositSourceAccount] = await createMintAndDepositSource(programVyper.provider, inputData.quantity.toNumber());

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: initialize protocol

  //   console.log("initialize mock protocol");
  //   const [protocolVault, protocolVaultBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
  //     programMockProtocol.programId
  //   );
  //   await programMockProtocol.rpc.initialize(protocolVaultBump, {
  //     accounts: {
  //       vault: protocolVault,
  //       mint,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });
  //   console.log("protocolVault: " + protocolVault);
  //   inputData.protocolBump = protocolVaultBump;

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // initialize tranche config

  //   console.log("creating tranche configs...");
  //   const {
  //     seniorTrancheMint,
  //     seniorTrancheMintBump,
  //     seniorTrancheVault,
  //     juniorTrancheMint,
  //     juniorTrancheMintBump,
  //     juniorTrancheVault,
  //   } = await createTranchesConfiguration(mint, programVyper);

  //   const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
  //     programVyper.programId
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: create tranche

  //   console.log("calling vyper createTranche...");
  //   const tx = await programVyper.rpc.createTranche(
  //     inputData,
  //     trancheConfigBump,
  //     seniorTrancheMintBump,
  //     juniorTrancheMintBump,
  //     {
  //       accounts: {
  //         authority: programVyper.provider.wallet.publicKey,

  //         trancheConfig,

  //         mint,
  //         depositSourceAccount,
  //         protocolVault,

  //         seniorTrancheMint: seniorTrancheMint,
  //         seniorTrancheVault: seniorTrancheVault,
  //         juniorTrancheMint: juniorTrancheMint,
  //         juniorTrancheVault: juniorTrancheVault,

  //         protocolProgram: programMockProtocol.programId,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       },
  //     }
  //   );
  //   console.log("tx", tx);

  //   let protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
  //   assert.deepEqual(protocolVaultInfo.mint, mint);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: simulate loss -> redeem

  //   const hackerKP = anchor.web3.Keypair.generate();
  //   const hackerTokenAccount = await findAssociatedTokenAddress(hackerKP.publicKey, mint);

  //   const mintToTx = new anchor.web3.Transaction();
  //   mintToTx.add(
  //     Token.createAssociatedTokenAccountInstruction(
  //       ASSOCIATED_TOKEN_PROGRAM_ID,
  //       TOKEN_PROGRAM_ID,
  //       mint,
  //       hackerTokenAccount,
  //       hackerKP.publicKey,
  //       programVyper.provider.wallet.publicKey
  //     )
  //   );
  //   await programVyper.provider.send(mintToTx);

  //   await programMockProtocol.rpc.redeem(bn(lossQuantity), protocolVaultBump, {
  //     accounts: {
  //       mint,
  //       vault: protocolVault,
  //       destAccount: hackerTokenAccount,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });

  //   const hackerVaultInfo = await getTokenAccount(programVyper.provider, hackerTokenAccount);
  //   assert.equal(hackerVaultInfo.amount.toNumber(), lossQuantity);

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber() - lossQuantity);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: Redeem

  //   await programVyper.rpc.redeem({
  //     accounts: {
  //       authority: programVyper.provider.wallet.publicKey,
  //       trancheConfig,
  //       mint,
  //       protocolVault,
  //       depositDestAccount: depositSourceAccount,

  //       seniorTrancheMint: seniorTrancheMint,
  //       seniorTrancheVault: seniorTrancheVault,
  //       juniorTrancheMint: juniorTrancheMint,
  //       juniorTrancheVault: juniorTrancheVault,

  //       protocolProgram: programMockProtocol.programId,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     },
  //   });

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), 0);

  //   const depositSourceAccountVaultInfo = await getTokenAccount(programVyper.provider, depositSourceAccount);
  //   assert.equal(depositSourceAccountVaultInfo.amount.toNumber(), inputData.quantity.toNumber() - lossQuantity);
  // });

  // it("redeem with profit and only senior tranches", async () => {
  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // define input data
  //   const profitQuantity = 200;
  //   const inputData = {
  //     quantity: bn(1000),
  //     capitalSplit: [to_bps(0.85), to_bps(0.15)],
  //     interestSplit: [to_bps(0.85), to_bps(1)],
  //     mintCount: [bn(4), bn(10)],
  //     startDate: bn(new Date("2021-01-01T10:00:00Z").getTime() / 1000),
  //     endDate: bn(new Date("2021-01-31T10:00:00Z").getTime() / 1000),
  //     createSerum: false,
  //     canMintMore: false,
  //     protocolBump: 0,
  //   };

  //   const [mint, depositSourceAccount] = await createMintAndDepositSource(
  //     programVyper.provider,
  //     profitQuantity + inputData.quantity.toNumber()
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: initialize protocol

  //   console.log("initialize mock protocol");
  //   const [protocolVault, protocolVaultBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
  //     programMockProtocol.programId
  //   );
  //   await programMockProtocol.rpc.initialize(protocolVaultBump, {
  //     accounts: {
  //       vault: protocolVault,
  //       mint,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });
  //   console.log("protocolVault: " + protocolVault);
  //   inputData.protocolBump = protocolVaultBump;

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // initialize tranche config

  //   console.log("creating tranche configs...");
  //   const {
  //     seniorTrancheMint,
  //     seniorTrancheMintBump,
  //     seniorTrancheVault,
  //     juniorTrancheMint,
  //     juniorTrancheMintBump,
  //     juniorTrancheVault,
  //   } = await createTranchesConfiguration(mint, programVyper);

  //   const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
  //     [mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
  //     programVyper.programId
  //   );

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: create tranche

  //   console.log("calling vyper createTranche...");
  //   const tx = await programVyper.rpc.createTranche(
  //     inputData,
  //     trancheConfigBump,
  //     seniorTrancheMintBump,
  //     juniorTrancheMintBump,
  //     {
  //       accounts: {
  //         authority: programVyper.provider.wallet.publicKey,

  //         trancheConfig,

  //         mint,
  //         depositSourceAccount,
  //         protocolVault,

  //         seniorTrancheMint: seniorTrancheMint,
  //         seniorTrancheVault: seniorTrancheVault,
  //         juniorTrancheMint: juniorTrancheMint,
  //         juniorTrancheVault: juniorTrancheVault,

  //         protocolProgram: programMockProtocol.programId,
  //         systemProgram: anchor.web3.SystemProgram.programId,
  //         tokenProgram: TOKEN_PROGRAM_ID,
  //         associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //         rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       },
  //     }
  //   );
  //   console.log("tx", tx);

  //   let protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber());
  //   assert.deepEqual(protocolVaultInfo.mint, mint);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // MOCK PROTOCOL: simulate interest -> deposit

  //   await programMockProtocol.rpc.deposit(bn(profitQuantity), protocolVaultBump, {
  //     accounts: {
  //       mint,
  //       vault: protocolVault,
  //       srcAccount: depositSourceAccount,
  //       authority: programMockProtocol.provider.wallet.publicKey,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //     },
  //   });

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), profitQuantity + inputData.quantity.toNumber());

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // sell my junior tranches on serum

  //   console.log("selling junior tranches on serum.. ");
  //   const buyerUserKP = anchor.web3.Keypair.generate();
  //   const buyerUserJuniorTranchesTokenAccount = await findAssociatedTokenAddress(buyerUserKP.publicKey, juniorTrancheMint);

  //   const sellTx = new anchor.web3.Transaction();
  //   sellTx.add(
  //     Token.createAssociatedTokenAccountInstruction(
  //       ASSOCIATED_TOKEN_PROGRAM_ID,
  //       TOKEN_PROGRAM_ID,
  //       juniorTrancheMint,
  //       buyerUserJuniorTranchesTokenAccount,
  //       buyerUserKP.publicKey,
  //       programVyper.provider.wallet.publicKey
  //     ),
  //     Token.createTransferInstruction(
  //       TOKEN_PROGRAM_ID,
  //       juniorTrancheVault,
  //       buyerUserJuniorTranchesTokenAccount,
  //       programVyper.provider.wallet.publicKey,
  //       [],
  //       inputData.mintCount[1].toNumber() // all the junior tranche tokens
  //     )
  //   );
  //   await programVyper.provider.send(sellTx);
  //   console.log("junior tranche tokens sold");

  //   const juniorVaultTokenInfo = await getTokenAccount(programVyper.provider, juniorTrancheVault);
  //   assert.equal(juniorVaultTokenInfo.amount.toNumber(), 0);

  //   // * * * * * * * * * * * * * * * * * * * * * * *
  //   // VYPER: Redeem

  //   await programVyper.rpc.redeem({
  //     accounts: {
  //       authority: programVyper.provider.wallet.publicKey,
  //       trancheConfig,
  //       mint,
  //       protocolVault,
  //       depositDestAccount: depositSourceAccount,

  //       seniorTrancheMint: seniorTrancheMint,
  //       seniorTrancheVault: seniorTrancheVault,
  //       juniorTrancheMint: juniorTrancheMint,
  //       juniorTrancheVault: juniorTrancheVault,

  //       protocolProgram: programMockProtocol.programId,
  //       systemProgram: anchor.web3.SystemProgram.programId,
  //       tokenProgram: TOKEN_PROGRAM_ID,
  //       associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  //       rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  //     },
  //   });

  //   let expectedRedeemQuantity: number = 0;

  //   // capital
  //   expectedRedeemQuantity += inputData.quantity.toNumber() * from_bps(inputData.capitalSplit[0]);
  //   // interest
  //   expectedRedeemQuantity += profitQuantity * from_bps(inputData.capitalSplit[0]) * from_bps(inputData.interestSplit[0]);

  //   const depositSourceAccountVaultInfo = await getTokenAccount(programVyper.provider, depositSourceAccount);
  //   assert.equal(depositSourceAccountVaultInfo.amount.toNumber(), expectedRedeemQuantity);

  //   protocolVaultInfo = await getTokenAccount(programVyper.provider, protocolVault);
  //   assert.equal(protocolVaultInfo.amount.toNumber(), inputData.quantity.toNumber() + profitQuantity - expectedRedeemQuantity);
  // });
});
function createMintAndTokenAccount() {
  throw new Error("Function not implemented.");
}
