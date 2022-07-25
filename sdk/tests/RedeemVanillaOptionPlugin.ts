import * as anchor from '@project-serum/anchor'
import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { expect } from "chai";
import { RedeemLogicVanillaOptionPlugin } from '../src/plugins/redeemLogicPlugin/redeemLogicVanillaOption/RedeemLogicVanillaOption';

dotenv.config();

const redeemLogicVanillaPluginId = new PublicKey('8fSeRtFseNrjdf8quE2YELhuzLkHV7WEGRPA9Jz8xEVe');

describe('Redeem Logic Vanilla Option Plugin', () => {

    const provider = AnchorProvider.env();
    const redeemLogicVanillaPlugin = RedeemLogicVanillaOptionPlugin.create(provider,redeemLogicVanillaPluginId);

    it('initialize and fetch the redeem logic vanilla configuration', async () => {
        const isCall = true;
        const isLinear = true;
        const strike = 2;

        await redeemLogicVanillaPlugin.initialize(isCall,isLinear,strike);  
        const redeemState = await redeemLogicVanillaPlugin.getRedeemLogicState();
        
        expect(redeemState.isCall).to.eql(isCall);
        expect(redeemState.isLinear).to.eql(isLinear);
        expect(redeemState.strike).to.eql(strike);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());

    })
}); 