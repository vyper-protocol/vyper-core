import { PublicKey } from "@solana/web3.js";
import { TrancheData } from "./TrancheData";

export class TrancheConfig {
    reserveMint: PublicKey;
    reserve: PublicKey;
    trancheData: TrancheData;
    seniorTrancheMint: PublicKey;
    juniorTrancheMint: PublicKey;
    trancheAuthority: PublicKey;
    authoritySeed: PublicKey;
    authorityBump: number[];
    owner: PublicKey;
    rateProgram: PublicKey;
    rateProgramState: PublicKey;
    redeemLogicProgram: PublicKey;
    redeemLogicProgramState: PublicKey;
    version: number[];
    createdAt: number;

    constructor(
        reserveMint: PublicKey,
        reserve: PublicKey,
        trancheData: TrancheData,
        seniorTrancheMint: PublicKey,
        juniorTrancheMint: PublicKey,
        trancheAuthority: PublicKey,
        authoritySeed: PublicKey,
        authorityBump: number[],
        owner: PublicKey,
        rateProgram: PublicKey,
        rateProgramState: PublicKey,
        redeemLogicProgram: PublicKey,
        redeemLogicProgramState: PublicKey,
        version: number[],
        createdAt: number,
    ) {
        this.reserveMint = reserveMint;
        this.reserve = reserve;
        this.trancheData = trancheData;
        this.seniorTrancheMint = seniorTrancheMint;
        this.juniorTrancheMint = juniorTrancheMint;
        this.trancheAuthority = trancheAuthority;
        this.authoritySeed = authoritySeed;
        this.authorityBump = authorityBump;
        this.owner = owner;
        this.rateProgram = rateProgram;
        this.rateProgramState = rateProgramState;
        this.redeemLogicProgram = redeemLogicProgram;
        this.redeemLogicProgramState = redeemLogicProgramState;
        this.version = version;
        this.createdAt = createdAt;
    }

}










