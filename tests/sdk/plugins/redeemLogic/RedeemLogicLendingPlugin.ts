import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { RedeemLogicLending } from "../../../../target/types/redeem_logic_lending";

export class RedeemLogicLendingPlugin {
    program: anchor.Program<RedeemLogicLending>;
    provider: anchor.AnchorProvider;
    state: PublicKey;

    get programID(): PublicKey {
        return this.program.programId;
    }

    static create(
        program: anchor.Program<RedeemLogicLending>,
        provider: anchor.AnchorProvider
    ): RedeemLogicLendingPlugin {
        const client = new RedeemLogicLendingPlugin();
        client.program = program;
        client.provider = provider;
        return client;
    }

    async initialize(interestSplit: number, fixedFeePerTranche: number = 0) {
        const redeemLogicProgramState = anchor.web3.Keypair.generate();
        await this.program.methods
            .initialize(interestSplit, new anchor.BN(fixedFeePerTranche))
            .accounts({
                redeemLogicConfig: redeemLogicProgramState.publicKey,
                owner: this.provider.wallet.publicKey,
                payer: this.provider.wallet.publicKey,
            })
            .signers([redeemLogicProgramState])
            .rpc();
        this.state = redeemLogicProgramState.publicKey;
    }
}
