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

        const interestSplit = [bn(5000), bn(10000)];
        await program.methods
            .initialize(interestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        const redeemLogicAccount = await program.account.redeemLogicConfig.fetch(redeemLogicConfig.publicKey);
        expect(redeemLogicAccount.interestSplit.map((c) => c.toNumber())).to.eql(
            interestSplit.map((c) => c.toNumber())
        );
        expect(redeemLogicAccount.owner.toBase58()).to.eq(provider.wallet.publicKey.toBase58());
    });

    it("update", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();

        const originalInterestSplit = [bn(5000), bn(10000)];
        const newInterestSplit = [bn(6000), bn(10000)];
        await program.methods
            .initialize(originalInterestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();
        await program.methods
            .update(newInterestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
            })
            .rpc();
        const redeemLogicAccount = await program.account.redeemLogicConfig.fetch(redeemLogicConfig.publicKey);
        expect(redeemLogicAccount.interestSplit.map((c) => c.toNumber())).to.eql(
            newInterestSplit.map((c) => c.toNumber())
        );
    });

    it("reject non owner update", async () => {
        const redeemLogicConfig = anchor.web3.Keypair.generate();

        const originalInterestSplit = [bn(5000), bn(10000)];
        const newInterestSplit = [bn(6000), bn(10000)];

        await program.methods
            .initialize(originalInterestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        try {
            await program.methods
                .update(newInterestSplit)
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

        const interestSplit = [bn(5000), bn(10000)];

        await program.methods
            .initialize(interestSplit)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                owner: provider.wallet.publicKey,
                payer: provider.wallet.publicKey,
            })
            .signers([redeemLogicConfig])
            .rpc();

        const old_tranche_fv = [bn(5000), bn(10000)];
        const old_reserve_fv = bn(5000);
        const new_reserve_fv = bn(5000);

        const viewResult = await program.methods
            .execute(old_tranche_fv, old_reserve_fv, new_reserve_fv)
            .accounts({
                redeemLogicConfig: redeemLogicConfig.publicKey,
                signer: provider.wallet.publicKey,
            })
            .view();
        expect((viewResult as anchor.BN[]).map((c) => c.toNumber())).to.be.eql([1993, 1993]);
    });
});
