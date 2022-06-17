import { ReserveFairValue } from "./ReserveFairValue";
import { TrancheFairValue } from "./TrancheFairValue";
import { HaltFlags } from "./HaltFlags";
import { OwnerRestrictedIxFlags } from "./OwnerRestrictedIxFlags";

export class TrancheData {
    depositedQuantity: number[];
    feeToCollectQuantity: number;
    reserveFairValue: ReserveFairValue;
    trancheFairValue: TrancheFairValue;
    haltFlags: HaltFlags;
    ownerRestrictedIx: OwnerRestrictedIxFlags;

    constructor(
        depositedQuantity: number[],
        feeToCollectQuantity: number,
        reserveFairValue: ReserveFairValue,
        trancheFairValue: TrancheFairValue,
        ownerRestrictedIx: OwnerRestrictedIxFlags,
        haltFlags: HaltFlags,
    ) {

        this.depositedQuantity = depositedQuantity;
        this.feeToCollectQuantity = feeToCollectQuantity;
        this.reserveFairValue = reserveFairValue;
        this.trancheFairValue = trancheFairValue;
        this.haltFlags = haltFlags;
        this.ownerRestrictedIx = ownerRestrictedIx;
    }
}