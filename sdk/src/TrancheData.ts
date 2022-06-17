import { ReserveFairValue } from "./ReserveFairValue";
import { TrancheFairValue } from "./TrancheFairValue";

export class TrancheData {
    depositedQuantity: number[];
    feeToCollectQuantity: number;
    reserveFairValue: ReserveFairValue;
    trancheFairValue: TrancheFairValue;
    haltFlags: number;
    ownerRestrictedIx: number;
    padding: number[];

    constructor(
        depositedQuantity: number[],
        feeToCollectQuantity: number,
        reserveFairValue: ReserveFairValue,
        trancheFairValue: TrancheFairValue,
        ownerRestrictedIx: number,
        haltFlags: number,
        padding: number[]) {

        this.depositedQuantity = depositedQuantity
        this.feeToCollectQuantity = feeToCollectQuantity
        this.reserveFairValue = reserveFairValue
        this.trancheFairValue = trancheFairValue
        this.haltFlags = haltFlags
        this.ownerRestrictedIx = ownerRestrictedIx
        this.padding = padding
    }
}