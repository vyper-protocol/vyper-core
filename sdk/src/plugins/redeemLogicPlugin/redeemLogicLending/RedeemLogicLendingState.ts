import { PublicKey } from "@solana/web3.js";

export class RedeemLogicLendingState {
    interestSplit: number;
    fixedFeePerTranche: number;
    owner: PublicKey

    constructor(interestSplit: number, fixedFeePerTranche: number, owner: PublicKey) {
        this.interestSplit = interestSplit;
        this.fixedFeePerTranche = fixedFeePerTranche;
        this.owner = owner;
    }
} 