import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RateMock, IDL } from "../../target/types/rate_mock";

// ANCHOR_WALLET=~/Dev/VyperWallets/devnet-plugins-authority/authority.json ts-node -T ./scripts/rate_mock/set_fair_value.ts

const PLUGIN_PROGRAM_ID = new PublicKey("FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG");
const PLUGIN_STATE = new PublicKey("2MV14QPzUh1WVgMXY7nYuDzurHiHsx8qkhp7vPrV1shL");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const tx = await program.methods
        .setFairValue(5000)
        .accounts({
            authority: wallet.publicKey,
            rateData: PLUGIN_STATE,
        })
        .rpc();

    console.log("tx: " + tx);
};

main();
