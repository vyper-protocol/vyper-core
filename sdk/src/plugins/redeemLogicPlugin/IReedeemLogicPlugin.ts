import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../target/types/redeem_logic_lending";
import { RedeemLogicLendingState } from "./redeemLogicLending/RedeemLogicLendingState";


export interface IRedeemLogicLendingPlugin {
    program: anchor.Program<RedeemLogicLending>,
    provider: anchor.Provider;
    redeemLendingStateId: PublicKey;

    getProgramId(): PublicKey;
    getRedeemLogicLendingState(redeemLogicLendingStateId?: PublicKey): Promise<RedeemLogicLendingState>;
    initialize(interestSplit: number, fixedFeePerTranche: number): Promise<void>;
}