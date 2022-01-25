import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { createMintAndVault, getTokenAccount, createAccountRentExempt } from "@project-serum/common";
import { ASSOCIATED_TOKEN_PROGRAM_ID, Token, TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { Vyper } from "../target/types/vyper";

export async function findAssociatedTokenAddress(
  walletAddress: anchor.web3.PublicKey,
  tokenMintAddress: anchor.web3.PublicKey
): Promise<anchor.web3.PublicKey> {
  return (
    await anchor.web3.PublicKey.findProgramAddress(
      [walletAddress.toBuffer(), TOKEN_PROGRAM_ID.toBuffer(), tokenMintAddress.toBuffer()],
      ASSOCIATED_TOKEN_PROGRAM_ID
    )
  )[0];
}

export function bn(v: number): anchor.BN {
  return new anchor.BN(v);
}

export function to_bps(v: number): number {
  if (v > 1 || v < 0) throw Error("input needs to be between 0 and 1");
  return v * 10000;
}

export function from_bps(v: number): number {
  if (v > 10000 || v < 0) throw Error("input needs to be between 0 and 10000");
  return v / 10000;
}

export async function createDepositConfiguration(
  quantity: number,
  program: Program<Vyper>
): Promise<[anchor.web3.PublicKey, anchor.web3.PublicKey]> {
  const [depositMint, depositGod] = await createMintAndVault(
    program.provider,
    bn(quantity),
    program.provider.wallet.publicKey,
    0
  );
  const depositFromAccount = await findAssociatedTokenAddress(program.provider.wallet.publicKey, depositMint);

  const createDepositFromAccountTx = new anchor.web3.Transaction();
  createDepositFromAccountTx.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      depositMint,
      depositFromAccount,
      program.provider.wallet.publicKey,
      program.provider.wallet.publicKey
    ),
    Token.createTransferInstruction(
      TOKEN_PROGRAM_ID,
      depositGod,
      depositFromAccount,
      program.provider.wallet.publicKey,
      [],
      quantity
    )
  );
  await program.provider.send(createDepositFromAccountTx);

  return [depositMint, depositFromAccount];
}

export interface TranchesConfiguration {
  seniorTrancheMint: anchor.web3.PublicKey;
  seniorTrancheMintBump: number;
  seniorTrancheVault: anchor.web3.PublicKey;
  juniorTrancheMint: anchor.web3.PublicKey;
  juniorTrancheMintBump: number;
  juniorTrancheVault: anchor.web3.PublicKey;
}

export async function createTranchesConfiguration(
  depositMint: anchor.web3.PublicKey,
  program: Program<Vyper>
): Promise<TranchesConfiguration> {
  const [seniorTrancheMint, seniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("senior"), depositMint.toBuffer()],
    program.programId
  );
  const seniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, seniorTrancheMint);
  console.log("seniorTrancheMint: " + seniorTrancheMint);
  console.log("seniorTrancheVault: " + seniorTrancheVault);

  const [juniorTrancheMint, juniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from("junior"), depositMint.toBuffer()],
    program.programId
  );
  const juniorTrancheVault = await findAssociatedTokenAddress(program.provider.wallet.publicKey, juniorTrancheMint);
  console.log("juniorTrancheMint: " + juniorTrancheMint);
  console.log("juniorTrancheVault: " + juniorTrancheVault);

   return {
    seniorTrancheMint,
    seniorTrancheMintBump,
    seniorTrancheVault,
    juniorTrancheMint,
    juniorTrancheMintBump,
    juniorTrancheVault,
  };
}
