export * from './Vyper';


// for testing
import { AnchorProvider } from '@project-serum/anchor';
import { Vyper } from './Vyper';
import { PublicKey, } from "@solana/web3.js";
import * as dotenv from 'dotenv';

dotenv.config();

(async () => {
    const provider = AnchorProvider.env();
    const vyper = Vyper.create(provider, new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'));
    const accounts = await provider.connection.getProgramAccounts(new PublicKey('mb9NrZKiC3ZYUutgGhXwwkAL6Jkvmu5WLDbxWRZ8L9U'));
    const account = await vyper.getTrancheConfiguration(new PublicKey(accounts[0].pubkey));
    console.log(account);
})() 