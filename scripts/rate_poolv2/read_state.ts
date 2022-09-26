import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";
import { IDL } from "../../target/types/rate_poolv2";

const PLUGIN_PROGRAM_ID = new PublicKey("5Vm2YZK3SeGbXbtQpKVByP9EvYy78ahnjFXKkf9B3yzW");
const PLUGIN_STATE = new PublicKey("Ft6sFaRyjSDHUtkhwDjn5Sz1Pke9Y466knSYNf5ac7yG");

const main = async () => {
    const provider = anchor.AnchorProvider.env();

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.rateState.fetch(PLUGIN_STATE);

    console.log("account: ", account);
};

main();
