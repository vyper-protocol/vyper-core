import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert, expect } from "chai";
import { RateMock } from "../target/types/rate_mock";
import { bn } from "./utils";

describe.only("rate_mock", async () => {
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
                rateData: rateData.publicKey,
            })
            .signers([rateData])
            .rpc();

        expect(
            (
                await programRateMock.account.rateState.fetch(
                    rateData.publicKey
                )
            ).fairValue.toNumber()
        ).to.eq(0);
    });

    it("set rate", async () => {
        const rateData = anchor.web3.Keypair.generate();

        const tx1 = await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .signers([rateData])
            .rpc();

        await programRateMock.methods
            .setFairValue(bn(1500))
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .rpc();

        expect(
            (
                await programRateMock.account.rateState.fetch(
                    rateData.publicKey
                )
            ).fairValue.toNumber()
        ).to.eq(1500);

        await programRateMock.methods
            .setFairValue(bn(2500))
            .accounts({
                signer: provider.wallet.publicKey,
                rateData: rateData.publicKey,
            })
            .rpc();

        expect(
            (
                await programRateMock.account.rateState.fetch(
                    rateData.publicKey
                )
            ).fairValue.toNumber()
        ).to.eq(2500);
    });
});
