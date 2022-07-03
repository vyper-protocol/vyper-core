import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateMock } from "../../../../../target/types/rate_mock";
import { RateState } from "./rateMock/RateMockState";

export interface IRateMockPlugin {
    program: anchor.Program<RateMock>;
    provider: anchor.AnchorProvider;
    rateMockStateId: PublicKey;
    getProgramId(): PublicKey;
    getRateMockPluginState(rateMockStateId?: PublicKey): Promise<RateState>;
    setFairValue(fairValue: number): Promise<void>;
    getSetFairValueIX(fairValue: number): Promise<anchor.web3.TransactionInstruction>;
    initialize(): Promise<void>;
}