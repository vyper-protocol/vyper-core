import { SlotTracking } from "./SlotTracking";

export class ReserveFairValue {
    value: number;
    slotTracking: SlotTracking;

    constructor(value: number, slotTracking: SlotTracking) {
        this.value = value;
        this.slotTracking = slotTracking
    }
}