import { PublicKey } from "@solana/web3.js";

export class RateState {
    fairValue: number[];
    refreshedSlot: number;
    switchboardAggregators: PublicKey[];

    constructor(fairValue: number[], refreshedSlot: number, switchboardAggregators: PublicKey[]) {
        this.fairValue = fairValue;
        this.refreshedSlot = refreshedSlot;
        this.switchboardAggregators = switchboardAggregators;
    }
} 