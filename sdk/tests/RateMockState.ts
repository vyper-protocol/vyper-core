import { AnchorProvider } from '@project-serum/anchor';
import { Vyper } from '../src/index';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect } from "chai";

dotenv.config();

describe('Rate Mock State', () => {

    const provider = AnchorProvider.env();
    const vyper = Vyper.create(provider, new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'));
    it('fetch existing rate mock state', async () => {

        vyper.createRateMockProgram(provider, new PublicKey('FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG'));
        const accounts = await provider.connection.getProgramAccounts(new PublicKey('FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG'));
        const rateState = await vyper.getRateMockState(accounts[0].pubkey);

        assert.ok(rateState.fairValue != undefined);
        assert.ok(rateState.refreshedSlot != undefined);
    })

}); 