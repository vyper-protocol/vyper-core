import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/redeem_logic_farming";

const PLUGIN_PROGRAM_ID = new PublicKey("8fSeRtFseNrjdf8quE2YELhuzLkHV7WEGRPA9Jz8xEVe");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const stateAccount = anchor.web3.Keypair.generate();
    const interestSplit = 5000;

    const tx = await program.methods
        .initialize(interestSplit)
        .accounts({
            redeemLogicConfig: stateAccount.publicKey,
            owner: new PublicKey("6zoqN77QehDFPanib6WfBRcYnh31QSBdbL64Aj9Eq2fM"),
            payer: provider.wallet.publicKey,
        })
        .signers([stateAccount])
        .rpc();

    console.log("tx: " + tx);
    console.log("redeem logic farming plugin state: " + stateAccount.publicKey);
};

main();
