import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RateMock, IDL } from "../../target/types/rate_mock";

const PLUGIN_PROGRAM_ID = new PublicKey("FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG");
const PLUGIN_STATE = new PublicKey("2MV14QPzUh1WVgMXY7nYuDzurHiHsx8qkhp7vPrV1shL");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });
    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);
    const account = await program.account.rateState.fetch(PLUGIN_STATE);

    console.log("account: ", account);
};

main();
