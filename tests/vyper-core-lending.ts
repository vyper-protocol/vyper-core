import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import {
    createMint,
    createMintAndVault,
    createTokenAccount,
    getMintInfo,
    getTokenAccount,
} from "@project-serum/common";
import {
    ASSOCIATED_TOKEN_PROGRAM_ID,
    Token,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import assert from "assert";
import { bn, printProgramShortDetails, to_bps } from "./utils";
import { VyperCoreLending } from "../target/types/vyper_core_lending";
import {
    createTrancheConfigInput,
    createTranchesConfiguration,
    findTrancheConfig,
} from "./vyper-core-utils";
import { DEVNET_SOLEND_PROGRAM_ID } from "./solend/solend";

describe("vyper-core-lending", () => {
    // Configure the client to use the local cluster.
    anchor.setProvider(anchor.Provider.env());

    //@ts-ignore
    const programVyperCoreLending = anchor.workspace
        .VyperCoreLending as Program<VyperCoreLending>;

    console.log("########################");
    printProgramShortDetails(programVyperCoreLending as Program);
    console.log("########################");

    it("creates tranche", async () => {
        // define input data
        const inputData = createTrancheConfigInput();
        const mint = await createMint(programVyperCoreLending.provider);

        // initialize tranche config

        const {
            seniorTrancheMint,
            seniorTrancheMintBump,
            juniorTrancheMint,
            juniorTrancheMintBump,
        } = await createTranchesConfiguration(
            DEVNET_SOLEND_PROGRAM_ID,
            mint,
            programVyperCoreLending
        );

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
                    authority:
                        programVyperCoreLending.provider.wallet.publicKey,
                    trancheConfig,
                    mint,
                    seniorTrancheMint: seniorTrancheMint,
                    juniorTrancheMint: juniorTrancheMint,
                    protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
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

        const account =
            await programVyperCoreLending.account.trancheConfig.fetch(
                trancheConfig
            );

        assert.equal(account.depositedQuantity[0].toNumber(), 0);
        assert.equal(account.depositedQuantity[1].toNumber(), 0);
        assert.deepEqual(account.interestSplit, inputData.interestSplit);
        assert.deepEqual(account.capitalSplit, inputData.capitalSplit);
        assert.equal(account.createSerum, inputData.createSerum);
        assert.ok(account.createdAt.toNumber() > 0);

        const seniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            seniorTrancheMint
        );
        assert.equal(seniorTrancheMintInfo.decimals, 0);
        assert.equal(seniorTrancheMintInfo.supply.toNumber(), 0);

        const juniorTrancheMintInfo = await getMintInfo(
            programVyperCoreLending.provider,
            juniorTrancheMint
        );
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

            const {
                seniorTrancheMint,
                seniorTrancheMintBump,
                juniorTrancheMint,
                juniorTrancheMintBump,
            } = await createTranchesConfiguration(
                DEVNET_SOLEND_PROGRAM_ID,
                mint,
                programVyperCoreLending
            );

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
                        authority:
                            programVyperCoreLending.provider.wallet.publicKey,
                        trancheConfig,
                        mint,
                        seniorTrancheMint: seniorTrancheMint,
                        juniorTrancheMint: juniorTrancheMint,
                        protocolProgram: DEVNET_SOLEND_PROGRAM_ID,
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

            var trancheConfigAccount =
                await programVyperCoreLending.account.trancheConfig.fetch(
                    trancheConfig
                );

            assert.deepEqual(
                trancheConfigAccount.interestSplit,
                inputData.interestSplit
            );

            // update interest split
            var newInterestSplit = [to_bps(0.5), to_bps(1)];

            await programVyperCoreLending.rpc.updateInterestSplit(
                newInterestSplit,
                {
                    accounts: {
                        authority:
                            programVyperCoreLending.provider.wallet.publicKey,
                        trancheConfig,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                }
            );

            trancheConfigAccount =
                await programVyperCoreLending.account.trancheConfig.fetch(
                    trancheConfig
                );
            assert.deepEqual(
                trancheConfigAccount.interestSplit,
                newInterestSplit
            );
        });

        it("can't update interst split with wrong authority", async () => {
            var newInterestSplit = [to_bps(0.15), to_bps(1)];

            assert.rejects(async () => {
                await programVyperCoreLending.rpc.updateInterestSplit(
                    newInterestSplit,
                    {
                        accounts: {
                            authority:
                                programVyperCoreLending.provider.wallet
                                    .publicKey,
                            trancheConfig,
                            systemProgram: anchor.web3.SystemProgram.programId,
                        },
                        signers: [anchor.web3.Keypair.generate()],
                    }
                );
            }, Error);
        });

        it("updates capital split", async () => {
            // fetch tranche config

            var trancheConfigAccount =
                await programVyperCoreLending.account.trancheConfig.fetch(
                    trancheConfig
                );

            assert.deepEqual(
                trancheConfigAccount.capitalSplit,
                inputData.capitalSplit
            );

            // update interest split
            var newCapitalSplit = [to_bps(0.85), to_bps(0.15)];

            await programVyperCoreLending.rpc.updateCapitalSplit(
                newCapitalSplit,
                {
                    accounts: {
                        authority:
                            programVyperCoreLending.provider.wallet.publicKey,
                        trancheConfig,
                        systemProgram: anchor.web3.SystemProgram.programId,
                    },
                }
            );

            trancheConfigAccount =
                await programVyperCoreLending.account.trancheConfig.fetch(
                    trancheConfig
                );
            assert.deepEqual(
                trancheConfigAccount.capitalSplit,
                newCapitalSplit
            );
        });

        it("can't update interst split with wrong authority", async () => {
            var newInterestSplit = [to_bps(0.75), to_bps(0.25)];

            assert.rejects(async () => {
                await programVyperCoreLending.rpc.updateCapitalSplit(
                    newInterestSplit,
                    {
                        accounts: {
                            authority:
                                programVyperCoreLending.provider.wallet
                                    .publicKey,
                            trancheConfig,
                            systemProgram: anchor.web3.SystemProgram.programId,
                        },
                        signers: [anchor.web3.Keypair.generate()],
                    }
                );
            }, Error);
        });

        it("can't update capital split with wrong parameters", async () => {
            var newInterestSplit = [to_bps(0.75), to_bps(0.75)];

            assert.rejects(async () => {
                await programVyperCoreLending.rpc.updateCapitalSplit(
                    newInterestSplit,
                    {
                        accounts: {
                            authority:
                                programVyperCoreLending.provider.wallet
                                    .publicKey,
                            trancheConfig,
                            systemProgram: anchor.web3.SystemProgram.programId,
                        },
                        signers: [anchor.web3.Keypair.generate()],
                    }
                );
            }, Error);
        });
    });
});
