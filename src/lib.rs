// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! cybergraph — local-first cyberlink processor.
//!
//! cybergraph is the unified API for cyberlink lifecycle: declare an intent,
//! seal an intent into a signal, submit a discrete signal (`link`), subscribe
//! to events, and query cybergraph relations via `inf`.
//!
//! Backends:
//!   bbg          — authenticated state (store)
//!   cyber-sync   — sync-protocol mechanics: chain, VDF, equivocation, DAS, CRDT
//!   radio        — wire transport
//!
//! Signal/chain/VDF types are re-exported from `cyber-sync` so downstream
//! callers (soma, soft3 SDK) have a single import for the public API.

pub mod api;

// Re-export foundational identity types so downstream crates don't need a
// direct dependency on bbg just to name a Particle or NeuronId.
pub use bbg::{NeuronId, Particle, IntentRecord, SignalRecord};

// Re-export the signal lifecycle primitives owned by sync.
pub use cyber_sync::{
    ChainError, CyberlinkRecord, Signal, SignalChain,
    VdfProof, vdf_evaluate, vdf_verify, challenge_from_hash,
};

// The five-verb public API.
pub use api::{ApiError, Cybergraph, Event, Filter, Intent, Scope};
