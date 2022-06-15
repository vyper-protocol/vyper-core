import * as anchor from "@project-serum/anchor";

export interface TrancheConfigData {
    authority: anchor.web3.PublicKey,
    id: number[],
    protocolProgramId: anchor.web3.PublicKey
    depositedQuantity: anchor.BN[],
    capitalSplit: number[],
    interestSplit: number[],
    seniorTrancheMint: anchor.web3.PublicKey,
    juniorTrancheMint: anchor.web3.PublicKey,
    createdAt: anchor.BN,
    createSerum: boolean,
    trancheConfigBump: number,
    seniorTrancheMintBump: number,
    juniorTrancheMintBump: number
}