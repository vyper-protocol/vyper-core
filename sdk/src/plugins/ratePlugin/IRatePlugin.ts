import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateMock } from "../../../../../target/types/rate_mock";
import { RateSwitchboard } from "../../../../target/types/rate_switchboard";
import { RateState } from "./rateMock/RateState";
import {RateState as RateSwitchboardState} from "./rateSwitchboard/RateState"

export interface IRatePlugin {
    program: anchor.Program<RateMock> | anchor.Program<RateSwitchboard>;
    provider: anchor.AnchorProvider;
    rateStateId: PublicKey;
    getProgramId(): PublicKey;
    getRatePluginState(rateStateId?: PublicKey): Promise<RateState |  RateSwitchboardState>;
    initialize(): Promise<void>;
    getRefreshIX(): Promise<anchor.web3.TransactionInstruction>
}