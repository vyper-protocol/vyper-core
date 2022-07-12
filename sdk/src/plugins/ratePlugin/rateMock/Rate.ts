import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateMock } from "../../../../../target/types/rate_mock";
import idlRateMock from "../../../../../target/idl/rate_mock.json";
import { RateState } from "./RateState";
import { IRatePlugin } from "../IRatePlugin";

export class RatePlugin implements IRatePlugin {

    program: anchor.Program<RateMock>;
    provider: anchor.AnchorProvider;
    rateStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, ratePluginId: PublicKey): RatePlugin {
        const client = new RatePlugin();
        const program = new anchor.Program(idlRateMock as any, ratePluginId, provider) as anchor.Program<RateMock>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getRatePluginState(rateStateId?: PublicKey) {

        if (!rateStateId) {
            rateStateId = this.rateStateId;
        }
        const ratePluginState = await this.program.account.rateState.fetch(rateStateId);
        const rateState = new RateState(
            ratePluginState.fairValue,
            ratePluginState.refreshedSlot.toNumber(),
        )
        return rateState;
    }

    async setFairValue(fairValue: number) {
        await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.rateStateId,
                signer: this.provider.wallet.publicKey,
            })
            .rpc();
    }

    async getSetFairValueIX(fairValue: number): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.rateStateId,
                signer: this.provider.wallet.publicKey,
            })
            .instruction();
    }

    async getRefreshIX(): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
        .refresh()
        .accounts({
            rateData: this.rateStateId,
            signer: this.provider.wallet.publicKey,
        })
        .instruction();
    }

    async initialize() {
        const rateState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize()
            .accounts({
                rateData: rateState.publicKey,
                authority: this.provider.wallet.publicKey,
                signer: this.provider.wallet.publicKey,
            })
            .signers([rateState])
            .rpc();
        this.rateStateId = rateState.publicKey;
    }


}