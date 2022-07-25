import * as anchor from "@project-serum/anchor";
import { Vyper } from '../src/index';
import { PublicKey} from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect } from "chai";
import {RedeemLogicLendingPlugin} from "../src/plugins/redeemLogicPlugin/redeemLogicLending/RedeemLogicLending";
import {RatePlugin} from "../src/plugins/ratePlugin/rateMock/Rate";
import {UpdateTrancheConfigFlags} from '../src/UpdateTrancheConfigFlags'
import {HaltFlags} from '../src/HaltFlags'
import { OwnerRestrictedIxFlags } from "../src/OwnerRestrictedIxFlags";
import { getMint } from "@solana/spl-token";
import {createMint, createMintAndVault, createTokenAccount, getTokenAccountAmount} from "../../tests/utils"

dotenv.config();

const vyperCoreId = new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U');
const rateMockPluginId = new PublicKey('FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG');
const redeemLogicLendingPluginId = new PublicKey('Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA');

describe('TrancheConfig', () => {

    const provider = anchor.AnchorProvider.env();

    it('initialised and fetch existing tranche configuration', async () => {
        
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        await redeemLogicLendingPlugin.initialize(5000);

        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        await rateMockPlugin.initialize();

        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
        );

        const trancheConfig = await vyper.getTrancheConfiguration();
        expect(trancheConfig.reserveMint.toBase58()).to.eql(reserveMint.toBase58());
        expect(trancheConfig.reserve.toBase58()).to.eql(vyper.reserve.toBase58());
        expect(trancheConfig.trancheData.haltFlags).to.eql(0);
        expect(trancheConfig.trancheData.ownerRestrictedIx).to.eql(0);
        expect(trancheConfig.trancheData.depositedQuantity).to.eql([0, 0]);
        expect(trancheConfig.trancheData.reserveFairValue.value).to.eql(Array(10).fill(10000));
        expect(trancheConfig.trancheData.reserveFairValue.slotTracking.lastUpdate.slot).to.greaterThan(0);
        expect(trancheConfig.trancheData.trancheFairValue.value).to.eql([10000, 10000]);
        expect(trancheConfig.trancheData.trancheFairValue.slotTracking.lastUpdate.slot).to.greaterThan(0);
        expect(trancheConfig.seniorTrancheMint.toBase58()).to.eql(vyper.seniorTrancheMint.toBase58());
        expect(trancheConfig.juniorTrancheMint.toBase58()).to.eql(vyper.juniorTrancheMint.toBase58());
        expect(trancheConfig.trancheAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());
        expect(trancheConfig.owner.toBase58()).to.eql(provider.wallet.publicKey.toBase58());
        expect(trancheConfig.rateProgram.toBase58()).to.eql(rateMockPlugin.getProgramId().toBase58());
        expect(trancheConfig.rateProgramState.toBase58()).to.eql(rateMockPlugin.rateStateId.toBase58());
        expect(trancheConfig.redeemLogicProgram.toBase58()).to.eql(redeemLogicLendingPlugin.getProgramId().toBase58());
        expect(trancheConfig.redeemLogicProgramState.toBase58()).to.eql(redeemLogicLendingPlugin.redeemLogicStateId.toBase58());
        expect(trancheConfig.createdAt).to.be.greaterThan(0);

        const juniorTrancheMintInfo = await getMint(provider.connection, vyper.juniorTrancheMint);
        expect(juniorTrancheMintInfo.decimals).to.eql(trancheMintDecimals);
        // @ts-ignore
        expect(juniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());

        const seniorTrancheMintInfo = await getMint(provider.connection, vyper.seniorTrancheMint);
        expect(seniorTrancheMintInfo.decimals).to.eql(trancheMintDecimals);
        // @ts-ignore
        expect(seniorTrancheMintInfo.mintAuthority.toBase58()).to.eql(vyper.trancheAuthority.toBase58());
    });

    it("update tranche halt flags", async () => {
        
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        await redeemLogicLendingPlugin.initialize(5000);

        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        await rateMockPlugin.initialize();

        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
        );
     
        await vyper.updateTrancheConfig(
            UpdateTrancheConfigFlags.HALT_FLAGS,
            HaltFlags.HALT_DEPOSITS,
            OwnerRestrictedIxFlags.NONE,
            2,
            2
        )

        let trancheConfigAccount = await vyper.getTrancheConfiguration();

        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(HaltFlags.HALT_DEPOSITS)

        await vyper.updateTrancheConfig(
            UpdateTrancheConfigFlags.HALT_FLAGS,
            HaltFlags.HALT_ALL,
            OwnerRestrictedIxFlags.NONE,
            2,
            2
        )

        trancheConfigAccount = await vyper.getTrancheConfiguration();

        expect(trancheConfigAccount.trancheData.haltFlags).to.eql(HaltFlags.HALT_ALL);

        await vyper.updateTrancheConfig(
            UpdateTrancheConfigFlags.HALT_FLAGS,
            HaltFlags.NONE,
            OwnerRestrictedIxFlags.NONE,
            2,
            2
        )
    })


    it("update fair value stale threshold", async () => {
        
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        await redeemLogicLendingPlugin.initialize(5000);

        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        await rateMockPlugin.initialize();

        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
        );

        let trancheConfigAccount = await vyper.getTrancheConfiguration();

        
        expect(trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold).to.eql(2);
        expect(trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold).to.eql(2);

        const newStaleSlotThreshold = 4;

        await vyper.updateTrancheConfig(
            UpdateTrancheConfigFlags.RESERVE_FAIR_VALUE_STALE_SLOT_THRESHOLD | UpdateTrancheConfigFlags.TRANCHE_FAIR_VALUE_STALE_SLOT_THRESHOLD,
            HaltFlags.HALT_ALL,
            OwnerRestrictedIxFlags.NONE,
            newStaleSlotThreshold,
            newStaleSlotThreshold,
        )
        trancheConfigAccount = await vyper.getTrancheConfiguration();

        expect(trancheConfigAccount.trancheData.reserveFairValue.slotTracking.staleSlotThreshold).to.eql(newStaleSlotThreshold);
        expect(trancheConfigAccount.trancheData.trancheFairValue.slotTracking.staleSlotThreshold).to.eql(newStaleSlotThreshold);
    
    });
    

    it('refresh tranche fair value', async () => {
        
        const trancheMintDecimals = 6;
        const reserveMint = await createMint(provider);

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        await redeemLogicLendingPlugin.initialize(5000);

        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        await rateMockPlugin.initialize();

        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
        );
        
        
        // with rpc call
        await rateMockPlugin.setFairValue(1500);
        await vyper.refreshTrancheFairValue();
        let trancheConfig = await vyper.getTrancheConfiguration();
        expect(trancheConfig.trancheData.reserveFairValue.value[0]).to.eq(1500);
        expect(trancheConfig.trancheData.trancheFairValue.value).to.eql([10000, 10000]);


        // with instruction call
        await rateMockPlugin.setFairValue(2500);
        const tx = new anchor.web3.Transaction();
        const instruction = await vyper.getRefreshTrancheFairValueIX();
        tx.add(instruction);
        await provider.sendAndConfirm(tx);
        trancheConfig = await vyper.getTrancheConfiguration();
        expect(trancheConfig.trancheData.reserveFairValue.value[0]).to.eq(2500);
        expect(trancheConfig.trancheData.trancheFairValue.value).to.eql([10000, 10000]);
    })


    it("deposit with reserve fair value 1", async() => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            10 * (seniorDepositAmount + juniorDepositAmount)
        );

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);

        await redeemLogicLendingPlugin.initialize(5000);
        await rateMockPlugin.initialize();
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            redeemLogicLendingPlugin,  
            rateMockPlugin,
            //owner
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
        tx.add(await rateMockPlugin.getSetFairValueIX(500));
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
        await provider.sendAndConfirm(tx)

        let trancheConfigAccount = await vyper.getTrancheConfiguration()
        expect(trancheConfigAccount.trancheData.depositedQuantity).to.eql([
            seniorDepositAmount,
            juniorDepositAmount,
        ]);
        // we have a tranche fair value of 1, so the amount of tranches is the same as the amount of reserve token deposited
        expect(await getTokenAccountAmount(provider, seniorTrancheTokenAccount)).to.eql(seniorDepositAmount);
        expect(await getTokenAccountAmount(provider, juniorTrancheTokenAccount)).to.eql(juniorDepositAmount);
    })


    it("multiple deposits", async () => {
        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const depositCount = 3;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            depositCount * (seniorDepositAmount + juniorDepositAmount)
        );

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);

        await redeemLogicLendingPlugin.initialize(5000);
        await rateMockPlugin.initialize();
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
            redeemLogicLendingPlugin,  
            rateMockPlugin,
            //owner
        );
        
        await rateMockPlugin.setFairValue(10000);
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
            tx.add(await rateMockPlugin.getRefreshIX());
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
    })

    it('redeem assests', async () => {

        const trancheMintDecimals = 6;
        const seniorDepositAmount = 1000 * 10 ** trancheMintDecimals;
        const juniorDepositAmount = 500 * 10 ** trancheMintDecimals;
        const [reserveMint, userReserveToken] = await createMintAndVault(
            provider,
            seniorDepositAmount + juniorDepositAmount
        );

        let redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
        await redeemLogicLendingPlugin.initialize(5000);

        let rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);
        await rateMockPlugin.initialize();

        let vyper = Vyper.create(provider,vyperCoreId,redeemLogicLendingPlugin,rateMockPlugin);
        await vyper.initialize(
            { trancheMintDecimals, ownerRestrictedIxs: 0, haltFlags: 0 },
            reserveMint,
        );

        await rateMockPlugin.setFairValue(1000);

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
        depositTx.add(await rateMockPlugin.getRefreshIX());
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

        const redeemTx = await vyper.getRedeemIx(
            await getTokenAccountAmount(provider, seniorTrancheTokenAccount),
            await getTokenAccountAmount(provider, juniorTrancheTokenAccount),
            userReserveToken,
            seniorTrancheTokenAccount,
            juniorTrancheTokenAccount
        )
        await provider.sendAndConfirm(redeemTx);

        const trancheConfig = await vyper.getTrancheConfiguration();
        expect(trancheConfig.trancheData.depositedQuantity).to.eql([0, 0]);

        expect(await getTokenAccountAmount(provider, userReserveToken)).to.eq(
            seniorDepositAmount + juniorDepositAmount
        );
        expect(await getTokenAccountAmount(provider, seniorTrancheTokenAccount)).to.eq(0);
        expect(await getTokenAccountAmount(provider, juniorTrancheTokenAccount)).to.eq(0);
    })

});