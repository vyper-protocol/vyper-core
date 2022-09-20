import * as anchor from "@project-serum/anchor";
import { Program, Wallet } from "@project-serum/anchor";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";
import { AggregatorAccount, loadSwitchboardProgram } from "@switchboard-xyz/switchboard-v2";
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";
import { IDL } from "../../target/types/rate_switchboard";

import * as fs from "fs";

const AGGREGATOR = new PublicKey("GvDMxPzN1sCj7L26YDK2HnMRXEQmQ2aemov8YBtPS7vR");

const main = async () => {
    const connection = new Connection("https://api.devnet.solana.com");

    const program = await loadSwitchboardProgram("devnet", connection);

    const aggregatorAccount = new AggregatorAccount({
        program,
        publicKey: AGGREGATOR,
    });

    const history = await aggregatorAccount.loadHistory();

    history.forEach((c) =>
        fs.appendFileSync("./out.csv", `${new Date(c.timestamp.toNumber() * 1000)};${c.value.toNumber()}\n`)
    );
};

main();
