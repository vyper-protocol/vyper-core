import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getAccount, getMint, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { assert, expect } from "chai";
import { OwnerRestrictedIxFlags } from "../sdk/src/OwnerRestrictedIxFlags";
import { RateMock } from "../target/types/rate_mock";
import { RedeemLogicLending } from "../target/types/redeem_logic_lending";
import { VyperCore } from "../target/types/vyper_core";
import { RateMockPlugin } from "./sdk/plugins/rates/RateMockPlugin";
import { RedeemLogicLendingPlugin } from "./sdk/plugins/redeemLogic/RedeemLogicLendingPlugin";
import { Vyper } from "./sdk/Vyper";
import {
    bn,
    createMint,
    createMintAndVault,
    createTokenAccount,
    getInitializeData,
    getTokenAccountAmount,
    TRANCHE_HALT_FLAGS,
    TRANCHE_HALT_FLAGS_HALT_ALL,
    UPDATE_TRANCHE_CONFIG_FLAGS,
} from "./utils";

describe("vyper_core", () => {
    const provider = anchor.AnchorProvider.env();
    // Configure the client to use the local cluster.
    anchor.setProvider(provider);

    const programVyperCore = anchor.workspace.VyperCore as Program<VyperCore>;
    const programRedeemLogicLending = anchor.workspace.RedeemLogicLending as Program<RedeemLogicLending>;
    const programRateMock = anchor.workspace.RateMock as Program<RateMock>;

    it("initialize", async () => {
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        const trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(vyper.trancheConfig);
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(0);
        expect(trancheConfigAccount.trancheData.ownerRestrictedIx).to.eql(0);
        expect(trancheConfigAccount.trancheData.depositedQuantity.map((c) => c.toNumber())).to.eql([0, 0]);
        // expect(trancheConfigAccount.trancheData.reserveFairValue.value).to.eql(Array(10).fill(10000));
        expect(
            //@ts-expect-error
            trancheConfigAccount.trancheData.reserveFairValue.slotTracking.lastUpdate.slot.toNumber()
        ).to.greaterThan(0);
        // expect(trancheConfigAccount.trancheData.trancheFairValue.value).to.eql([10000, 10000]);
        expect(
            //@ts-expect-error
            trancheConfigAccount.trancheData.trancheFairValue.slotTracking.lastUpdate.slot.toNumber()
        ).to.greaterThan(0);

        expect(trancheConfigAccount.owner.toBase58()).to.eql(provider.wallet.publicKey.toBase58());
        expect(trancheConfigAccount.trancheAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());
        expect(trancheConfigAccount.rateProgram.toBase58()).to.eql(programRateMock.programId.toBase58());
        expect(trancheConfigAccount.redeemLogicProgram.toBase58()).to.eql(
            programRedeemLogicLending.programId.toBase58()
        );
        expect(trancheConfigAccount.reserveMint.toBase58()).to.eql(reserveMint.toBase58());
        expect(trancheConfigAccount.reserve.toBase58()).to.eql(vyper.reserve.toBase58());
        expect(trancheConfigAccount.seniorTrancheMint.toBase58()).to.eql(vyper.seniorTrancheMint.toBase58());
        expect(trancheConfigAccount.juniorTrancheMint.toBase58()).to.eql(vyper.juniorTrancheMint.toBase58());
        expect(trancheConfigAccount.createdAt.toNumber()).to.be.greaterThan(0);

        const juniorTrancheMintInfo = await getMint(provider.connection, vyper.juniorTrancheMint);
        expect(juniorTrancheMintInfo.decimals).to.eql(trancheMintDecimals);
        expect(juniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());

        const seniorTrancheMintInfo = await getMint(provider.connection, vyper.seniorTrancheMint);
        expect(seniorTrancheMintInfo.decimals).to.eql(trancheMintDecimals);
        expect(seniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());
    });

    it("collect fees", async () => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            seniorDepositAmount + juniorDepositAmount
        );

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5, 15); // with fee
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        await rateMock.setFairValue(1000);

        const seniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.seniorTrancheMint,
            provider.wallet.publicKey
        );
        const juniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.juniorTrancheMint,
            provider.wallet.publicKey
        );

        const depositTx = new anchor.web3.Transaction();
        depositTx.add(await rateMock.getRefreshIX());
        depositTx.add(await vyper.getRefreshTrancheFairValueIX());
        depositTx.add(
            await vyper.getDepositIx(
                seniorDepositAmount,
                juniorDepositAmount,
                userReserveToken,
                seniorTrancheTokenAccount,
                juniorTrancheTokenAccount
            )
        );
        const signature = await provider.sendAndConfirm(depositTx);

        await vyper.refreshTrancheFairValue();

        const trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(vyper.trancheConfig);
        expect(trancheConfigAccount.trancheData.feeToCollectQuantity.toNumber()).to.eql(30);
    });

    it("update tranche halt flags", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const redeemLogicProgramState = anchor.web3.Keypair.generate();

        const [trancheAuthority] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), anchor.utils.bytes.utf8.encode("authority")],
            programVyperCore.programId
        );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        await programVyperCore.methods
            .initialize({ trancheMintDecimals: 6, ownerRestrictedIxs: 0, haltFlags: 0 })
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                rateProgramState: rateProgramState.publicKey,
                redeemLogicProgram: programRedeemLogicLending.programId,
                redeemLogicProgramState: redeemLogicProgramState.publicKey,
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
                ownerRestrictedIxs: 0,
                reserveFairValueStaleSlotThreshold: bn(2),
                trancheFairValueStaleSlotThreshold: bn(2),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        let trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(trancheConfig.publicKey);
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(TRANCHE_HALT_FLAGS.HALT_DEPOSITS);

        await programVyperCore.methods
            .updateTrancheData({
                bitmask: UPDATE_TRANCHE_CONFIG_FLAGS.HALT_FLAGS,
                haltFlags: TRANCHE_HALT_FLAGS_HALT_ALL,
                ownerRestrictedIxs: 0,
                reserveFairValueStaleSlotThreshold: bn(2),
                trancheFairValueStaleSlotThreshold: bn(2),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(trancheConfig.publicKey);
        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(TRANCHE_HALT_FLAGS_HALT_ALL);
    });

    it("update fair value stale threshold", async () => {
        const reserveMint = await createMint(provider);

        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const redeemLogicProgramState = anchor.web3.Keypair.generate();

        const [trancheAuthority] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), anchor.utils.bytes.utf8.encode("authority")],
            programVyperCore.programId
        );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );

        await programVyperCore.methods
            .initialize({ trancheMintDecimals: 6, ownerRestrictedIxs: 0, haltFlags: 0 })
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                rateProgramState: rateProgramState.publicKey,
                redeemLogicProgram: programRedeemLogicLending.programId,
                redeemLogicProgramState: redeemLogicProgramState.publicKey,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        let trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(trancheConfig.publicKey);
        //@ts-expect-error
        expect(trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold.toNumber()).to.eql(2);
        //@ts-expect-error
        expect(trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold.toNumber()).to.eql(2);

        const newStaleSlotThreshold = 4;

        await programVyperCore.methods
            .updateTrancheData({
                bitmask:
                    UPDATE_TRANCHE_CONFIG_FLAGS.RESERVE_FAIR_VALUE_STALE_SLOT_THRESHOLD |
                    UPDATE_TRANCHE_CONFIG_FLAGS.TRANCHE_FAIR_VALUE_STALE_SLOT_THRESHOLD,
                haltFlags: TRANCHE_HALT_FLAGS.HALT_DEPOSITS,
                ownerRestrictedIxs: 0,
                reserveFairValueStaleSlotThreshold: bn(newStaleSlotThreshold),
                trancheFairValueStaleSlotThreshold: bn(newStaleSlotThreshold),
            })
            .accounts({
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
            })
            .rpc();

        trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(trancheConfig.publicKey);
        //@ts-expect-error
        expect(trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold.toNumber()).to.eql(
            newStaleSlotThreshold
        );
        //@ts-expect-error
        expect(trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold.toNumber()).to.eql(
            newStaleSlotThreshold
        );
    });

    it("prevent rateProgramState attack", async () => {
        const reserveMint = await createMint(provider);
        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const rateProgramState = anchor.web3.Keypair.generate();
        const redeemLogicProgramState = anchor.web3.Keypair.generate();
        const [trancheAuthority] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), anchor.utils.bytes.utf8.encode("authority")],
            programVyperCore.programId
        );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            programVyperCore.programId
        );
        await programVyperCore.methods
            .initialize({ trancheMintDecimals: 6, ownerRestrictedIxs: 0, haltFlags: 0 })
            .accounts({
                payer: provider.wallet.publicKey,
                owner: provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: programRateMock.programId,
                redeemLogicProgram: programRedeemLogicLending.programId,
                redeemLogicProgramState: redeemLogicProgramState.publicKey,
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
                .refreshTrancheFairValue()
                .accounts({
                    signer: provider.wallet.publicKey,
                    trancheConfig: trancheConfig.publicKey,
                    rateProgramState: anchor.web3.Keypair.generate().publicKey,
                    redeemLogicProgram: programRedeemLogicLending.programId,
                    redeemLogicProgramState: redeemLogicProgramState.publicKey,
                })
                .rpc();
            expect(false).to.be.true;
        } catch (err) {
            assert(true);
        }
    });

    it("refresh tranche fair value", async () => {
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        await rateMock.setFairValue(1500);
        await vyper.refreshTrancheFairValue();

        // TODO deserialize fair values
        // const trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(vyper.trancheConfig);
        // expect(trancheConfigAccount.trancheData.reserveFairValue.value[0]).to.eq(1500);
        // expect(trancheConfigAccount.trancheData.trancheFairValue.value).to.eql([10000, 10000]);
    });

    it("deposit with reserve fair value 1", async () => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            10 * (seniorDepositAmount + juniorDepositAmount)
        );

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        const seniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.seniorTrancheMint,
            provider.wallet.publicKey
        );
        const juniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.juniorTrancheMint,
            provider.wallet.publicKey
        );
        const tx = new anchor.web3.Transaction();
        tx.add(await rateMock.getSetFairValueIX(500));
        tx.add(await vyper.getRefreshTrancheFairValueIX());
        tx.add(
            await vyper.getDepositIx(
                seniorDepositAmount,
                juniorDepositAmount,
                userReserveToken,
                seniorTrancheTokenAccount,
                juniorTrancheTokenAccount
            )
        );
        await provider.sendAndConfirm(tx);

        const trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(vyper.trancheConfig);
        expect(trancheConfigAccount.trancheData.depositedQuantity.map((c) => c.toNumber())).to.eql([
            seniorDepositAmount,
            juniorDepositAmount,
        ]);
        // we have a tranche fair value of 1, so the amount of tranches is the same as the amount of reserve token deposited
        expect(await getTokenAccountAmount(provider, seniorTrancheTokenAccount)).to.eql(seniorDepositAmount);
        expect(await getTokenAccountAmount(provider, juniorTrancheTokenAccount)).to.eql(juniorDepositAmount);
    });

    it("multiple deposits", async () => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const depositCount = 3;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            depositCount * (seniorDepositAmount + juniorDepositAmount)
        );

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        await rateMock.setFairValue(10000);

        const seniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.seniorTrancheMint,
            provider.wallet.publicKey
        );
        const juniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.juniorTrancheMint,
            provider.wallet.publicKey
        );

        for (let i = 0; i < depositCount; i++) {
            const tx = new anchor.web3.Transaction();
            tx.add(await rateMock.getRefreshIX());
            tx.add(await vyper.getRefreshTrancheFairValueIX());
            tx.add(
                await vyper.getDepositIx(
                    seniorDepositAmount,
                    juniorDepositAmount,
                    userReserveToken,
                    seniorTrancheTokenAccount,
                    juniorTrancheTokenAccount
                )
            );
            await provider.sendAndConfirm(tx);
        }

        expect(await getTokenAccountAmount(provider, vyper.reserve)).to.eq(
            depositCount * (seniorDepositAmount + juniorDepositAmount)
        );
    });

    it("redeem", async () => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            seniorDepositAmount + juniorDepositAmount
        );

        let redeemLogic = RedeemLogicLendingPlugin.create(programRedeemLogicLending, provider);
        let rateMock = RateMockPlugin.create(programRateMock, provider);
        let vyper = Vyper.create(programVyperCore, provider);

        await rateMock.initialize();
        await redeemLogic.initialize(0.5);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            rateMock.programID,
            rateMock.state,
            redeemLogic.programID,
            redeemLogic.state
        );

        await rateMock.setFairValue(1000);

        const seniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.seniorTrancheMint,
            provider.wallet.publicKey
        );
        const juniorTrancheTokenAccount = await createTokenAccount(
            provider,
            vyper.juniorTrancheMint,
            provider.wallet.publicKey
        );

        const depositTx = new anchor.web3.Transaction();
        depositTx.add(await rateMock.getRefreshIX());
        depositTx.add(await vyper.getRefreshTrancheFairValueIX());
        depositTx.add(
            await vyper.getDepositIx(
                seniorDepositAmount,
                juniorDepositAmount,
                userReserveToken,
                seniorTrancheTokenAccount,
                juniorTrancheTokenAccount
            )
        );
        await provider.sendAndConfirm(depositTx);

        const redeemTx = new anchor.web3.Transaction();
        redeemTx.add(await rateMock.getRefreshIX());
        redeemTx.add(await vyper.getRefreshTrancheFairValueIX());
        redeemTx.add(
            await vyper.getRedeemIx(
                await getTokenAccountAmount(provider, seniorTrancheTokenAccount),
                await getTokenAccountAmount(provider, juniorTrancheTokenAccount),
                userReserveToken,
                seniorTrancheTokenAccount,
                juniorTrancheTokenAccount
            )
        );
        await provider.sendAndConfirm(redeemTx);

        const trancheConfigAccount = await programVyperCore.account.trancheConfig.fetch(vyper.trancheConfig);
        expect(trancheConfigAccount.trancheData.depositedQuantity.map((c) => c.toNumber())).to.eql([0, 0]);

        expect(await getTokenAccountAmount(provider, userReserveToken)).to.eq(
            seniorDepositAmount + juniorDepositAmount
        );
        expect(await getTokenAccountAmount(provider, seniorTrancheTokenAccount)).to.eq(0);
        expect(await getTokenAccountAmount(provider, juniorTrancheTokenAccount)).to.eq(0);
    });
});
