import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/rate_switchboard";

const PLUGIN_PROGRAM_ID = new PublicKey("2hGXiH1oEQwjCXRx8bNdHTi49ScZp7Mj2bxcjxtULKe1");
const PLUGIN_STATE = new PublicKey("5iz3MJ8cnRcXmzBDgNmb65HPwfeiw4djUfEaLqKuBA41");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });
    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.rateState.fetch(PLUGIN_STATE);

    const sig = await program.methods
        .refresh()
        .accounts({
            rateData: PLUGIN_STATE,
        })
        .remainingAccounts(
            (account.switchboardAggregators as (null | PublicKey)[])
                .filter((c) => c != null)
                .map((c) => ({ pubkey: c, isSigner: false, isWritable: false }))
        )
        .rpc();
    console.log("sig: ", sig);
};

main();
