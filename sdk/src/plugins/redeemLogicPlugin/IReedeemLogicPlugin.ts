import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../target/types/redeem_logic_lending";
import { RedeemLogicVanillaOption } from "../../../../target/types/redeem_logic_vanilla_option";
import { RedeemLogicLendingFee } from "../../../../target/types/redeem_logic_lending_fee";
import { RedeemLogicFarming } from "../../../../target/types/redeem_logic_farming";
import { RedeemLogicLendingState } from "./redeemLogicLending/RedeemLogicLendingState";
import { RedeemLogicVanillaOptionState } from "./redeemLogicVanillaOption/RedeemLogicVanillaOptionState";
import { RedeemLogicLendingFeeState } from "./redeemLogicLendingFee/RedeemLogicLendingFeeState";
import { RedeemLogicFarmingState } from "./redeemLogicFarming/RedeemLogicFarmingState";

export interface IRedeemLogicPlugin {
    program: anchor.Program<RedeemLogicLendingFee> | anchor.Program<RedeemLogicLending> | anchor.Program<RedeemLogicVanillaOption> | anchor.Program<RedeemLogicFarming>,
    provider: anchor.Provider;
    redeemLogicStateId: PublicKey;

    getProgramId(): PublicKey;
    getRedeemLogicState(redeemLogicStateId?: PublicKey): Promise<RedeemLogicLendingState | RedeemLogicVanillaOptionState | RedeemLogicLendingFeeState | RedeemLogicFarmingState>;
    initialize(...args): Promise<void>;
}
