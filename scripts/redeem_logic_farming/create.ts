import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { IDL } from "../../target/types/redeem_logic_farming";

const PLUGIN_PROGRAM_ID = new PublicKey("Fd87TGcYmWs1Gfa7XXZycJwt9kXjRs8axMtxCWtCmowN");

const main = async () => {
    const provider = anchor.AnchorProvider.env();

    const program = new Program(IDL, PLUGIN_PROGRAM_ID, provider);

    const stateAccount = anchor.web3.Keypair.generate();
    const interestSplit = 0.5;
    const capLow = 0.8;
    const capHigh = 1.2;

    const tx = await program.methods
        .initialize(interestSplit, capLow, capHigh)
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
