import * as anchor from '@project-serum/anchor'
import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { expect } from "chai";
import { RedeemLogicLendingFeePlugin } from '../src/plugins/redeemLogicPlugin/redeemLogicLendingFee/RedeemLogicLendingFee';

dotenv.config();

const redeemLogicLendingPluginId = new PublicKey('3mq416it8YJsd5DKNuWeoCCAH8GYJfpuefHSNkSP6LyS');

describe('Redeem Logic Lending Fee Plugin', () => {

    const provider = AnchorProvider.env();
    const redeemLogicLendingPlugin = RedeemLogicLendingFeePlugin.create(provider,redeemLogicLendingPluginId);
    
    it('initialize and fetch existing redeem logic configuration', async () => {
        const interestSplit = 5000;
        const mgmtFee = 5;
        const perfFee = 5;
        await redeemLogicLendingPlugin.initialize(interestSplit,mgmtFee,perfFee);  
        const redeemState = await redeemLogicLendingPlugin.getRedeemLogicState();

        expect(redeemState.interestSplit).to.eql(interestSplit);
        expect(redeemState.mgmtFee).to.eql(mgmtFee);
        expect(redeemState.perfFee).to.eql(perfFee);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
    })
}); 