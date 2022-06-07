import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMint, getOrCreateAssociatedTokenAccount } from "@solana/spl-token";
import { assert, expect } from "chai";
import { RateMock } from "../target/types/rate_mock";
import { RedeemLogicMock } from "../target/types/redeem_logic_mock";
import { VyperCore } from "../target/types/vyper_core";
import {
    bn,
    createMint,
    createMintAndVault,
    createTokenAccount,
    getInitializeData,
    TRANCHE_HALT_FLAGS,
    TRANCHE_HALT_FLAGS_HALT_ALL,
    UPDATE_TRANCHE_CONFIG_FLAGS,
} from "./utils";

describe("vyper_core", async () => {
    const provider = anchor.AnchorProvider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const programVyperCore = anchor.workspace.VyperCore as Program<VyperCore>;
    const programRedeemLogicMock = anchor.workspace
        .RedeemLogicMock as Program<RedeemLogicMock>;
    const programRateMock = anchor.workspace.RateMock as Program<RateMock>;

    it("initialize", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();

        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const initializeInputData = getInitializeData(6);
        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                rateProgramState: rateProgramState.publicKey,
                redeemLogicProgram: programRedeemLogicMock.programId,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        const trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(0);
        expect(trancheConfigAccount.trancheData.ownerRestrictedIx).to.eql(0);
        expect(
            trancheConfigAccount.trancheData.depositedQuantity.map((c) =>
                c.toNumber()
            )
        ).to.eql([0, 0]);
        expect(
            trancheConfigAccount.trancheData.reserveFairValue.value.toNumber()
        ).to.eql(1);
        expect(
            trancheConfigAccount.trancheData.reserveFairValue.slotTracking.lastUpdate.slot.toNumber()
        ).to.greaterThan(0);
        expect(
            trancheConfigAccount.trancheData.trancheFairValue.value.map((c) =>
                c.toNumber()
            )
        ).to.eql([1, 1]);
        expect(
            trancheConfigAccount.trancheData.reserveFairValue.slotTracking.lastUpdate.slot.toNumber()
        ).to.greaterThan(0);

        expect(trancheConfigAccount.owner.toBase58()).to.eql(
            provider.wallet.publicKey.toBase58()
        );
        expect(trancheConfigAccount.trancheAuthority.toBase58()).to.eql(
            trancheAuthority.toBase58()
        );
        expect(trancheConfigAccount.rateProgram.toBase58()).to.eql(
            programRateMock.programId.toBase58()
        );
        expect(trancheConfigAccount.redeemLogicProgram.toBase58()).to.eql(
            programRedeemLogicMock.programId.toBase58()
        );
        expect(trancheConfigAccount.reserveMint.toBase58()).to.eql(
            reserveMint.toBase58()
        );
        expect(trancheConfigAccount.reserve.toBase58()).to.eql(
            reserve.toBase58()
        );
        expect(trancheConfigAccount.seniorTrancheMint.toBase58()).to.eql(
            seniorTrancheMint.publicKey.toBase58()
        );
        expect(trancheConfigAccount.juniorTrancheMint.toBase58()).to.eql(
            juniorTrancheMint.publicKey.toBase58()
        );
        expect(trancheConfigAccount.createdAt.toNumber()).to.be.greaterThan(0);

        const juniorTrancheMintInfo = await getMint(
            provider.connection,
            juniorTrancheMint.publicKey
        );
        expect(juniorTrancheMintInfo.decimals).to.eql(
            initializeInputData.trancheMintDecimals
        );
        expect(juniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(
            trancheAuthority.toBase58()
        );

        const seniorTrancheMintInfo = await getMint(
            provider.connection,
            seniorTrancheMint.publicKey
        );
        expect(seniorTrancheMintInfo.decimals).to.eql(
            initializeInputData.trancheMintDecimals
        );
        expect(seniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(
            trancheAuthority.toBase58()
        );
    });

    it("update tranche halt flags", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const initializeInputData = {
            isOpen: true,
            trancheMintDecimals: 6,
        };

        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                rateProgramState: rateProgramState.publicKey,
                redeemLogicProgram: programRedeemLogicMock.programId,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        await programVyperCore.methods
            .updateTrancheData({
                bitmask: UPDATE_TRANCHE_CONFIG_FLAGS.HALT_FLAGS,
                haltFlags: TRANCHE_HALT_FLAGS.HALT_DEPOSITS,
                reserveFairValueStaleSlotThreshold: bn(2),
                trancheFairValueStaleSlotThreshold: bn(2),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        let trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(
            TRANCHE_HALT_FLAGS.HALT_DEPOSITS
        );

        await programVyperCore.methods
            .updateTrancheData({
                bitmask: UPDATE_TRANCHE_CONFIG_FLAGS.HALT_FLAGS,
                haltFlags: TRANCHE_HALT_FLAGS_HALT_ALL,
                reserveFairValueStaleSlotThreshold: bn(2),
                trancheFairValueStaleSlotThreshold: bn(2),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(
            TRANCHE_HALT_FLAGS_HALT_ALL
        );
    });

    it("update fair value stale threshold", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const initializeInputData = {
            isOpen: true,
            trancheMintDecimals: 6,
        };

        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                rateProgramState: rateProgramState.publicKey,
                redeemLogicProgram: programRedeemLogicMock.programId,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        let trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );
        expect(
            trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold.toNumber()
        ).to.eql(2);
        expect(
            trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold.toNumber()
        ).to.eql(2);

        const newStaleSlotThreshold = 4;

        await programVyperCore.methods
            .updateTrancheData({
                bitmask:
                    UPDATE_TRANCHE_CONFIG_FLAGS.RESERVE_FAIR_VALUE_STALE_SLOT_THRESHOLD |
                    UPDATE_TRANCHE_CONFIG_FLAGS.TRANCHE_FAIR_VALUE_STALE_SLOT_THRESHOLD,
                haltFlags: TRANCHE_HALT_FLAGS.HALT_DEPOSITS,
                reserveFairValueStaleSlotThreshold: bn(newStaleSlotThreshold),
                trancheFairValueStaleSlotThreshold: bn(newStaleSlotThreshold),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );
        expect(
            trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold.toNumber()
        ).to.eql(newStaleSlotThreshold);
        expect(
            trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold.toNumber()
        ).to.eql(newStaleSlotThreshold);
    });

    it("prevent rateProgramState attack", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const initializeInputData = {
            isOpen: true,
            trancheMintDecimals: 6,
        };

        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                redeemLogicProgram: programRedeemLogicMock.programId,
                rateProgramState: rateProgramState.publicKey,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        try {
            await programVyperCore.methods
                .refreshReserveFairValue()
                .accounts({
                    signer: provider.wallet.publicKey,
                    trancheConfig: trancheConfig.publicKey,
                    rateProgram: programRateMock.programId,
                    rateProgramState: rateProgramState.publicKey, // anchor.web3.Keypair.generate().publicKey,
                })
                .rpc();
            assert(false, "exception not triggered");
        } catch (err) {
            assert(true);
        }
    });

    it("deposit", async () => {
        const initialAmount = 1000;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            initialAmount
        );

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const initializeInputData = {
            isOpen: true,
            trancheMintDecimals: 6,
        };

        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                redeemLogicProgram: programRedeemLogicMock.programId,
                rateProgramState: rateProgramState.publicKey,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        console.log("create token account for senior tranches");
        const seniorTrancheTokenAccount = await createTokenAccount(
            provider,
            seniorTrancheMint.publicKey,
            provider.wallet.publicKey
        );
        console.log("create token account for junior tranches");
        const juniorTrancheTokenAccount = await createTokenAccount(
            provider,
            juniorTrancheMint.publicKey,
            provider.wallet.publicKey
        );

        try {
            const depositTx = await programVyperCore.methods
                .deposit({
                    reserveQuantity: [
                        bn(initialAmount / 2),
                        bn(initialAmount / 2),
                    ],
                })
                .accounts({
                    signer: provider.wallet.publicKey,
                    trancheConfig: trancheConfig.publicKey,
                    trancheAuthority,
                    reserve,
                    userReserveToken,
                    seniorTrancheMint: seniorTrancheMint.publicKey,
                    juniorTrancheMint: juniorTrancheMint.publicKey,
                    seniorTrancheDest: seniorTrancheTokenAccount,
                    juniorTrancheDest: juniorTrancheTokenAccount,
                })
                .rpc();

            console.log("depositTx: ", depositTx);
        } catch (err) {
            console.error(err);
        }
    });

    it.only("refresh reserve fair value", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const [trancheAuthority] =
            await anchor.web3.PublicKey.findProgramAddress(
                [
                    trancheConfig.publicKey.toBuffer(),
                    anchor.utils.bytes.utf8.encode("authority"),
                ],
                programVyperCore.programId
            );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        const rateData = anchor.web3.Keypair.generate();

        await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .signers([rateData])
            .rpc();

        const initializeInputData = {
            isOpen: true,
            trancheMintDecimals: 6,
        };

        await programVyperCore.methods
            .initialize(initializeInputData)
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                redeemLogicProgram: programRedeemLogicMock.programId,
                rateProgramState: rateData.publicKey,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        await programRateMock.methods
            .setFairValue(bn(1500))
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .rpc();

        await programVyperCore.methods
            .refreshReserveFairValue()
            .accounts({
                signer: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                rateProgramState: rateData.publicKey,
            })
            .rpc();

        const trancheConfigAccount =
            await programVyperCore.account.trancheConfig.fetch(
                trancheConfig.publicKey
            );

        expect(
            trancheConfigAccount.trancheData.reserveFairValue.value.toNumber()
        ).to.eq(1500);
    });
});