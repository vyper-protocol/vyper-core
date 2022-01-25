import * as anchor from "@project-serum/anchor";
import { BN } from "@project-serum/anchor";
import { Vyper } from "../../target/types/vyper";
import { Program } from "@project-serum/anchor";
import { PublicKey, Keypair, SystemProgram } from "@solana/web3.js";
import { findAssociatedTokenAddress } from "../utils";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  Token,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

export const DEX_PID = new PublicKey(
  "9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin"
);

export interface SerumAccounts {
  market: anchor.web3.Signer;
  requestQueue: anchor.web3.Signer;
  eventQueue: anchor.web3.Signer;
  asks: anchor.web3.Signer;
  bids: anchor.web3.Signer;
  vaultOwner: anchor.web3.PublicKey;
  vaultOwnerNonce: number;
  trancheSerumVault: anchor.web3.PublicKey;
  usdcSerumVault: anchor.web3.PublicKey;
}

export async function createSerumAccounts(
  trancheMint: anchor.web3.PublicKey,
  usdcMint: anchor.web3.PublicKey,
  program: Program<Vyper>
): Promise<SerumAccounts> {
  const market = new Keypair();
  const requestQueue = new Keypair();
  const eventQueue = new Keypair();
  const asks = new Keypair();
  const bids = new Keypair();

  await createSerumAccount(market, 376, program);
  await createSerumAccount(requestQueue, 640, program);
  await createSerumAccount(eventQueue, 65536, program);
  await createSerumAccount(asks, 65536, program);
  await createSerumAccount(bids, 65536, program);

  const [vaultOwner, vaultOwnerNonce] = await getVaultOwnerAndNonce(
    market.publicKey
  );
  const usdcSerumVault = await findAssociatedTokenAddress(vaultOwner, usdcMint);
  const trancheSerumVault = await findAssociatedTokenAddress(
    vaultOwner,
    trancheMint
  );

  const createTrancheSerumVaultTx = new anchor.web3.Transaction();
  createTrancheSerumVaultTx.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      trancheMint,
      trancheSerumVault,
      vaultOwner,
      program.provider.wallet.publicKey
    )
  );
  await program.provider.send(createTrancheSerumVaultTx);

  const createUsdcSerumVaultTx = new anchor.web3.Transaction();
  createUsdcSerumVaultTx.add(
    Token.createAssociatedTokenAccountInstruction(
      ASSOCIATED_TOKEN_PROGRAM_ID,
      TOKEN_PROGRAM_ID,
      usdcMint,
      usdcSerumVault,
      vaultOwner,
      program.provider.wallet.publicKey
    )
  );
  await program.provider.send(createUsdcSerumVaultTx);

  return {
    market,
    requestQueue,
    eventQueue,
    asks,
    bids,
    vaultOwner,
    vaultOwnerNonce,
    trancheSerumVault,
    usdcSerumVault,
  };
}

export async function getVaultOwnerAndNonce(
  marketPublicKey: PublicKey,
  dexProgramId: PublicKey = DEX_PID
): Promise<[anchor.web3.PublicKey, number]> {
  const nonce = new BN(0);
  while (nonce.toNumber() < 255) {
    try {
      const vaultOwner = await PublicKey.createProgramAddress(
        [marketPublicKey.toBuffer(), nonce.toArrayLike(Buffer, "le", 8)],
        dexProgramId
      );
      return [vaultOwner, nonce.toNumber()];
    } catch (e) {
      nonce.iaddn(1);
    }
  }
  throw new Error("Unable to find nonce");
}

async function createSerumAccount(
  account: anchor.web3.Signer,
  unpaddedLen: number,
  program: Program<Vyper>
) {
  const space = 5 + unpaddedLen + 7;
  const createAccountTx = new anchor.web3.Transaction();
  createAccountTx.add(
    SystemProgram.createAccount({
      fromPubkey: program.provider.wallet.publicKey,
      newAccountPubkey: account.publicKey,
      lamports:
        await program.provider.connection.getMinimumBalanceForRentExemption(
         space 
        ),
      space,
      programId: DEX_PID,
    })
  );

  createAccountTx.recentBlockhash = (
    await program.provider.connection.getRecentBlockhash()
  ).blockhash;

  createAccountTx.feePayer = program.provider.wallet.publicKey;
  createAccountTx.addSignature(
    program.provider.wallet.publicKey,
    (await program.provider.wallet.signTransaction(createAccountTx)).signature
  );
  createAccountTx.sign(account);

  await program.provider.send(createAccountTx);
}
