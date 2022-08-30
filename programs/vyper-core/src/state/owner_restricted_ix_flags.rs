bitflags::bitflags! {
    pub struct OwnerRestrictedIxFlags: u16 {
        /// Owner restricted: Deposits
        const DEPOSITS = 1 << 0;

        /// Owner restricted: Refreshes
        const REFRESHES = 1 << 1;

        /// Owner restricted: Redeems
        const REDEEMS = 1 << 2;

        /// Disable all operations
        const ALL = Self::DEPOSITS.bits
                       | Self::REFRESHES.bits
                       | Self::REDEEMS.bits;

    }
}
