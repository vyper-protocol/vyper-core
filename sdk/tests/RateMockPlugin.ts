import * as anchor from "@project-serum/anchor";
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect} from "chai";
import { RatePlugin } from '../src/plugins/ratePlugin/rateMock/Rate';

dotenv.config();

const rateMockPluginId = new PublicKey('FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG');

describe('Rate Mock Plugin', () => {

    const provider = anchor.AnchorProvider.env();
    const rateMockPlugin = RatePlugin.create(provider, rateMockPluginId);

    it("initialize", async () => {
        await rateMockPlugin.initialize();
        const rateState = await rateMockPlugin.getRatePluginState();
        expect(rateState.fairValue).to.eql(Array(10).fill(0));
    });
    
    it('fetch existing rate mock state', async () => {
        await rateMockPlugin.initialize();
        const rateState = await rateMockPlugin.getRatePluginState();
        
        assert.ok(rateState.fairValue != undefined);
        assert.ok(rateState.refreshedSlot != undefined);
    })

    it('set mock rate', async () => {
        await rateMockPlugin.initialize();
        let rateState = await rateMockPlugin.getRatePluginState();
        
        // with direct rpc call
        await rateMockPlugin.setFairValue(1500);
        rateState = await rateMockPlugin.getRatePluginState();
        expect(rateState.fairValue[0]).to.eq(1500);

        // with setFairValueIX
        const tx = new anchor.web3.Transaction();
        const instruction = await rateMockPlugin.getSetFairValueIX(2500);
        tx.add(instruction);
        await provider.sendAndConfirm(tx);

        rateState = await rateMockPlugin.getRatePluginState();
        expect(rateState.fairValue[0]).to.eq(2500);
    })

}); 