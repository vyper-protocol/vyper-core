import { PublicKey } from "@solana/web3.js";

export class RedeemLogicLendingFeeState {
    interestSplit: number;
    mgmtFee: number;
    perfFee: number;
    owner: PublicKey;


    constructor(interestSplit: number, mgmtFee: number,perfFee: number, owner: PublicKey) {
        this.interestSplit = interestSplit;
        this.mgmtFee = mgmtFee;
        this.perfFee = perfFee;
        this.owner = owner;
    }
} 