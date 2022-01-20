import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { getMintInfo, getTokenAccount } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, MintLayout, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { MockProtocol } from "../target/types/mock_protocol";
import assert from "assert";
import { findAssociatedTokenAddress } from "./utils";

describe.only("mock_protocol", () => {
  console.log(anchor.workspace);
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

  it("can create mint and vault", async () => {
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
});
