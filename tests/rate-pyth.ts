import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey, SYSVAR_EPOCH_SCHEDULE_PUBKEY } from "@solana/web3.js";
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";
import { assert, expect } from "chai";
import { RatePyth } from "../target/types/rate_pyth";
import { RateSwitchboard } from "../target/types/rate_switchboard";
import { bn } from "./utils";

const BTC_USD_PYTH_FEED = new PublicKey("GVXRSBjFk6e6J3NbVPXohDJetcTjaeeuykUpbQF8UoMU");
const SOL_USD_PYTH_FEED = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");

const PYTH_ORACLES = [BTC_USD_PYTH_FEED, SOL_USD_PYTH_FEED];

describe.only("rate_pyth", () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const program = anchor.workspace.RatePyth as Program<RatePyth>;

    it("initialize", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .remainingAccounts(PYTH_ORACLES.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        const rateDataAccount = await program.account.rateState.fetch(rateData.publicKey);

        for (let i = 0; i < 10; i++) {
            if (i < PYTH_ORACLES.length) {
                expect(rateDataAccount.pythOracles[i].toBase58()).to.eql(PYTH_ORACLES[i].toBase58());
                expect(new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[i])).toNumber()).to.be.not.eq(0);
            } else {
                expect(rateDataAccount.pythOracles[i]).to.be.null;
                expect(new RustDecimalWrapper(new Uint8Array(rateDataAccount.fairValue[i])).toNumber()).to.be.eq(0);
            }
        }

        expect(rateDataAccount.refreshedSlot.toNumber()).to.be.gt(0);
    });

    it("refresh", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .remainingAccounts(PYTH_ORACLES.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        const oldSlot = (await program.account.rateState.fetch(rateData.publicKey)).refreshedSlot.toNumber();

        await program.methods
            .refresh()
            .accounts({
                rateData: rateData.publicKey,
            })
            .remainingAccounts(PYTH_ORACLES.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .rpc();

        expect((await program.account.rateState.fetchNullable(rateData.publicKey)).refreshedSlot.toNumber()).to.be.gt(
            oldSlot
        );
    });

    it("cannot change aggregators order", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .remainingAccounts(PYTH_ORACLES.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        try {
            await program.methods
                .refresh()
                .accounts({
                    rateData: rateData.publicKey,
                })
                .remainingAccounts(
                    PYTH_ORACLES.reverse().map((c) => ({ pubkey: c, isSigner: false, isWritable: false }))
                )
                .rpc();
            expect(true).to.be.eq(false);
        } catch (err) {
            expect(err.error.errorCode.code).to.be.eql("RequireKeysEqViolated");
        }
    });

    it("cannot provide less aggregators", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .remainingAccounts(PYTH_ORACLES.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        try {
            await program.methods
                .refresh()
                .accounts({
                    rateData: rateData.publicKey,
                })
                .remainingAccounts(
                    [PYTH_ORACLES[0]].map((c) => ({
                        pubkey: c,
                        isSigner: false,
                        isWritable: false,
                    }))
                )
                .rpc();
            expect(true).to.be.eq(false);
        } catch (err) {
            expect(err.error.errorCode.code).to.be.eql("InvalidAggregatorsNumber");
        }
    });
});
