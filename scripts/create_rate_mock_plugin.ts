import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RateMock, IDL } from "../target/types/rate_mock";

const PLUGIN_PROGRAM_ID = new PublicKey("FB7HErqohbgaVV21BRiiMTuiBpeUYT8Yw7Z6EdEL7FAG");

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
            authority: new PublicKey("6zoqN77QehDFPanib6WfBRcYnh31QSBdbL64Aj9Eq2fM"),
            rateData: rateData.publicKey,
        })
        .signers([rateData])
        .rpc();

    console.log("tx: " + tx);
    console.log("rate plugin state: " + rateData.publicKey);
};

main();
