import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateMock } from "../../../../../target/types/rate_mock";
import idlRateMock from "../../../../../target/idl/rate_mock.json";
import { RateState } from "./RateMockState";
import { IRateMockPlugin } from "../IRatePlugin";

export class RateMockPlugin implements IRateMockPlugin {

    program: anchor.Program<RateMock>;
    provider: anchor.AnchorProvider;
    rateMockStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, rateMockId: PublicKey): RateMockPlugin {
        const client = new RateMockPlugin();
        const program = new anchor.Program(idlRateMock as any, rateMockId, provider) as anchor.Program<RateMock>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getRateMockPluginState(rateMockStateId?: PublicKey) {

        if (!rateMockStateId) {
            rateMockStateId = this.rateMockStateId;
        }
        const rateMockState = await this.program.account.rateState.fetch(rateMockStateId);
        const rateState = new RateState(
            rateMockState.fairValue,
            rateMockState.refreshedSlot.toNumber(),
        )
        return rateState;
    }

    async setFairValue(fairValue: number) {
        await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.rateMockStateId,
                signer: this.provider.wallet.publicKey,
            })
            .rpc();
    }

    async getSetFairValueIX(fairValue: number): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.rateMockStateId,
                signer: this.provider.wallet.publicKey,
            })
            .instruction();
    }
}