import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createMintToInstruction } from "@solana/spl-token";
import { Keypair, Transaction } from "@solana/web3.js";
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";
import { expect } from "chai";
import { RatePoolv2 } from "../target/types/rate_poolv2";
import { createMint, createTokenAccount } from "./utils";

describe("rate_poolv2", () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const program = anchor.workspace.RatePoolv2 as Program<RatePoolv2>;

    it("initialize", async () => {
        const baseMint = 9;
        const quoteDecimals = 6;
        const lpDecimals = 6;
        const baseAmount = 74_792 * 10 ** baseMint;
        const quoteAmount = 2_394_354 * 10 ** quoteDecimals;
        const lpAmount = 4_142_365 * 10 ** lpDecimals;
        const poolConfig = await createPoolConfig(
            provider,
            baseAmount,
            quoteAmount,
            lpAmount,
            baseMint,
            quoteDecimals,
            lpDecimals
        );

        const rateData = anchor.web3.Keypair.generate();
        const sig = await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
                pool: poolConfig.poolId.publicKey,
                lpMint: poolConfig.mints.lpMint,
                baseMint: poolConfig.mints.baseMint,
                quoteMint: poolConfig.mints.quoteMint,
                baseTokenAccount: poolConfig.ata.baseATA,
                quoteTokenAccount: poolConfig.ata.quoteATA,
            })
            .signers([rateData])
            .rpc();
        console.log("init sig: " + sig);

        const rateDataAccount = await program.account.rateState.fetch(rateData.publicKey);
        expect(rateDataAccount.baseTokenAccount).to.be.eql(poolConfig.ata.baseATA);
        expect(rateDataAccount.quoteTokenAccount).to.be.eql(poolConfig.ata.quoteATA);
        expect(rateDataAccount.lpMint).to.be.eql(poolConfig.mints.lpMint);
        expect(rateDataAccount.refreshedSlot.toNumber()).to.be.gt(0);
        expect(new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[0])).toNumber()).to.be.closeTo(
            1.1560323631,
            0.000000001
        );
        expect(new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[1])).toNumber()).to.be.closeTo(
            32.013504118,
            0.00000001
        );
    });

    it("refresh", async () => {
        const baseMint = 9;
        const quoteDecimals = 6;
        const lpDecimals = 6;
        const baseAmount = 10 * 10 ** baseMint;
        const quoteAmount = 10 * 10 ** quoteDecimals;
        const lpAmount = 100 * 10 ** lpDecimals;
        const poolConfig = await createPoolConfig(
            provider,
            baseAmount,
            quoteAmount,
            lpAmount,
            baseMint,
            quoteDecimals,
            lpDecimals
        );

        const rateData = anchor.web3.Keypair.generate();
        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                pool: poolConfig.poolId.publicKey,
                rateData: rateData.publicKey,
                lpMint: poolConfig.mints.lpMint,
                baseMint: poolConfig.mints.baseMint,
                quoteMint: poolConfig.mints.quoteMint,
                baseTokenAccount: poolConfig.ata.baseATA,
                quoteTokenAccount: poolConfig.ata.quoteATA,
            })
            .signers([rateData])
            .rpc();

        let rateDataAccount = await program.account.rateState.fetch(rateData.publicKey);
        const initialLpPrice = new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[0])).toNumber();
        // mint some new lp tokens
        const tx = new Transaction();
        tx.add(
            createMintToInstruction(
                poolConfig.mints.lpMint,
                poolConfig.ata.lpATA,
                poolConfig.poolId.publicKey,
                lpAmount,
                [poolConfig.poolId]
            )
        );
        await provider.sendAndConfirm(tx, [poolConfig.poolId]);

        // refresh prices
        const refreshSig = await program.methods
            .refresh()
            .accounts({
                rateData: rateData.publicKey,
                lpMint: poolConfig.mints.lpMint,
                baseMint: poolConfig.mints.baseMint,
                quoteMint: poolConfig.mints.quoteMint,
                baseTokenAccount: poolConfig.ata.baseATA,
                quoteTokenAccount: poolConfig.ata.quoteATA,
            })
            .rpc();
        console.log("refreshSig: ", refreshSig);

        rateDataAccount = await program.account.rateState.fetch(rateData.publicKey);

        const finalLpPrice = new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[0])).toNumber();
        expect(finalLpPrice).to.be.eq(initialLpPrice / 2);
    });
});

async function createPoolConfig(
    provider: anchor.AnchorProvider,
    baseAmount: number,
    quoteAmount: number,
    lpAmount: number,
    baseDecimals: number,
    quoteDecimals: number,
    lpDecimals: number
) {
    const mintAuthority = anchor.web3.Keypair.generate();
    const poolId = anchor.web3.Keypair.generate();

    const baseMint = await createMint(provider, baseDecimals, mintAuthority.publicKey);
    const quoteMint = await createMint(provider, quoteDecimals, mintAuthority.publicKey);
    const lpMint = await createMint(provider, lpDecimals, poolId.publicKey);

    const baseATA = await createTokenAccount(provider, baseMint, poolId.publicKey);
    const quoteATA = await createTokenAccount(provider, quoteMint, poolId.publicKey);
    const lpATA = await createTokenAccount(provider, lpMint, Keypair.generate().publicKey);

    // mint some tokens
    const tx = new Transaction();
    tx.add(createMintToInstruction(baseMint, baseATA, mintAuthority.publicKey, baseAmount, [mintAuthority]));
    tx.add(createMintToInstruction(quoteMint, quoteATA, mintAuthority.publicKey, quoteAmount, [mintAuthority]));
    tx.add(createMintToInstruction(lpMint, lpATA, poolId.publicKey, lpAmount, [poolId]));
    await provider.sendAndConfirm(tx, [mintAuthority, poolId]);

    return {
        poolId,
        mints: {
            baseMint,
            quoteMint,
            lpMint,
        },
        ata: {
            baseATA,
            quoteATA,
            lpATA,
        },
    };
}
