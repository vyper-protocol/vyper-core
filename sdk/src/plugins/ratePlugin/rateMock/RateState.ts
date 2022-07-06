export class RateState {
    fairValue: number[];
    refreshedSlot: number;

    constructor(fairValue: number[], refreshedSlot: number) {
        this.fairValue = fairValue;
        this.refreshedSlot = refreshedSlot;
    }
} 