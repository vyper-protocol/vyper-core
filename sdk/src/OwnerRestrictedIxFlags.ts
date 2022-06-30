export enum OwnerRestrictedIxFlags {
    NONE = 0,
    
    DEPOSITS = 1 << 0,

    // Owner restricted: Refreshes
    REFRESHES = 1 << 1,

    // Owner restricted: Redeems
    REDEEMS = 1 << 2,

    // Disable all operations
    ALL = DEPOSITS | REFRESHES | REDEEMS
}