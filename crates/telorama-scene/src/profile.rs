//! The canonical profile model: folded stacks.
//!
//! A folded stack is one sampled call path, root frame first, with the
//! number of samples that took exactly that path. It is the interchange
//! shape every profile source (pprof exports, perf scripts, Pyroscope
//! responses) can be folded into, and the only profile shape the
//! derivations read.

use serde::{Deserialize, Serialize};

/// One folded stack: frames from root to leaf, and its sample count.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StackSample {
    /// The call path, root frame first and leaf last.
    pub frames: Vec<String>,
    /// How many samples took exactly this path.
    pub count: u64,
}
