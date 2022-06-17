import { LastUpdate } from "./LastUpdate";

export class SlotTracking {
    lastUpdate: LastUpdate;
    staleSlotThreshold: number;

    constructor(lastUpdate: LastUpdate, staleSlotThreshold: number) {
        this.lastUpdate = lastUpdate
        this.staleSlotThreshold = staleSlotThreshold;
    }
}
