import { SlotTracking } from "./SlotTracking";

export class TrancheFairValue {
    value: number[];
    slotTracking: SlotTracking;

    constructor(value: number[], slotTracking: SlotTracking) {
        this.value = value;
        this.slotTracking = slotTracking;
    }
}