import * as anchor from '@project-serum/anchor'
export class LastUpdate {
    slot: number;
    padding: number[];

    constructor(slot: anchor.BN, padding: number[]) {
        this.slot = slot.toNumber();
        this.padding = padding;
    }
}