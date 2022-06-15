import { PublicKey } from '@solana/web3.js';
import { VyperCoreLending } from './types/vyper_core_lending';
import { Program } from '@project-serum/anchor';
import { TrancheConfigData } from './types/Trache';

export class TracheConfig {

    static async fetchTracheConfiguration(trancheId: PublicKey, program: Program<VyperCoreLending>): Promise<TrancheConfigData> {
        const account = await program.account.trancheConfig.fetch(trancheId);
        return account;
    }

}



