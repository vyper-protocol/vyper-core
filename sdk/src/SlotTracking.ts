import * as anchor from '@project-serum/anchor';
import { LastUpdate } from "./LastUpdate";

export class SlotTracking {
    lastUpdate: LastUpdate;
    staleSlotThreshold: number;

    constructor(lastUpdate: LastUpdate, staleSlotThreshold: anchor.BN) {
        this.lastUpdate = lastUpdate
        this.staleSlotThreshold = staleSlotThreshold.toNumber();
    }
}
