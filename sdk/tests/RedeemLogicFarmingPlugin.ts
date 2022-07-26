import * as anchor from '@project-serum/anchor'
import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { expect } from "chai";
import { RedeemLogicFarmingPlugin } from '../src/plugins/redeemLogicPlugin/redeemLogicFarming/RedeemLogicFarming';

dotenv.config();

const redeemLogicFarmingPluginId = new PublicKey('Fd87TGcYmWs1Gfa7XXZycJwt9kXjRs8axMtxCWtCmowN');

describe('Redeem Logic Farming Plugin', () => {

    const provider = AnchorProvider.env();
    const redeemLogicFarmingPlugin = RedeemLogicFarmingPlugin.create(provider,redeemLogicFarmingPluginId);
    
    it('initialize and fetch existing redeem logic configuration', async () => {
        const interestSplit = 5000;
        
        await redeemLogicFarmingPlugin.initialize(interestSplit);  
        const redeemState = await redeemLogicFarmingPlugin.getRedeemLogicState();

        expect(redeemState.interestSplit).to.eql(interestSplit);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
    })
}); 