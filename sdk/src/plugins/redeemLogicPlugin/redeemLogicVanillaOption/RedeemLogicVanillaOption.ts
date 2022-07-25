import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicVanillaOption } from "../../../../../target/types/redeem_logic_vanilla_option";
import idlRedeemLogicVanilla from "../../../../../target/idl/redeem_logic_vanilla_option.json";
import { RedeemLogicVanillaOptionState } from "./RedeemLogicVanillaOptionState";
import { IRedeemLogicPlugin } from "../IReedeemLogicPlugin";

export class RedeemLogicVanillaOptionPlugin implements IRedeemLogicPlugin {

    program: anchor.Program<RedeemLogicVanillaOption>;
    provider: anchor.AnchorProvider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, redeemLogicStateId: PublicKey): RedeemLogicVanillaOptionPlugin {
        const client = new RedeemLogicVanillaOptionPlugin();
        const program = new anchor.Program(idlRedeemLogicVanilla as any, redeemLogicStateId, provider) as anchor.Program<RedeemLogicVanillaOption>;
        client.program = program;
        client.provider = provider;
        return client;
    }


    async getRedeemLogicState(redeemLogicStateId?: PublicKey) {

        if (!redeemLogicStateId) {
            redeemLogicStateId = this.redeemLogicStateId;
        }
        
        const redeemVanillaState = await this.program.account.redeemLogicConfig.fetch(redeemLogicStateId);
        const redeemLogicState = new RedeemLogicVanillaOptionState(
            redeemVanillaState.isCall,
            redeemVanillaState.isLinear,
            redeemVanillaState.strike,
            redeemVanillaState.owner
        )
        return redeemLogicState;
    }

    async initialize(isCall: boolean, isLinear: boolean, strike: number) {
        const redeemVanillaState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(strike,isCall,isLinear)
            .accounts({
                redeemLogicConfig: redeemVanillaState.publicKey,
                owner: this.provider.wallet.publicKey,
                payer: this.provider.wallet.publicKey,
            })
            .signers([redeemVanillaState])
            .rpc();
        this.redeemLogicStateId = redeemVanillaState.publicKey;
    }
    
}