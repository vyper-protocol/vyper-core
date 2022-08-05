import * as anchor from "@project-serum/anchor";
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect} from "chai";
import { RateSwitchboardPlugin } from '../src/plugins/ratePlugin/rateSwitchboard/Rate';
import {  SWITCHBOARD_AGGREGATORS } from '../src/plugins/ratePlugin/rateSwitchboard/SwitchboardAggregators' 

dotenv.config();

const rateSwitchboardPluginId = new PublicKey('2hGXiH1oEQwjCXRx8bNdHTi49ScZp7Mj2bxcjxtULKe1');


describe('Rate Switch Board Plugin', () => {

    const provider = anchor.AnchorProvider.env();
    const rateSwitchboardPlugin = RateSwitchboardPlugin.create(provider, rateSwitchboardPluginId);

    it("initialize and fetch existing rate mock state", async () => {
        await rateSwitchboardPlugin.initialize();
        const rateState = await rateSwitchboardPlugin.getRatePluginState();

        for (let i = 0; i < 10; i++) {
            if (i < SWITCHBOARD_AGGREGATORS.length) {
                expect(rateState.switchboardAggregators[i].toBase58()).to.eql(
                    SWITCHBOARD_AGGREGATORS[i].toBase58()
                );
                expect(rateState.fairValue[i]).to.be.not.eq(0);
            } else {
                expect(rateState.switchboardAggregators[i]).to.be.null;
                expect(rateState.fairValue[i]).to.be.eq(0);
            }
        }

        expect(rateState.refreshedSlot).to.be.gt(0);
    });

    it("refresh", async() => {
        await rateSwitchboardPlugin.initialize();

        const tx = new anchor.web3.Transaction();
        const instruction = await rateSwitchboardPlugin.getRefreshIX();
        tx.add(instruction);
        await provider.sendAndConfirm(tx);

        const rateState = await rateSwitchboardPlugin.getRatePluginState();
        expect(rateState.refreshedSlot).to.be.gt(0);
    })

    it("cannot change aggregators order", async () => {
        await rateSwitchboardPlugin.initialize();

        const tx = new anchor.web3.Transaction();
        const instruction = await rateSwitchboardPlugin.getRefreshIX();
        tx.add(instruction);
        await provider.sendAndConfirm(tx);

        try {
            await rateSwitchboardPlugin.program.methods
            .refresh()
            .accounts({
                rateData: rateSwitchboardPlugin.rateStateId
            }).remainingAccounts(
                SWITCHBOARD_AGGREGATORS.reverse().map((c) => ({
                    pubkey: c, isSigner: false, isWritable: false}))
            )
            .rpc()

            expect(true).to.be.eq(false);
        } catch(err) {
            expect(err.error.errorCode.code).to.be.eql("RequireKeysEqViolated");
        }
    });

    it("cannot provide less aggregators", async () => {
        await rateSwitchboardPlugin.initialize();

        const tx = new anchor.web3.Transaction();
        const instruction = await rateSwitchboardPlugin.getRefreshIX();
        tx.add(instruction);
        await provider.sendAndConfirm(tx);

        try {
            await rateSwitchboardPlugin.program.methods
            .refresh()
            .accounts({
                rateData: rateSwitchboardPlugin.rateStateId
            }).remainingAccounts(
                [SWITCHBOARD_AGGREGATORS[0], SWITCHBOARD_AGGREGATORS[1]].map((c) => ({
                    pubkey: c, isSigner: false, isWritable: false
                }))
            )
            .rpc();

            expect(true).to.be.eq(false);
        } catch(err) {
            expect(err.error.errorCode.code).to.be.eql("InvalidAggregatorsNumber");
        }
    }); 

});