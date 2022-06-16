import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { assert, expect } from "chai";
import { RedeemLogicLending } from "../target/types/redeem_logic_lending";
import { bn } from "./utils";

describe("redeem_logic_lending", async () => {
    const provider = anchor.AnchorProvider.env();

    // Configure the client to use the local cluster.
    anchor.setProvider(provider);
    const program = anchor.workspace.RedeemLogicLending as Program<RedeemLogicLending>;

    it("initialize", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();

        const interestSplit = 5000;
        await program.methods
            .initialize(interestSplit, bn(0))
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        const redeemLogicAccount = await program.account.redeemLogicConfig.fetch(redeemLogicConfig.publicKey);
        expect(redeemLogicAccount.interestSplit).to.eql(interestSplit);
        expect(redeemLogicAccount.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
    });

    it("update", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();

        const originalInterestSplit = 5000;
        const newInterestSplit = 6000;
        await program.methods
            .initialize(originalInterestSplit, bn(0))
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();
        await program.methods
            .update(newInterestSplit, bn(0))
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
            })
            .rpc();
        const redeemLogicAccount = await program.account.redeemLogicConfig.fetch(redeemLogicConfig.publicKey);
        expect(redeemLogicAccount.interestSplit).to.eql(newInterestSplit);
    });

    it("reject non owner update", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();

        const originalInterestSplit = 5000;
        const newInterestSplit = 6000;

        await program.methods
            .initialize(originalInterestSplit, bn(0))
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        try {
            await program.methods
                .update(newInterestSplit, bn(0))
                .accounts({
                    redeemLogicConfig: redeemLogicConfig.publicKey,
                    owner: anchor.web3.Keypair.generate().publicKey,
                })
                .rpc();
            expect(false).to.be.true;
        } catch (err) {
            assert(true);
        }
    });

    it("execute", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();
        const interestSplit = 2_000;
        await program.methods
            .initialize(interestSplit, bn(0))
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        const oldQuantity = [bn(100_000), bn(100_000)];
        const oldReserveFV = 6_000;
        const newReserveFV = 7_500;
        const tx = await program.methods
            .execute({
                oldQuantity: oldQuantity,
                oldReserveFairValueBps: oldReserveFV,
                newReserveFairValueBps: newReserveFV,
            })
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
            })
            .rpc();

        // TODO find a way to read the solana return value from the client
        // expect((viewResult.newQuantity as anchor.BN[]).map((c) => c.toNumber())).to.be.eql([100_000, 100_000]);
        // expect((viewResult.feeQuantity as anchor.BN).toNumber()).to.be.eql(0);
    });
});
