// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cybergraph — signal lifecycle, equivocation detection, and local sync.
//!
//! owns layer 1 (validity) and layer 2 (ordering) from the structural sync spec.
//! calls `bbg::apply_tx` for layer 3 (completeness / state).

pub mod chain;
pub mod vdf;

pub use chain::{ChainError, CyberlinkRecord, Signal, SignalChain};
pub use vdf::{VdfProof, challenge_from_hash, evaluate as vdf_evaluate, verify as vdf_verify};

// Re-export the foundational identity types so downstream crates do not
// need a direct dependency on bbg just to name a Particle or NeuronId.
pub use bbg::{NeuronId, Particle};
