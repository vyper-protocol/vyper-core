export enum UpdateTrancheConfigFlags {
    HALT_FLAGS = 1 << 0,

    RESERVE_FAIR_VALUE_STALE_SLOT_THRESHOLD = 1 << 1,

    TRANCHE_FAIR_VALUE_STALE_SLOT_THRESHOLD = 1 << 2,
};

export enum TrancheHaltFlags{
    HALT_DEPOSITS = 1 << 0,

    HALT_REFRESHES = 1 << 1,

    HALT_REDEEMS = 1 << 2,
};

export const TrancheHaltFlagsAll =
    TrancheHaltFlags.HALT_DEPOSITS | TrancheHaltFlags.HALT_REFRESHES | TrancheHaltFlags.HALT_REDEEMS;