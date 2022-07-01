import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../../target/types/redeem_logic_lending";
import idlRedeemLogicLending from "../../../../../target/idl/redeem_logic_lending.json";
import { RedeemLogicLendingState } from "./RedeemLogicLendingState";
import { IRedeemLogicLendingPlugin } from "../IReedeemLogicPlugin";

export class RedeemLogicLendingPlugin implements IRedeemLogicLendingPlugin {

    program: anchor.Program<RedeemLogicLending>;
    provider: anchor.AnchorProvider;
    redeemLendingStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, redeemLogicLendingId: PublicKey): RedeemLogicLendingPlugin {
        const client = new RedeemLogicLendingPlugin();
        const program = new anchor.Program(idlRedeemLogicLending as any, redeemLogicLendingId, provider) as anchor.Program<RedeemLogicLending>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getRedeemLogicLendingState(redeemLogicLendingStateId?: PublicKey) {

        if (!redeemLogicLendingStateId) {
            redeemLogicLendingStateId = this.redeemLendingStateId;
        }
        
        const redeemLogicLendingState = await this.program.account.redeemLogicConfig.fetch(redeemLogicLendingStateId);
        const redeemLogicState = new RedeemLogicLendingState(
            redeemLogicLendingState.interestSplit,
            redeemLogicLendingState.fixedFeePerTranche.toNumber(),
            redeemLogicLendingState.owner
        )
        return redeemLogicState;
    }

    async initialize(interestSplit: number, fixedFeePerTranche: number = 0) {
        const redeemLogicState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(interestSplit, new anchor.BN(fixedFeePerTranche))
            .accounts({
                redeemLogicConfig: redeemLogicState.publicKey,
                owner: this.provider.wallet.publicKey,
                payer: this.provider.wallet.publicKey,
            })
            .signers([redeemLogicState])
            .rpc();
        this.redeemLendingStateId = redeemLogicState.publicKey;
    }
    
}