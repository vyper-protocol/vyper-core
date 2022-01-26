import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MockProtocol } from "../target/types/mock_protocol";
import assert from "assert";
import { bn, createMint, createMintAndDepositSource, createUserAndTokenAccount, findAssociatedTokenAddress } from "./utils";

describe("mock_protocol", () => {
  const program = anchor.workspace.MockProtocol as Program<MockProtocol>;

  anchor.setProvider(anchor.Provider.env());

  it("can initialize vault", async () => {
    const mint = await createMint(program.provider);
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
    const mint = await createMint(program.provider);
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

  it("deposits", async () => {
    const quantity = 1000;
    const [mint, srcAccount] = await createMintAndDepositSource(program.provider, quantity);
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

    await program.rpc.deposit(bn(quantity), vaultBump, {
      accounts: {
        mint,
        vault,
        srcAccount,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const mintInfo = await getMintInfo(program.provider, mint);
    const userVaultAccountInfo = await getTokenAccount(program.provider, srcAccount);
    const protocolVaultAccountInfo = await getTokenAccount(program.provider, vault);

    assert.ok(mintInfo.supply.toNumber() == quantity);
    assert.ok(userVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(userVaultAccountInfo.amount.toNumber() == 0);
    assert.ok(protocolVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(protocolVaultAccountInfo.amount.toNumber() == quantity);
  });

  it("redeem", async () => {
    const quantity = 1000;
    const redeemQuantity = 200;
    const [mint, srcAccount] = await createMintAndDepositSource(program.provider, quantity);
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

    await program.rpc.deposit(bn(quantity), vaultBump, {
      accounts: {
        mint,
        vault,
        srcAccount,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    await program.rpc.redeem(bn(redeemQuantity), vaultBump, {
      accounts: {
        mint,
        vault,
        destAccount: srcAccount,
        authority: program.provider.wallet.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
      },
    });

    const mintInfo = await getMintInfo(program.provider, mint);
    const userVaultAccountInfo = await getTokenAccount(program.provider, srcAccount);
    const protocolVaultAccountInfo = await getTokenAccount(program.provider, vault);

    assert.ok(mintInfo.supply.toNumber() == quantity);
    assert.ok(userVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(userVaultAccountInfo.amount.toNumber() == redeemQuantity);
    assert.ok(protocolVaultAccountInfo.mint.toBase58() == mint.toBase58());
    assert.ok(protocolVaultAccountInfo.amount.toNumber() == quantity - redeemQuantity);
  });
});
