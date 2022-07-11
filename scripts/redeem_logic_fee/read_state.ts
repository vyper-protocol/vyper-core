import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RedeemLogicLendingFee, IDL } from "../../target/types/redeem_logic_lending_fee";

const PLUGIN_PROGRAM_ID = new PublicKey("3mq416it8YJsd5DKNuWeoCCAH8GYJfpuefHSNkSP6LyS");
const PLUGIN_STATE = new PublicKey("ES8pE5xKpyKxwSryJ1sPqWh4agfAhbQThFntLfi9JXez");

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
