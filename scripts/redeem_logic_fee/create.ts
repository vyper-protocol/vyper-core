import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { RedeemLogicLendingFee, IDL } from "../../target/types/redeem_logic_lending_fee";

const PLUGIN_PROGRAM_ID = new PublicKey("3mq416it8YJsd5DKNuWeoCCAH8GYJfpuefHSNkSP6LyS");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const wallet = Wallet.local();
    const provider = new anchor.AnchorProvider(connection, wallet, {
        commitment: "confirmed",
    });

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const stateAccount = anchor.web3.Keypair.generate();
    const interestSplit = 5000;
    const mgmtFeeBps = 100; // 1%
    const perfFeeBps = 10_000; // 100%

    const tx = await program.methods
        .initialize(interestSplit, mgmtFeeBps, perfFeeBps)
        .accounts({
            redeemLogicConfig: stateAccount.publicKey,
            owner: new PublicKey("6zoqN77QehDFPanib6WfBRcYnh31QSBdbL64Aj9Eq2fM"),
            payer: provider.wallet.publicKey,
        })
        .signers([stateAccount])
        .rpc();

    console.log("tx: " + tx);
    console.log("redeem logic plugin state: " + stateAccount.publicKey);
};

main();
