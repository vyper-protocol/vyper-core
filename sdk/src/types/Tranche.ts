import * as anchor from "@project-serum/anchor";


export interface LastUpdateType {
    slot: anchor.BN,
    padding: number[],
}

export interface SlotTrackingType {
    lastUpdate: LastUpdateType,
    staleSlotThreshold: anchor.BN,
}

export interface ReserveFairValueType {
    value: number,
    slotTracking: SlotTrackingType
}

export interface TrancheFairValueType {
    value: number[],
    slotTracking: SlotTrackingType
}

export interface TrancheDataType {
    depositedQuantity: anchor.BN[],
    feeToCollectQuantity: anchor.BN,
    reserveFairValue: ReserveFairValueType,
    trancheFairValue: TrancheFairValueType,
    haltFlags: number,
    ownerRestrictedIx: number,
    padding: number[]
}


