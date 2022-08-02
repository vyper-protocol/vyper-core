import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RateSwitchboard } from "../../../../../target/types/rate_switchboard";
import idlRateSwitchboard from "../../../../../target/idl/rate_switchboard.json";
import { RateState } from "./RateState";
import { IRatePlugin } from "../IRatePlugin";
import { SWITCHBOARD_AGGREGATORS } from "./SwitchboardAggregators"


export class RateSwitchboardPlugin implements IRatePlugin {

    program: anchor.Program<RateSwitchboard>;
    provider: anchor.AnchorProvider;
    rateStateId: PublicKey;

    getProgramId(): PublicKey {
        return this.program.programId;
    }

    static create(provider: anchor.AnchorProvider, ratePluginId: PublicKey): RateSwitchboardPlugin {
        const client = new RateSwitchboardPlugin();
        const program = new anchor.Program(idlRateSwitchboard as any, ratePluginId, provider) as anchor.Program<RateSwitchboard>;
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
            ratePluginState.switchboardAggregators as PublicKey[]
        )
        return rateState;
    }

    async getRefreshIX(): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .refresh()
            .accounts({
                rateData: this.rateStateId,
            })
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .instruction()
    }

    async initialize() {
        const rateState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize()
            .accounts({
                signer: this.provider.wallet.publicKey,
                rateData: rateState.publicKey,
            })
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateState])
            .rpc();
        this.rateStateId = rateState.publicKey;
    }


}