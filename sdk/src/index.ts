export * from './Program'
export * from './TrancheConfig'

// for testing
// import { AnchorProvider } from '@project-serum/anchor';
// import { getVyperCoreProgram } from './Program';
// import { PublicKey } from "@solana/web3.js";
// import { TracheConfig } from './TrancheConfig';
// import * as dotenv from 'dotenv';
// dotenv.config();

// (async () => {
//     const provider = AnchorProvider.env();
//     const program = getVyperCoreProgram(provider);
//     const account = await TracheConfig.fetchTracheConfiguration(new PublicKey('EVeGAc8fA2EcwFpoVNdWLZSpxqJ38hBZtsjF5N3YaZuL'), program)
//     console.log(account);
// })()