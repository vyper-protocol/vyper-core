import { AnchorProvider } from '@project-serum/anchor';
import { Vyper } from '../src/index';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect } from "chai";

dotenv.config();

describe('Redeem Lending State', () => {

    const provider = AnchorProvider.env();
    const vyper = Vyper.create(provider,
        new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'),
        new PublicKey('Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA'),
        new PublicKey('FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG')
    );
    it('fetch existing redeem logic configuration', async () => {

        const accounts = await provider.connection.getProgramAccounts(new PublicKey('Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA'));
        const redeemState = await vyper.getRedeemLendingConfiguration(accounts[0].pubkey);

        assert.ok(redeemState.interestSplit != undefined);
        expect(redeemState.fixedFeePerTranche != undefined);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58())
    })

}); 