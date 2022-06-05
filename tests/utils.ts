import * as anchor from "@project-serum/anchor";
import { createInitializeMintInstruction, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export async function createMint(provider: anchor.AnchorProvider, decimals: number = 6, authority?: anchor.web3.PublicKey) {
    if (authority === undefined) {
        authority = anchor.web3.Keypair.generate().publicKey;
    }
    const mint = anchor.web3.Keypair.generate();
    const instructions = await createMintInstructions(provider, mint.publicKey, decimals, authority);

    const tx = new anchor.web3.Transaction();
    tx.add(...instructions);

    await provider.sendAndConfirm(tx, [mint]);

    return mint.publicKey;
}

export async function createMintInstructions(
    provider: anchor.AnchorProvider,
    mint: anchor.web3.PublicKey,
    decimals: number,
    authority: anchor.web3.PublicKey
) {
    return [
        anchor.web3.SystemProgram.createAccount({
            fromPubkey: provider.wallet.publicKey,
            newAccountPubkey: mint,
            space: 82,
            lamports: await provider.connection.getMinimumBalanceForRentExemption(82),
            programId: TOKEN_PROGRAM_ID,
        }),
        createInitializeMintInstruction(mint, decimals, authority, null),
    ];
}

export function getInitializeData(trancheMintDecimals: number) {
    return {
        trancheMintDecimals,
    };
}

export const UPDATE_TRANCHE_CONFIG_FLAGS = {
    HALT_FLAGS: 1 << 0,
};

export const TRANCHE_HALT_FLAGS = {
    HALT_DEPOSITS: 1 << 0,
    HALT_REFRESHES: 1 << 1,
    HALT_REDEEMS: 1 << 2,
};
export const TRANCHE_HALT_FLAGS_HALT_ALL =
    TRANCHE_HALT_FLAGS.HALT_DEPOSITS | TRANCHE_HALT_FLAGS.HALT_REFRESHES | TRANCHE_HALT_FLAGS.HALT_REDEEMS;
