---
title: particle alias survey — one type, five definitions
tags: cybergraph, audit, particle
date: 2026-09-19
---
# particle alias survey

property #19 of [the launch registry](https://github.com/cyberia-to/cyber/blob/master/launch.md)
freezes `file::Particle` as the one type bbg, cybergraph, foculus and tru
share. Before that decision lands, this is what exists in the checkout on
2026-09-19: five independent representations, two incompatible byte widths,
and no domain separation on the path that will address 3.1M migrated
bostrom files. This document makes no decision; it lists what a decision
must reconcile.

## the definitions

1. `file::Particle` — `cyber-file` crate, `src/lib.rs:16`. `struct
   Particle([u8; 32])`, a newtype with hex/debug/display impls.
   `Particle::hash(data)` at `src/lib.rs:34` calls `hemera::hash(data)`
   directly on the whole byte slice — no chunking, no domain prefix.

2. `bbg::Particle` — `bbg/rs/src/types.rs:9`. `pub type Particle = [u8;
   32]` — a bare alias, no newtype, no hex/debug helpers of its own.
   `ParticleRecord` at `types.rs:19` wraps it for storage. This is the
   type cybergraph re-exports (`cybergraph/src/lib.rs:34`: `pub use
   bbg::{IntentRecord, NeuronId, Particle, SignalRecord};`) and the type
   foculus imports directly in five modules (`conflict.rs:24`,
   `finality.rs:23`, `pay_proof.rs:24`, `finality_evidence.rs:11`,
   `reconcile.rs:23`, all `use bbg::Particle;`).

3. A second, unrelated alias inside the same bbg crate —
   `bbg/rs/src/storage/application.rs:14`: `pub type Particle = [u8;
   32]`. Same shape as (2), declared independently in a different
   module. Two type aliases with the same name and the same
   representation, defined twice in one crate, is itself a finding
   independent of the cross-crate question: nothing stops the two from
   diverging in a future edit, and nothing today enforces that they stay
   the same alias.

4. `tru`'s local convention — `tru/rs/vocab.rs`. No `Particle` type at
   all; a bare `[u8; 32]` field (`vocab.rs:16`) and a private
   `particle_of()` function (`vocab.rs:28`) that calls
   `cyber_hemera::hash(bytes)` and copies the digest into an array. Same
   construction as `file::Particle::hash`, but reimplemented rather than
   depending on the `file` crate, so a change to hashing in `file` would
   not reach `tru` without a second edit.

5. `cybergraph/specs/particle.md` — the specification, not code.
   Describes a 64-byte flat namespace (`particle.md:3`: "64 raw bytes, no
   headers, no version prefix"), shared by particles, edges, neurons,
   commitments and nullifiers with domain-separation prefixes (`0x01`
   edges, `0x02` records, `0x03` nullifiers, `0x04` Merkle internal
   nodes; particle content addressing itself uses no prefix, per the
   domain table at `particle.md:66-76`). It also specifies a tree
   structure for content over 4KB (`particle.md:44-52`, left-balanced
   binary, `Hemera(chunk_bytes)` leaves) that none of the four code
   definitions above implement — every one of them hashes the full byte
   slice in a single call, with no chunking.

6. `radio/particle` — a CLI (`radio/particle/src/main.rs`), independent
   of all of the above, built directly on `cyber_bao` (Poseidon2 backend,
   BAO verified streaming: `hash`, `encode`, `decode`, `outboard`,
   `verify`). It does not import `file`, `bbg`, or `tru`'s particle
   types; it computes hashes itself.

## the contradictions a decision must resolve

- byte width: 32 bytes (`file`, `bbg`, `tru`) vs. 64 bytes (spec). Every
  particle computed today by code is half the width the spec says the
  namespace uses.
- domain separation: none in any of the four code paths; the spec
  reserves four prefix bytes for non-particle uses of the same hash
  function and explicitly special-cases bare content addressing as the
  unprefixed default. If particle identity is ever computed anywhere
  with a domain prefix (BAO's tree-node hashing in `radio/particle`
  hashes chunks and internal nodes together with content — worth
  checking against the spec's leaf/internal split before reuse), the
  address for the same bytes will not match `file::Particle::hash`'s
  direct call.
- chunking: the spec's 4KB tree applies past a size threshold; no
  current implementation branches on input size. A large bostrom file
  hashed by `file::Particle::hash` today gets a different address than
  the spec's tree construction would assign it once implemented.
- ownership: `bbg` carries two independently declared identical aliases
  in the same crate (finding 3); cybergraph re-exports bbg's alias
  rather than depending on `file`; `tru` reimplements the hash function
  rather than depending on `file`. Freezing `file::Particle` as the
  shared type means four call sites (bbg's two aliases, cybergraph's
  re-export, tru's `particle_of`) each need a migration, not one.

## what this does not decide

Which definition wins — `hemera(data)` at 32 bytes with no prefix (the
code today) or hemera over a chunked tree at 64 bytes with domain
separation (the spec) — is still the open decision logged on the launch
tracker. This survey exists so that whichever way it resolves, the scope
of the migration is known in advance: five source locations across four
crates, plus the CLI in `radio/particle` if it is ever asked to compute
particle identities rather than generic content hashes.
