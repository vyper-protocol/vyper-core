import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLendingFee } from "../../../../../target/types/redeem_logic_lending_fee";
import idlRedeemLogicLendingFee from "../../../../../target/idl/redeem_logic_lending_fee.json";
import { RedeemLogicLendingFeeState } from "./RedeemLogicLendingFeeState";
import { IRedeemLogicPlugin } from "../IReedeemLogicPlugin";

export class RedeemLogicLendingFeePlugin implements IRedeemLogicPlugin {

    program: anchor.Program<RedeemLogicLendingFee>;
    provider: anchor.AnchorProvider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, redeemLogicStateId: PublicKey): RedeemLogicLendingFeePlugin {
        const client = new RedeemLogicLendingFeePlugin();
        const program = new anchor.Program(idlRedeemLogicLendingFee as any, redeemLogicStateId, provider) as anchor.Program<RedeemLogicLendingFee>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getRedeemLogicState(redeemLogicStateId?: PublicKey) {

        if (!redeemLogicStateId) {
            redeemLogicStateId = this.redeemLogicStateId;
        }
        
        const redeemLogicLendingState = await this.program.account.redeemLogicConfig.fetch(redeemLogicStateId);
        const redeemLogicState = new RedeemLogicLendingFeeState(
            redeemLogicLendingState.interestSplit,
            redeemLogicLendingState.mgmtFee,
            redeemLogicLendingState.perfFee,
            redeemLogicLendingState.owner
        )
        return redeemLogicState;
    }

    async initialize(interestSplit: number, mgmtFee: number, perfFee: number) {
        const redeemLogicState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(interestSplit,mgmtFee,perfFee)
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