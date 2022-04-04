import { PublicKey } from "@solana/web3.js";
import Big from "big.js";

export abstract class Asset {
  abstract getApy(): Promise<Big>;
  abstract getLpTokenAccountValue(address: PublicKey): Promise<Big>;
}
