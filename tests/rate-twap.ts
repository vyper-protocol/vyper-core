import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { RustDecimalWrapper } from "@vyper-protocol/rust-decimal-wrapper";
import { assert, expect } from "chai";
import { RateMock } from "../target/types/rate_mock";
import { RateTwap } from "../target/types/rate_twap";
import { bn } from "./utils";

describe.only("rate_twap", () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const programRateMock = anchor.workspace.RateMock as Program<RateMock>;
    const programRateTwap = anchor.workspace.RateTwap as Program<RateTwap>;

    it("initialize", async () => {
        const twapMinSlotDelta = 0;
        const twapSamplingSize = 10;

        const rateMockData = anchor.web3.Keypair.generate();
        await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                authority: provider.wallet.publicKey,
                rateData: rateMockData.publicKey,
            })
            .signers([rateMockData])
            .rpc();

        const rateTwapData = anchor.web3.Keypair.generate();
        await programRateTwap.methods
            .initialize({
                minSlotDelta: bn(twapMinSlotDelta),
                samplingSize: twapSamplingSize,
            })
            .accounts({
                rateState: rateTwapData.publicKey,
                rateStateSource: rateMockData.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([rateTwapData])
            .rpc();

        const twapAccountInfo = await programRateTwap.account.rateState.fetch(rateTwapData.publicKey);
        expect(twapAccountInfo.rateStateSource).to.be.eql(rateMockData.publicKey);
    });

    it("twap", async () => {
        const twapMinSlotDelta = 0;
        const twapSamplingSize = 10;

        const rateMockData = anchor.web3.Keypair.generate();
        await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                authority: provider.wallet.publicKey,
                rateData: rateMockData.publicKey,
            })
            .signers([rateMockData])
            .rpc();

        const rateTwapData = anchor.web3.Keypair.generate();
        await programRateTwap.methods
            .initialize({
                minSlotDelta: bn(twapMinSlotDelta),
                samplingSize: twapSamplingSize,
            })
            .accounts({
                rateState: rateTwapData.publicKey,
                rateStateSource: rateMockData.publicKey,
            })
            .signers([rateTwapData])
            .rpc();

        for (let i = 0; i < twapSamplingSize; i++) {
            await programRateMock.methods
                .setFairValue(5)
                .accounts({
                    authority: provider.wallet.publicKey,
                    rateData: rateMockData.publicKey,
                })
                .rpc();

            await programRateTwap.methods
                .refresh()
                .accounts({
                    rateState: rateTwapData.publicKey,
                    rateStateSource: rateMockData.publicKey,
                })
                .rpc();
        }

        for (let i = 0; i < twapSamplingSize; i++) {
            await programRateMock.methods
                .setFairValue(50)
                .accounts({
                    authority: provider.wallet.publicKey,
                    rateData: rateMockData.publicKey,
                })
                .rpc();

            await programRateTwap.methods
                .refresh()
                .accounts({
                    rateState: rateTwapData.publicKey,
                    rateStateSource: rateMockData.publicKey,
                })
                .rpc();
        }

        const twapAccountInfo = await programRateTwap.account.rateState.fetch(rateTwapData.publicKey);

        expect(
            //@ts-ignore
            twapAccountInfo.fairValue.map((c) => new RustDecimalWrapper(new Uint8Array(c)).toNumber())
        ).to.be.eql([50, 1, 1, 1, 1, 1, 1, 1, 1, 1]);
    });

    it("fails on refresh burst", async () => {
        const twapMinSlotDelta = 10;
        const twapSamplingSize = 10;

        const rateMockData = anchor.web3.Keypair.generate();
        await programRateMock.methods
            .initialize()
            .accounts({
                signer: provider.wallet.publicKey,
                authority: provider.wallet.publicKey,
                rateData: rateMockData.publicKey,
            })
            .signers([rateMockData])
            .rpc();

        const rateTwapData = anchor.web3.Keypair.generate();
        await programRateTwap.methods
            .initialize({
                minSlotDelta: bn(twapMinSlotDelta),
                samplingSize: twapSamplingSize,
            })
            .accounts({
                rateState: rateTwapData.publicKey,
                rateStateSource: rateMockData.publicKey,
            })
            .signers([rateTwapData])
            .rpc();

        try {
            for (let i = 0; i < twapSamplingSize; i++) {
                await programRateTwap.methods
                    .refresh()
                    .accounts({
                        rateState: rateTwapData.publicKey,
                        rateStateSource: rateMockData.publicKey,
                    })
                    .rpc();
            }
        } catch (err) {
            expect(err.error.errorCode.code).to.be.eql("AnotherTooRecentSample");
        }
    });
});
