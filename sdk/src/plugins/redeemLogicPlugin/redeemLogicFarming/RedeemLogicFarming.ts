import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicFarming } from "../../../../../target/types/redeem_logic_farming";
import idlRedeemLogicFarming from "../../../../../target/idl/redeem_logic_farming.json";
import { RedeemLogicFarmingState } from "./RedeemLogicFarmingState";
import { IRedeemLogicPlugin } from "../IReedeemLogicPlugin";

export class RedeemLogicFarmingPlugin implements IRedeemLogicPlugin {

    program: anchor.Program<RedeemLogicFarming>;
    provider: anchor.AnchorProvider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, redeemLogicStateId: PublicKey): RedeemLogicFarmingPlugin {
        const client = new RedeemLogicFarmingPlugin();
        const program = new anchor.Program(idlRedeemLogicFarming as any, redeemLogicStateId, provider) as anchor.Program<RedeemLogicLending>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getRedeemLogicState(redeemLogicStateId?: PublicKey) {

        if (!redeemLogicStateId) {
            redeemLogicStateId = this.redeemLogicStateId;
        }
        
        const redeemLogicLendingState = await this.program.account.redeemLogicConfig.fetch(redeemLogicStateId);
        const redeemLogicState = new RedeemLogicFarmingState(
            redeemLogicLendingState.interestSplit,
            redeemLogicLendingState.owner
        )
        return redeemLogicState;
    }

    async initialize(interestSplit: number) {
        const redeemLogicState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(interestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicState.publicKey,
                owner: this.provider.wallet.publicKey,
                payer: this.provider.wallet.publicKey,
            })
            .signers([redeemLogicState])
            .rpc();
        this.redeemLogicStateId = redeemLogicState.publicKey;
    }
    
}