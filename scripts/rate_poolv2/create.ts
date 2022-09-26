import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/rate_poolv2";

const PLUGIN_PROGRAM_ID = new PublicKey("5Vm2YZK3SeGbXbtQpKVByP9EvYy78ahnjFXKkf9B3yzW");

const ORCA_POOL_ID = new PublicKey("JU8kmKzDHF9sXWsnoznaFDFezLsE5uomX2JkRMbmsQP");

const QUOTE_USDC_TA = new PublicKey("75HgnSvXbWKZBpZHveX68ZzAhDqMzNDS29X6BGLtxMo1");
const QUOTE_USDC_MINT = new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
const BASE_SOL_TA = new PublicKey("ANP74VNsHwSrq9uUSjiSNyNWvf6ZPrKTmE4gHoNd13Lg");
const BASE_SOL_MINT = new PublicKey("So11111111111111111111111111111111111111112");

const LP_MINT = new PublicKey("APDFRM3HMr8CAGXwKHiu2f5ePSpaiEJhaURwhsRrUUt9");

const main = async () => {
    const provider = anchor.AnchorProvider.env();

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const rateData = anchor.web3.Keypair.generate();
    const tx = await program.methods
        .initialize()
        .accounts({
            signer: provider.wallet.publicKey,
            rateData: rateData.publicKey,
            pool: ORCA_POOL_ID,
            baseMint: BASE_SOL_MINT,
            baseTokenAccount: BASE_SOL_TA,
            quoteMint: QUOTE_USDC_MINT,
            quoteTokenAccount: QUOTE_USDC_TA,
            lpMint: LP_MINT,
        })
        .signers([rateData])
        .rpc();

    console.log("tx: " + tx);
    console.log("rate plugin state: " + rateData.publicKey);
};

main();
