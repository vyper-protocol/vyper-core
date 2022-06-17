import * as anchor from "@project-serum/anchor";
import { PublicKey } from "@solana/web3.js";
import { VyperCore } from "../../target/types/vyper_core";
import idlVyperCore from '../../target/idl/vyper_core.json';
import { TrancheConfig, TrancheData } from "./TrancheConfig";

export class Vyper {

    program: anchor.Program<VyperCore>;
    provider: anchor.AnchorProvider;
    trancheConfig: PublicKey;

    static create(provider: anchor.AnchorProvider, vyperCoreId: PublicKey): Vyper {
        const client = new Vyper();
        const program = new anchor.Program(idlVyperCore as any, vyperCoreId, provider) as anchor.Program<VyperCore>;
        client.program = program;
        client.provider = provider;
        return client;
    }

    async getTrancheConfiguration(trancheId: PublicKey): Promise<TrancheConfig> {
        let trancheConfig = new TrancheConfig();
        const trancheInfo = await this.program.account.trancheConfig.fetch(trancheId);
        trancheConfig.reserveMint = trancheInfo.reserveMint;
        trancheConfig.reserve = trancheInfo.reserve;
        trancheConfig.trancheData = TrancheData.create(trancheInfo.trancheData);
        trancheConfig.seniorTrancheMint = trancheInfo.seniorTrancheMint;
        trancheConfig.juniorTrancheMint = trancheInfo.juniorTrancheMint;
        trancheConfig.trancheAuthority = trancheInfo.trancheAuthority;
        trancheConfig.authoritySeed = trancheInfo.authoritySeed;
        trancheConfig.authorityBump = trancheInfo.authorityBump;
        trancheConfig.owner = trancheInfo.owner;
        trancheConfig.rateProgram = trancheInfo.rateProgram;
        trancheConfig.rateProgramState = trancheInfo.rateProgramState;
        trancheConfig.redeemLogicProgram = trancheInfo.redeemLogicProgram;
        trancheConfig.redeemLogicProgramState = trancheInfo.redeemLogicProgramState;
        trancheConfig.version = trancheInfo.version;
        trancheConfig.createdAt = trancheInfo.createdAt.toNumber();
        return trancheConfig;
    }
}

