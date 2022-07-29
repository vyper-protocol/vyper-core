import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/rate_switchboard";

const PLUGIN_PROGRAM_ID = new PublicKey("3By5v1am74SMQUkM9va8iHgNLVEjGVMRmMSt9nDiuczZ");
const PLUGIN_STATE = new PublicKey("5iz3MJ8cnRcXmzBDgNmb65HPwfeiw4djUfEaLqKuBA41");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });
    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.rateState.fetch(PLUGIN_STATE);

    // console.log("account: ", account);
    console.log("fairValue: " + account.fairValue);
    console.log("refreshedSlot: " + account.refreshedSlot.toNumber());
    console.log(
        "aggregators: " +
            (account.switchboardAggregators as (null | PublicKey)[]).filter((c) => c != null).map((c) => c.toBase58())
    );
};

main();
