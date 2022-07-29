import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/rate_switchboard";

const PLUGIN_PROGRAM_ID = new PublicKey("2hGXiH1oEQwjCXRx8bNdHTi49ScZp7Mj2bxcjxtULKe1");

const LP_ORCA_SOL_USDC_SWITCHBOARD_AGGREGATOR = new PublicKey("3By5v1am74SMQUkM9va8iHgNLVEjGVMRmMSt9nDiuczZ");
const SOL_USD_SWITCHBOARD_AGGREGATOR = new PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");
const SWITCHBOARD_AGGREGATORS = [LP_ORCA_SOL_USDC_SWITCHBOARD_AGGREGATOR, SOL_USD_SWITCHBOARD_AGGREGATOR];

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const rateData = anchor.web3.Keypair.generate();
    const tx = await program.methods
        .initialize()
        .accounts({
            signer: provider.wallet.publicKey,
            rateData: rateData.publicKey,
        })
        .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
        .signers([rateData])
        .rpc();

    console.log("tx: " + tx);
    console.log("rate plugin state: " + rateData.publicKey);
};

main();
