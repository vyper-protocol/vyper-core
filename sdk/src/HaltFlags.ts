export enum HaltFlags {
    // Disable deposits
    HALT_DEPOSITS = 1 << 0,

    // Disable refreshes
    HALT_REFRESHES = 1 << 1,

    // Disable redeems
    HALT_REDEEMS = 1 << 2,

    // Disable all operations
    HALT_ALL = HALT_DEPOSITS | HALT_REFRESHES | HALT_REDEEMS
}