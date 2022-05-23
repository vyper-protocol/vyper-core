import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { VyperCoreLending } from "../target/types/vyper_core_lending";
import { JUNIOR, SENIOR } from "./constants";
import { bn, to_bps } from "./utils";

export interface CrateTrancheConfigData {
    capitalSplit: number[];
    interestSplit: number[];
    createSerum: boolean;
}

export function createTrancheConfigInput(
    capitalSplit: number[] = [to_bps(0.85), to_bps(0.15)],
    interestSplit: number[] = [to_bps(0.85), to_bps(0.15)],
    createSerum: boolean = false
): CrateTrancheConfigData {
    return {
        capitalSplit: [to_bps(0.85), to_bps(0.15)],
        interestSplit: [to_bps(0.85), to_bps(1)],
        createSerum: false,
    };
}

export interface TranchesConfiguration {
    seniorTrancheMint: anchor.web3.PublicKey;
    seniorTrancheMintBump: number;
    juniorTrancheMint: anchor.web3.PublicKey;
    juniorTrancheMintBump: number;
}

export async function createTranchesConfiguration(
    proxyProtocolProgram: anchor.web3.PublicKey,
    depositMint: anchor.web3.PublicKey,
    trancheID: anchor.BN,
    program: Program<VyperCoreLending>
): Promise<TranchesConfiguration> {
    const [seniorTrancheMint, seniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
        [trancheID.toArrayLike(Buffer, "be", 8), Buffer.from(SENIOR), proxyProtocolProgram.toBuffer(), depositMint.toBuffer()],
        program.programId
    );

    const [juniorTrancheMint, juniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
        [trancheID.toArrayLike(Buffer, "be", 8), Buffer.from(JUNIOR), proxyProtocolProgram.toBuffer(), depositMint.toBuffer()],
        program.programId
    );

    return {
        seniorTrancheMint,
        seniorTrancheMintBump,
        juniorTrancheMint,
        juniorTrancheMintBump,
    };
}

export function createTrancheID(): anchor.BN {
    return bn(new Date().getDate());
}

export async function findTrancheConfig(
    mint: anchor.web3.PublicKey,
    seniorTrancheMint: anchor.web3.PublicKey,
    juniorTrancheMint: anchor.web3.PublicKey,
    trancheID: anchor.BN,
    vyperCoreProgramID: anchor.web3.PublicKey
): Promise<[anchor.web3.PublicKey, number]> {
    const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
        [trancheID.toArrayLike(Buffer, "be", 8), mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
        vyperCoreProgramID
    );

    return [trancheConfig, trancheConfigBump];
}
