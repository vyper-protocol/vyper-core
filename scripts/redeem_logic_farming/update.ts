import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/redeem_logic_farming";

const PLUGIN_PROGRAM_ID = new PublicKey("Fd87TGcYmWs1Gfa7XXZycJwt9kXjRs8axMtxCWtCmowN");
const PLUGIN_STATE = new PublicKey("E5X5QbyUUNyPzJyobyRpmfEpP17mevGj9Nu8jKMsXhMb");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const interestSplit = 0;
    const capLow = 0.3;
    const capHigh = 0.3;

    const tx = await program.methods
        .update(interestSplit, capLow, capHigh)
        .accounts({
            redeemLogicConfig: PLUGIN_STATE,
            owner: provider.wallet.publicKey,
        })
        .rpc();

    console.log("tx: " + tx);
};

main();
