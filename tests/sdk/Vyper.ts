import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { VyperCore } from "../../target/types/vyper_core";

export type InitializationData = {
    trancheMintDecimals: number;
    haltFlags: number;
    ownerRestrictedIxs: number;
};

export class Vyper {
    program: anchor.Program<VyperCore>;
    provider: anchor.AnchorProvider;

    juniorTrancheMint: PublicKey;
    seniorTrancheMint: PublicKey;
    trancheConfig: PublicKey;
    trancheAuthority: PublicKey;
    reserveMint: PublicKey;
    reserve: PublicKey;
    ratePlugin: PublicKey;
    ratePluginState: PublicKey;
    redeemLogicPlugin: PublicKey;
    redeemLogicPluginState: PublicKey;

    static create(program: anchor.Program<VyperCore>, provider: anchor.AnchorProvider): Vyper {
        const client = new Vyper();
        client.program = program;
        client.provider = provider;
        return client;
    }

    async initialize(
        initData: InitializationData,
        reserveMint: PublicKey,
        ratePlugin: PublicKey,
        ratePluginState: PublicKey,
        redeemLogicPlugin: PublicKey,
        redeemLogicPluginState: PublicKey
    ) {
        const juniorTrancheMint = anchor.web3.Keypair.generate();
        const seniorTrancheMint = anchor.web3.Keypair.generate();
        const trancheConfig = anchor.web3.Keypair.generate();
        const [trancheAuthority] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), anchor.utils.bytes.utf8.encode("authority")],
            this.program.programId
        );
        const [reserve] = await anchor.web3.PublicKey.findProgramAddress(
            [trancheConfig.publicKey.toBuffer(), reserveMint.toBuffer()],
            this.program.programId
        );

        await this.program.methods
            .initialize(initData)
            .accounts({
                payer: this.provider.wallet.publicKey,
                owner: this.provider.wallet.publicKey,
                trancheConfig: trancheConfig.publicKey,
                trancheAuthority,
                rateProgram: ratePlugin,
                rateProgramState: ratePluginState,
                redeemLogicProgram: redeemLogicPlugin,
                redeemLogicProgramState: redeemLogicPluginState,
                reserveMint,
                reserve,
                juniorTrancheMint: juniorTrancheMint.publicKey,
                seniorTrancheMint: seniorTrancheMint.publicKey,
            })
            .signers([juniorTrancheMint, seniorTrancheMint, trancheConfig])
            .rpc();

        this.seniorTrancheMint = seniorTrancheMint.publicKey;
        this.juniorTrancheMint = juniorTrancheMint.publicKey;
        this.trancheConfig = trancheConfig.publicKey;
        this.trancheAuthority = trancheAuthority;
        this.reserveMint = reserveMint;
        this.reserve = reserve;
        this.ratePlugin = ratePlugin;
        this.ratePluginState = ratePluginState;
        this.redeemLogicPlugin = redeemLogicPlugin;
        this.redeemLogicPluginState = redeemLogicPluginState;
    }

    async getRefreshTrancheFairValueIX(): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .refreshTrancheFairValue()
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: this.trancheConfig,
                seniorTrancheMint: this.seniorTrancheMint,
                juniorTrancheMint: this.juniorTrancheMint,
                rateProgramState: this.ratePluginState,
                redeemLogicProgram: this.redeemLogicPlugin,
                redeemLogicProgramState: this.redeemLogicPluginState,
            })
            .instruction();
    }

    async refreshTrancheFairValue() {
        await this.program.methods
            .refreshTrancheFairValue()
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: this.trancheConfig,
                seniorTrancheMint: this.seniorTrancheMint,
                juniorTrancheMint: this.juniorTrancheMint,
                rateProgramState: this.ratePluginState,
                redeemLogicProgram: this.redeemLogicPlugin,
                redeemLogicProgramState: this.redeemLogicPluginState,
            })
            .rpc();
    }

    async getDepositIx(
        seniorDepositAmount: number,
        juniorDepositAmount: number,
        userReserveToken: PublicKey,
        userSeniorTrancheTokenAccount: PublicKey,
        userJuniorTrancheTokenAccount: PublicKey
    ): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .deposit({
                reserveQuantity: [new anchor.BN(seniorDepositAmount), new anchor.BN(juniorDepositAmount)],
            })
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: this.trancheConfig,
                trancheAuthority: this.trancheAuthority,
                reserve: this.reserve,
                userReserveToken,
                seniorTrancheMint: this.seniorTrancheMint,
                juniorTrancheMint: this.juniorTrancheMint,
                seniorTrancheDest: userSeniorTrancheTokenAccount,
                juniorTrancheDest: userJuniorTrancheTokenAccount,
            })
            .instruction();
    }

    async getRedeemIx(
        seniorDepositAmount: number,
        juniorDepositAmount: number,
        userReserveToken: anchor.web3.PublicKey,
        seniorTrancheTokenAccount: anchor.web3.PublicKey,
        juniorTrancheTokenAccount: anchor.web3.PublicKey
    ): Promise<anchor.web3.TransactionInstruction> {
        return await this.program.methods
            .redeem({
                trancheQuantity: [new anchor.BN(seniorDepositAmount), new anchor.BN(juniorDepositAmount)],
            })
            .accounts({
                signer: this.provider.wallet.publicKey,
                trancheConfig: this.trancheConfig,
                trancheAuthority: this.trancheAuthority,
                reserve: this.reserve,
                userReserveToken,
                seniorTrancheMint: this.seniorTrancheMint,
                juniorTrancheMint: this.juniorTrancheMint,
                seniorTrancheSource: seniorTrancheTokenAccount,
                juniorTrancheSource: juniorTrancheTokenAccount,
            })
            .instruction();
    }
}
