import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicVanillaOption } from "../../../../target/types/redeem_logic_vanilla_option";

export class RedeemLogicVanillaOptionPlugin {
    program: anchor.Program<RedeemLogicVanillaOption>;
    provider: anchor.AnchorProvider;
    state: PublicKey;

    get programID(): PublicKey {
        return this.program.programId;
    }

    static create(
        program: anchor.Program<RedeemLogicVanillaOption>,
        provider: anchor.AnchorProvider
    ): RedeemLogicVanillaOptionPlugin {
        const client = new RedeemLogicVanillaOptionPlugin();
        client.program = program;
        client.provider = provider;
        return client;
    }

    async initialize(strike: number, notional: number, isCall: boolean, isLinear: boolean) {
        const redeemLogicProgramState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(strike, new anchor.BN(notional), isCall, isLinear)
            .accounts({
                redeemLogicConfig: redeemLogicProgramState.publicKey,
                payer: this.provider.wallet.publicKey,
            })
            .signers([redeemLogicProgramState])
            .rpc();
        this.state = redeemLogicProgramState.publicKey;
    }
}
