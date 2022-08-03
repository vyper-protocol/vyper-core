bitflags::bitflags! {
    pub struct TrancheHaltFlags: u16 {
        /// Disable deposits
        const HALT_DEPOSITS = 1 << 0;

        /// Disable refreshes
        const HALT_REFRESHES = 1 << 1;

        /// Disable redeems
        const HALT_REDEEMS = 1 << 2;

        /// Disable all operations
        const HALT_ALL = Self::HALT_DEPOSITS.bits
                       | Self::HALT_REFRESHES.bits
                       | Self::HALT_REDEEMS.bits;

    }
}
