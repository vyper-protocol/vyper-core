import * as anchor from "@project-serum/anchor";
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

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
