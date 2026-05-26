/// A star system — the top-level container.
/// Identified by name from journal FSDJump events.
#[derive(Debug, Clone, Default)]
pub struct System {
    pub name: String,
    pub system_address: u64,
    pub body_count_discovered: u32,
    pub body_count_total: u32,
    pub total_value: u64,
}

impl System {
    /// Create a new system with the given name and address.
    /// Counters and value start at zero.
    pub fn new(name: String, system_address: u64) -> Self {
        Self {
            name,
            system_address,
            ..Default::default()
        }
    }
}
