import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateMock } from "../../../../target/types/rate_mock";

export class RateMockPlugin {
    program: anchor.Program<RateMock>;
    provider: anchor.AnchorProvider;
    state: PublicKey;

    get programID(): PublicKey {
        return this.program.programId;
    }

    static create(program: anchor.Program<RateMock>, provider: anchor.AnchorProvider): RateMockPlugin {
        const client = new RateMockPlugin();
        client.program = program;
        client.provider = provider;
        return client;
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
        this.state = rateState.publicKey;
    }

    async setFairValue(fairValue: number) {
        await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.state,
                authority: this.provider.wallet.publicKey,
            })
            .rpc();
    }

    async getSetFairValueIX(fairValue: number): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .setFairValue(fairValue)
            .accounts({
                rateData: this.state,
                authority: this.provider.wallet.publicKey,
            })
            .instruction();
    }

    async getRefreshIX(): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .refresh()
            .accounts({
                rateData: this.state,
                authority: this.provider.wallet.publicKey,
            })
            .instruction();
    }
}
