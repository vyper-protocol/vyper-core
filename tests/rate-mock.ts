import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert, expect } from "chai";
import { RateMock } from "../target/types/rate_mock";
import { bn } from "./utils";

describe("rate_mock", () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const programRateMock = anchor.workspace.RateMock as Program<RateMock>;

    it("initialize", async () => {
        const rateData = anchor.web3.Keypair.generate();

        await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                authority: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .signers([rateData])
            .rpc();

        expect((await programRateMock.account.rateState.fetch(rateData.publicKey)).authority.toBase58()).to.be.eql(
            provider.wallet.publicKey.toBase58()
        );
        expect((await programRateMock.account.rateState.fetch(rateData.publicKey)).refreshedSlot.toNumber()).to.be.gte(
            0
        );
    });
});
