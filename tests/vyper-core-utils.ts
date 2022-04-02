import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { VyperCoreLending } from "../target/types/vyper_core_lending";
import { JUNIOR, SENIOR } from "./constants";
import { to_bps } from "./utils";

export interface CrateTrancheConfigData {
  capitalSplit: number[];
  interestSplit: number[];
  createSerum: boolean;
  protocolBump: number;
}

export function createTrancheConfigInput(
  capitalSplit: number[] = [to_bps(0.85), to_bps(0.15)],
  interestSplit: number[] = [to_bps(0.85), to_bps(0.15)],
  createSerum: boolean = false,
  protocolBump: number = 0
): CrateTrancheConfigData {
  return {
    capitalSplit: [to_bps(0.85), to_bps(0.15)],
    interestSplit: [to_bps(0.85), to_bps(1)],
    createSerum: false,
    protocolBump: 0,
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
  program: Program<VyperCoreLending>
): Promise<TranchesConfiguration> {
  const [seniorTrancheMint, seniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(SENIOR), proxyProtocolProgram.toBuffer(), depositMint.toBuffer()],
    program.programId
  );

  const [juniorTrancheMint, juniorTrancheMintBump] = await anchor.web3.PublicKey.findProgramAddress(
    [Buffer.from(JUNIOR), proxyProtocolProgram.toBuffer(), depositMint.toBuffer()],
    program.programId
  );

  return {
    seniorTrancheMint,
    seniorTrancheMintBump,
    juniorTrancheMint,
    juniorTrancheMintBump,
  };
}

export async function findTrancheConfig(
  mint: anchor.web3.PublicKey,
  seniorTrancheMint: anchor.web3.PublicKey,
  juniorTrancheMint: anchor.web3.PublicKey,
  vyperCoreProgramID: anchor.web3.PublicKey
): Promise<[anchor.web3.PublicKey, number]> {
  const [trancheConfig, trancheConfigBump] = await anchor.web3.PublicKey.findProgramAddress(
    [mint.toBuffer(), seniorTrancheMint.toBuffer(), juniorTrancheMint.toBuffer()],
    vyperCoreProgramID
  );

  return [trancheConfig, trancheConfigBump];
}
