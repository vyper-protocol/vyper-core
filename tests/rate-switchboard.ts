import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { assert, expect } from "chai";
import { RateSwitchboard } from "../target/types/rate_switchboard";
import { bn } from "./utils";

const BTC_USD_SWITCHBOARD_AGGREGATOR = new PublicKey("8SXvChNYFhRq4EZuZvnhjrB3jJRQCv4k3P4W6hesH3Ee");
const USDC_USD_SWITCHBOARD_AGGREGATOR = new PublicKey("BjUgj6YCnFBZ49wF54ddBVA9qu8TeqkFtkbqmZcee8uW");
const XTZ_USD_SWITCHBOARD_AGGREGATOR = new PublicKey("F11LACseaLXuRaPSvnD6w15vSPHtx73YaGZv9293rQQm");
const SWITCHBOARD_AGGREGATORS = [
    BTC_USD_SWITCHBOARD_AGGREGATOR,
    USDC_USD_SWITCHBOARD_AGGREGATOR,
    XTZ_USD_SWITCHBOARD_AGGREGATOR,
];

describe("rate_switchboard", async () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const program = anchor.workspace.RateSwitchboard as Program<RateSwitchboard>;

    it("initialize", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await program.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        const rateDataAccount = await program.account.rateState.fetchNullable(rateData.publicKey);

        for (let i = 0; i < 10; i++) {
            if (i < SWITCHBOARD_AGGREGATORS.length) {
                expect(rateDataAccount.switchboardAggregators[i].toBase58()).to.eql(
                    SWITCHBOARD_AGGREGATORS[i].toBase58()
                );
                expect(rateDataAccount.fairValue[i]).to.be.not.eq(0);
            } else {
                expect(rateDataAccount.switchboardAggregators[i]).to.be.null;
                expect(rateDataAccount.fairValue[i]).to.be.eq(0);
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
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();
        await program.methods
            .refresh()
            .accounts({
                rateData: rateData.publicKey,
            })
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .rpc();

        expect((await program.account.rateState.fetchNullable(rateData.publicKey)).refreshedSlot.toNumber()).to.be.gt(
            0
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
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        try {
            await program.methods
                .refresh()
                .accounts({
                    rateData: rateData.publicKey,
                })
                .remainingAccounts(
                    SWITCHBOARD_AGGREGATORS.reverse().map((c) => ({ pubkey: c, isSigner: false, isWritable: false }))
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
            .remainingAccounts(SWITCHBOARD_AGGREGATORS.map((c) => ({ pubkey: c, isSigner: false, isWritable: false })))
            .signers([rateData])
            .rpc();

        try {
            await program.methods
                .refresh()
                .accounts({
                    rateData: rateData.publicKey,
                })
                .remainingAccounts(
                    [SWITCHBOARD_AGGREGATORS[0], SWITCHBOARD_AGGREGATORS[1]].map((c) => ({
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
