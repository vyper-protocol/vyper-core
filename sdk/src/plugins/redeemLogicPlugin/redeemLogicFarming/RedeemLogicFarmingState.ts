import { PublicKey } from "@solana/web3.js";

export class RedeemLogicFarmingState {
    interestSplit: number;
    owner: PublicKey

    constructor(interestSplit: number, owner: PublicKey) {
        this.interestSplit = interestSplit;
        this.owner = owner;
    }
} 