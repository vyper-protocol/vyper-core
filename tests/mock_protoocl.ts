import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MockProtocol } from "../target/types/mock_protocol";
import assert from "assert";
import { bn, findAssociatedTokenAddress } from "./utils";

describe.only("mock_protocol", () => {
  const program = anchor.workspace.MockProtocol as Program<MockProtocol>;

  anchor.setProvider(anchor.Provider.env());

  async function createMint(): Promise<anchor.web3.PublicKey> {
    const mintKP = anchor.web3.Keypair.generate();
    const mint = mintKP.publicKey;

    const tx = new anchor.web3.Transaction();
    tx.add(
      anchor.web3.SystemProgram.createAccount({
        fromPubkey: program.provider.wallet.publicKey,
        newAccountPubkey: mint,
        space: MintLayout.span,
        lamports: await Token.getMinBalanceRentForExemptMint(program.provider.connection),
        programId: TOKEN_PROGRAM_ID,
      }),
      Token.createInitMintInstruction(
        TOKEN_PROGRAM_ID,
        mint,
        0,
        program.provider.wallet.publicKey,
        program.provider.wallet.publicKey
      )
    );
    await program.provider.send(tx, [mintKP]);

    return mint;
  }

  async function createUserAndTokenAccount(
    mint: anchor.web3.PublicKey,
    quantity: number
  ): Promise<[anchor.web3.Keypair, anchor.web3.PublicKey]> {
    const userKP = anchor.web3.Keypair.generate();
    const userTokenAccount = await findAssociatedTokenAddress(userKP.publicKey, mint);

    const tx = new anchor.web3.Transaction();
    tx.add(
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint,
        userTokenAccount,
        userKP.publicKey,
        program.provider.wallet.publicKey
      ),
      Token.createMintToInstruction(TOKEN_PROGRAM_ID, mint, userTokenAccount, program.provider.wallet.publicKey, [], quantity)
    );
    await program.provider.send(tx);

    return [userKP, userTokenAccount];
  }

  it("can initialize vault", async () => {
    const mint = await createMint();
    const [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
      program.programId
    );

    await program.rpc.initialize(vaultBump, {
      accounts: {
        vault,
        mint,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const mintInfo = await getMintInfo(program.provider, mint);
    const vaultAccountInfo = await getTokenAccount(program.provider, vault);

    assert.ok(mintInfo.mintAuthority.toBase58() == program.provider.wallet.publicKey.toBase58());
    assert.ok(mintInfo.isInitialized);
    assert.ok(mintInfo.supply.toNumber() == 0);
    assert.ok(vaultAccountInfo.isInitialized);
    assert.ok(vaultAccountInfo.mint.toBase58() == mint.toBase58());
  });

  it("authority can mint tokens to a new account", async () => {
    const mint = await createMint();
    const [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
      program.programId
    );

    await program.rpc.initialize(vaultBump, {
      accounts: {
        vault,
        mint,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const quantity = 1000;
    const userWallet = anchor.web3.Keypair.generate();
    const userTokenAccount = await findAssociatedTokenAddress(userWallet.publicKey, mint);

    const tx = new anchor.web3.Transaction();
    tx.add(
      Token.createAssociatedTokenAccountInstruction(
        ASSOCIATED_TOKEN_PROGRAM_ID,
        TOKEN_PROGRAM_ID,
        mint,
        userTokenAccount,
        userWallet.publicKey,
        program.provider.wallet.publicKey
      ),
      Token.createMintToInstruction(TOKEN_PROGRAM_ID, mint, userTokenAccount, program.provider.wallet.publicKey, [], quantity)
    );
    await program.provider.send(tx);

    const mintInfo = await getMintInfo(program.provider, mint);
    const userVaultAccountInfo = await getTokenAccount(program.provider, userTokenAccount);

    assert.ok(mintInfo.supply.toNumber() == quantity);
    assert.ok(userVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(userVaultAccountInfo.amount.toNumber() == quantity);
  });

  it("simulate interest", async () => {
    const mint = await createMint();
    const [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
      program.programId
    );

    await program.rpc.initialize(vaultBump, {
      accounts: {
        vault,
        mint,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const quantity = 1000;
    const [userKP, userTokenAccount] = await createUserAndTokenAccount(mint, quantity);

    await program.rpc.simulateInterest(bn(quantity), {
      accounts: {
        mint,
        vault,
        sourceAccount: userTokenAccount,
        sourceAccountAuthority: userKP.publicKey,
        // authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [userKP],
    });

    const mintInfo = await getMintInfo(program.provider, mint);
    const userVaultAccountInfo = await getTokenAccount(program.provider, userTokenAccount);
    const protocolVaultAccountInfo = await getTokenAccount(program.provider, vault);

    assert.ok(mintInfo.supply.toNumber() == quantity);
    assert.ok(userVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(userVaultAccountInfo.amount.toNumber() == 0);
    assert.ok(protocolVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(protocolVaultAccountInfo.amount.toNumber() == quantity);
  });

  it("simulate hack", async () => {
    const mint = await createMint();
    const [vault, vaultBump] = await anchor.web3.PublicKey.findProgramAddress(
      [Buffer.from(anchor.utils.bytes.utf8.encode("my-token-seed")), mint.toBuffer()],
      program.programId
    );

    await program.rpc.initialize(vaultBump, {
      accounts: {
        vault,
        mint,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const quantityInterest = 1000;
    const quantityHack = 500;
    const [userKP, userTokenAccount] = await createUserAndTokenAccount(mint, quantityInterest);

    await program.rpc.simulateInterest(bn(quantityInterest), {
      accounts: {
        mint,
        vault,
        sourceAccount: userTokenAccount,
        sourceAccountAuthority: userKP.publicKey,
        // authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
      signers: [userKP],
    });

    await program.rpc.simulateHack(bn(quantityHack), {
      accounts: {
        mint,
        vault,
        destAccount: userTokenAccount,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const mintInfo = await getMintInfo(program.provider, mint);
    const userVaultAccountInfo = await getTokenAccount(program.provider, userTokenAccount);
    const protocolVaultAccountInfo = await getTokenAccount(program.provider, vault);

    assert.ok(mintInfo.supply.toNumber() == quantityInterest);
    assert.ok(userVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(userVaultAccountInfo.amount.toNumber() == quantityHack);
    assert.ok(protocolVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(protocolVaultAccountInfo.amount.toNumber() == quantityInterest - quantityHack);
  });
});
