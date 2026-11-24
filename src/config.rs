#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    pub core_start: i64,
    pub upkeep_per_alloc: i64,
    /// Upper bound on evaluation steps; exceeding it aborts the run with a diagnostic.
    pub max_steps: u64,
    /// Maximum call depth, kept under the native stack limit.
    pub max_depth: u64,
    /// `balanced(a)` holds when `max(slots) - min(slots)` is at most this.
    pub balance_tolerance: i64,
    /// Largest allowed `apportionment[N]`.
    pub max_apportionment: i64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            core_start: 3,
            upkeep_per_alloc: 1,
            max_steps: 10_000_000,
            max_depth: 512,
            balance_tolerance: 0,
            max_apportionment: 4096,
        }
    }
}
