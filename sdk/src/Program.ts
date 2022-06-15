import { Program, Provider } from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';
import { VyperCoreLending } from './types/vyper_core_lending';
import idlVyperCore from './types/vyper_core_lending.json';

export const VYPER_CORE_PROGRAM_DEVNET_ID = new PublicKey('CJt5bFSebqNErzCdLNvk678S8Bmwdx2dCR8vrBS1eBoU');

export function getVyperCoreProgram(provider: Provider, pubkey: PublicKey = VYPER_CORE_PROGRAM_DEVNET_ID): Program<VyperCoreLending> {
    return new Program(idlVyperCore as any, pubkey, provider) as Program<VyperCoreLending>;
}
