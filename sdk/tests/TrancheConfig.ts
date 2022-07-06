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
import {createMint} from "../../tests/utils"

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
        expect(trancheConfig.redeemLogicProgramState.toBase58()).to.eql(redeemLogicLendingPlugin.redeemLendingStateId.toBase58());
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

});