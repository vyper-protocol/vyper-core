import { PublicKey } from "@solana/web3.js";
import { TrancheDataType, ReserveFairValueType, SlotTrackingType, LastUpdateType, TrancheFairValueType } from './types/Tranche'

export class TrancheConfig {
    reserveMint: PublicKey;
    reserve: PublicKey;
    trancheData: TrancheData;
    seniorTrancheMint: PublicKey;
    juniorTrancheMint: PublicKey;
    trancheAuthority: PublicKey;
    authoritySeed: PublicKey;
    authorityBump: number[];
    owner: PublicKey;
    rateProgram: PublicKey;
    rateProgramState: PublicKey;
    redeemLogicProgram: PublicKey;
    redeemLogicProgramState: PublicKey;
    version: number[];
    createdAt: number;
}

export class TrancheData {
    depositedQuantity: number[];
    feeToCollectQuantity: number;
    reserveFairValue: ReserveFairValue;
    trancheFairValue: TrancheFairValue;
    haltFlags: number;
    ownerRestrictedIx: number;
    padding: number[];

    static create(trancheDataInfo: TrancheDataType): TrancheData {
        const trancheData = new TrancheData();
        trancheData.depositedQuantity = trancheDataInfo.depositedQuantity.map((x) => x.toNumber())
        trancheData.feeToCollectQuantity = trancheDataInfo.feeToCollectQuantity.toNumber()
        trancheData.reserveFairValue = ReserveFairValue.create(trancheDataInfo.reserveFairValue)
        trancheData.trancheFairValue = TrancheFairValue.create(trancheDataInfo.trancheFairValue)
        trancheData.haltFlags = trancheDataInfo.haltFlags
        trancheData.ownerRestrictedIx = trancheDataInfo.ownerRestrictedIx
        trancheData.padding = trancheDataInfo.padding
        return trancheData;
    }
}

export class LastUpdate {
    slot: number;
    padding: number[];

    static create(lastUpdateInfo: LastUpdateType) {
        const lastUpdate = new LastUpdate();
        lastUpdate.slot = lastUpdateInfo.slot.toNumber();
        lastUpdate.padding = lastUpdateInfo.padding;
        return lastUpdate;
    }
}

export class SlotTracking {
    lastUpdate: LastUpdate;
    staleSlotThreshold: number;

    static create(slotTrackingInfo: SlotTrackingType) {
        const slotTracking = new SlotTracking();
        slotTracking.lastUpdate = LastUpdate.create(slotTrackingInfo.lastUpdate);
        slotTracking.staleSlotThreshold = slotTrackingInfo.staleSlotThreshold.toNumber();
        return slotTracking;
    }
}

export class ReserveFairValue {
    value: number;
    slotTracking: SlotTracking;

    static create(reserveFairValueInfo: ReserveFairValueType) {
        const reserveFairValue = new ReserveFairValue();
        reserveFairValue.value = reserveFairValueInfo.value;
        reserveFairValue.slotTracking = SlotTracking.create(reserveFairValueInfo.slotTracking)
        return reserveFairValue;
    }
}

export class TrancheFairValue {
    value: number[];
    slotTracking: SlotTracking;

    static create(trancheFairValueInfo: TrancheFairValueType) {
        const trancheFairValue = new TrancheFairValue();
        trancheFairValue.value = trancheFairValueInfo.value;
        trancheFairValue.slotTracking = SlotTracking.create(trancheFairValueInfo.slotTracking);
        return trancheFairValue;
    }
}

