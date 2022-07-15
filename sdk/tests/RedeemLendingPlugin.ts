import * as anchor from '@project-serum/anchor'
import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { expect } from "chai";
import { RedeemLogicLendingPlugin } from '../src/plugins/redeemLogicPlugin/redeemLogicLending/RedeemLogicLending';

dotenv.config();

const redeemLogicLendingPluginId = new PublicKey('Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA');

describe('Redeem Logic Lending Plugin', () => {

    const provider = AnchorProvider.env();
    const redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
    
    it('initialize', async () => {
        const interestSplit = 5000;
        await redeemLogicLendingPlugin.initialize(interestSplit);  
        const redeemState = await redeemLogicLendingPlugin.getRedeemLogicState();

        expect(redeemState.interestSplit).to.eql(interestSplit);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
    })


    it('fetch existing redeem logic configuration', async () => {
        const interestSplit = 5000;
        await redeemLogicLendingPlugin.initialize(interestSplit); 
        const redeemState = await redeemLogicLendingPlugin.getRedeemLogicState();
        
        expect(redeemState.interestSplit != undefined);
        expect(redeemState.fixedFeePerTranche != undefined);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58())
    })

    

}); 