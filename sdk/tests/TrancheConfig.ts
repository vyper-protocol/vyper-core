import { AnchorProvider } from '@project-serum/anchor';
import { Vyper } from '../src/index';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';
import { assert, expect } from "chai";

dotenv.config();

describe('TrancheConfig', () => {

    const provider = AnchorProvider.env();
    const vyper = Vyper.create(provider, new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'));

    it('fetch existing tranche configuration', async () => {
        const accounts = await provider.connection.getProgramAccounts(new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'));
        vyper.trancheId = new PublicKey(accounts[0].pubkey);
        const trancheConfig = await vyper.getTrancheConfiguration();

        assert.ok(trancheConfig.reserveMint)
        assert.ok(trancheConfig.reserve)
        assert.ok(trancheConfig.trancheData)
        assert.ok(trancheConfig.seniorTrancheMint)
        assert.ok(trancheConfig.juniorTrancheMint)
        assert.ok(trancheConfig.trancheAuthority)
        assert.ok(trancheConfig.authoritySeed)
        assert.ok(trancheConfig.authorityBump)
        assert.ok(trancheConfig.owner)
        assert.ok(trancheConfig.rateProgram)
        assert.ok(trancheConfig.rateProgramState)
        assert.ok(trancheConfig.redeemLogicProgram)
        assert.ok(trancheConfig.redeemLogicProgramState)
        assert.ok(trancheConfig.version)
        assert.ok(trancheConfig.createdAt)
    });
});