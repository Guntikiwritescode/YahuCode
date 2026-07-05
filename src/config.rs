//! Runtime configuration. Every runtime number lives here with a named default —
//! never an inline magic constant (handoff §12, Appendix H). The spike's defaults
//! were `CORE_START = 3`, `UPKEEP_PER_ALLOC = 1`.

/// Named runtime parameters for the coalition/persistence model (handoff §7.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Initial coalition core support (spike `CORE_START`).
    pub core_start: i64,
    /// Coalition cost charged per live allocation, per turn (spike `UPKEEP_PER_ALLOC`).
    pub upkeep_per_alloc: i64,
    /// Implementation safety valve for the Turing-complete core: an upper bound on
    /// evaluation steps. Exceeding it is an *implementation* abort (a loud diagnostic,
    /// never a silent hang and never an in-world halt — §12), not the in-language
    /// `elections` outcome. Large enough never to bite a real program.
    pub max_steps: u64,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        RuntimeConfig {
            core_start: 3,
            upkeep_per_alloc: 1,
            max_steps: 10_000_000,
        }
    }
}
