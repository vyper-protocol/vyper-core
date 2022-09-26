import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/rate_poolv2";

const PLUGIN_PROGRAM_ID = new PublicKey("5Vm2YZK3SeGbXbtQpKVByP9EvYy78ahnjFXKkf9B3yzW");
const PLUGIN_STATE = new PublicKey("Ft6sFaRyjSDHUtkhwDjn5Sz1Pke9Y466knSYNf5ac7yG");

const main = async () => {
    const provider = anchor.AnchorProvider.env();

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.rateState.fetch(PLUGIN_STATE);

    const sig = await program.methods
        .refresh()
        .accounts({
            rateData: PLUGIN_STATE,
            baseMint: account.baseMint,
            baseTokenAccount: account.baseTokenAccount,
            quoteMint: account.quoteMint,
            quoteTokenAccount: account.quoteTokenAccount,
            lpMint: account.lpMint,
        })
        .rpc();
    console.log("sig: ", sig);
};

main();
