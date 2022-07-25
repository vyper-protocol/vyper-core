import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../target/types/redeem_logic_lending";
import { RedeemLogicVanillaOption } from "../../../../target/types/redeem_logic_vanilla_option";
import { RedeemLogicLendingFee } from "../../../../target/types/redeem_logic_lending_fee";
import { RedeemLogicLendingState } from "./redeemLogicLending/RedeemLogicLendingState";
import { RedeemLogicVanillaOptionState } from "./redeemLogicVanillaOption/RedeemLogicVanillaOptionState";
import { RedeemLogicLendingFeeState } from "./redeemLogicLendingFee/RedeemLogicLendingFeeState";
export interface IRedeemLogicPlugin {
    program: anchor.Program<RedeemLogicLendingFee> | anchor.Program<RedeemLogicLending> | anchor.Program<RedeemLogicVanillaOption>,
    provider: anchor.Provider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey;
    getRedeemLogicState(redeemLogicStateId?: PublicKey): Promise<RedeemLogicLendingState | RedeemLogicVanillaOptionState | RedeemLogicLendingFeeState>;
    initialize(...args): Promise<void>;
}
