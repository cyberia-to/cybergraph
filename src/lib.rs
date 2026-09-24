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
//!   foculus   — sync-protocol mechanics: chain, VDF, equivocation, DAS, CRDT
//!   radio        — wire transport
//!
//! Signal/chain/VDF types are re-exported from `foculus` so downstream
//! callers (soma, soft3 SDK) have a single import for the public API.

pub mod api;
#[cfg(feature = "local-storage")]
pub mod application;
#[cfg(feature = "local-storage")]
pub mod catalog;
#[cfg(feature = "local-storage")]
pub mod files;
#[cfg(feature = "local-storage")]
pub mod text_archive;
#[cfg(feature = "local-storage")]
pub mod legacy_file;
pub mod content;
#[cfg(feature = "local-storage")]
pub mod native;
pub mod source;

// Re-export foundational identity types so downstream crates don't need a
// direct dependency on bbg just to name a Particle or NeuronId.
pub use bbg::{IntentRecord, NeuronId, Particle, SignalRecord};

// Re-export the signal lifecycle primitives owned by sync.
pub use foculus::{
    ChainError, CyberlinkRecord, SELF_NETWORK, Signal, SignalChain, VdfProof, challenge_from_hash,
    vdf_evaluate, vdf_verify,
};

// The five-verb public API.
pub use api::{ApiError, Cybergraph, Event, Filter, Intent, QueryError, Scope, private_network};
pub use source::BbgSource;

// inf query result type, re-exported so callers don't depend on inf-eval directly.
pub use inf_eval::Output as QueryOutput;
