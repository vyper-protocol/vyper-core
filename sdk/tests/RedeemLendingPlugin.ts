import { AnchorProvider } from '@project-serum/anchor';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { expect } from "chai";
import { RedeemLogicLendingPlugin } from '../src/plugins/redeemLogicPlugin/redeemLogicLending/RedeemLogicLending';

dotenv.config();

const redeemLogicLendingPluginId = new PublicKey('Gc2ZKNuCpdNKhAzEGS2G9rBSiz4z8MULuC3M3t8EqdWA');

describe('Redeem Lending Plugin', () => {

    const provider = AnchorProvider.env();
    const redeemLogicLendingPlugin = RedeemLogicLendingPlugin.create(provider,redeemLogicLendingPluginId);
    
    it('fetch existing redeem logic configuration', async () => {

        const accounts = await provider.connection.getProgramAccounts(redeemLogicLendingPluginId);
        const redeemState = await redeemLogicLendingPlugin.getRedeemLogicLendingState(accounts[0].pubkey);
        
        expect(redeemState.interestSplit != undefined);
        expect(redeemState.fixedFeePerTranche != undefined);
        expect(redeemState.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58())
    })

}); 