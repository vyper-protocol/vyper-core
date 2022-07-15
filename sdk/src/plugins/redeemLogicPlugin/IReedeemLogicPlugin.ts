import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../target/types/redeem_logic_lending";
import { RedeemLogicVanillaOption } from "../../../../target/types/redeem_logic_vanilla_option";
import { RedeemLogicLendingState } from "./redeemLogicLending/RedeemLogicLendingState";
import { RedeemLogicVanillaOptionState } from "./redeemLogicVanillaOption/RedeemLogicVanillaOptionState";
export interface IRedeemLogicPlugin {
    program: anchor.Program<RedeemLogicLending> | anchor.Program<RedeemLogicVanillaOption>,
    provider: anchor.Provider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey;
    getRedeemLogicState(redeemLogicStateId?: PublicKey): Promise<RedeemLogicLendingState | RedeemLogicVanillaOptionState>;
    initialize(...args): Promise<void>;
}
