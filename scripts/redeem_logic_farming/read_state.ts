import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/redeem_logic_farming";

const PLUGIN_PROGRAM_ID = new PublicKey("Fd87TGcYmWs1Gfa7XXZycJwt9kXjRs8axMtxCWtCmowN");
const PLUGIN_STATE = new PublicKey("AjEifDpSYF57BGfJSnZ9qb9k3WNNknY1CJ8KeRoCwwU");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.redeemLogicConfig.fetch(PLUGIN_STATE);

    console.log("account: ", account);
};

main();
