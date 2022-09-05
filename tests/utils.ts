import * as anchor from "@project-serum/anchor";
import {
    createAccount,
    createAssociatedTokenAccount,
    createAssociatedTokenAccountInstruction,
    createInitializeAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    getAccount,
    getAssociatedTokenAddress,
    getMint,
    TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export async function createTokenAccount(
    provider: anchor.AnchorProvider,
    mint: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey
) {
    const tx = new anchor.web3.Transaction();

    const aToken = await getAssociatedTokenAddress(mint, owner);
    tx.add(createAssociatedTokenAccountInstruction(provider.wallet.publicKey, aToken, owner, mint));
    const signature = await provider.sendAndConfirm(tx);
    // console.log("createTokenAccount signature: ", signature);

    return aToken;
}

export async function createMintAndVault(provider: anchor.AnchorProvider, amount: number, decimals: number = 6) {
    const mint = anchor.web3.Keypair.generate();
    const authority = anchor.web3.Keypair.generate();

    const createMintIx = await createMintInstructions(provider, mint.publicKey, decimals, authority.publicKey);
    const aToken = await getAssociatedTokenAddress(mint.publicKey, provider.wallet.publicKey);

    const aTokenCreationIx = createAssociatedTokenAccountInstruction(
        provider.wallet.publicKey,
        aToken,
        provider.wallet.publicKey,
        mint.publicKey
    );
    const mintToIx = createMintToInstruction(mint.publicKey, aToken, authority.publicKey, amount);

    const tx = new anchor.web3.Transaction();
    tx.add(...createMintIx);
    tx.add(aTokenCreationIx);
    tx.add(mintToIx);

    const signature = await provider.sendAndConfirm(tx, [mint, authority]);

    return [mint.publicKey, aToken];
}

export async function createMint(
    provider: anchor.AnchorProvider,
    decimals: number = 6,
    authority?: anchor.web3.PublicKey
) {
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

export function bn(v: number): anchor.BN {
    return new anchor.BN(v);
}

export function getInitializeData(trancheMintDecimals: number) {
    return {
        trancheMintDecimals,
    };
}

export async function getTokenAccountAmount(
    provider: anchor.AnchorProvider,
    tokenAccount: anchor.web3.PublicKey
): Promise<number> {
    return Number((await getAccount(provider.connection, tokenAccount, undefined, TOKEN_PROGRAM_ID)).amount);
}

export const UPDATE_TRANCHE_CONFIG_FLAGS = {
    HALT_FLAGS: 1 << 0,
    OWNER_RESTRICTED_IXS: 1 << 1,
    RESERVE_FAIR_VALUE_STALE_SLOT_THRESHOLD: 1 << 2,
    TRANCHE_FAIR_VALUE_STALE_SLOT_THRESHOLD: 1 << 3,
    DEPOSIT_CAP: 1 << 4,
};

export const TRANCHE_HALT_FLAGS = {
    NONE: 0,
    HALT_DEPOSITS: 1 << 0,
    HALT_REFRESHES: 1 << 1,
    HALT_REDEEMS: 1 << 2,
};

export const TRANCHE_HALT_FLAGS_HALT_ALL =
    TRANCHE_HALT_FLAGS.HALT_DEPOSITS | TRANCHE_HALT_FLAGS.HALT_REFRESHES | TRANCHE_HALT_FLAGS.HALT_REDEEMS;

export const OWNER_RESTRICTED_IX_FLAGS = {
    NONE: 0,
    DEPOSITS: 1 << 0,
    REFRESHES: 1 << 1,
    REDEEMS: 1 << 2,
};

export const OWNER_RESTRICTED_IX_FLAGS_ALL =
    OWNER_RESTRICTED_IX_FLAGS.DEPOSITS | OWNER_RESTRICTED_IX_FLAGS.REDEEMS | OWNER_RESTRICTED_IX_FLAGS.REFRESHES;
