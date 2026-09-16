---
title: shared BBG database consumer migration
tags: cybergraph, bbg, storage, audit
date: 2026-09-12
status: consumer-validated
---

# Shared BBG database consumer migration

The local-storage feature now selects BBG's Fjall SSD backend. ApplicationGraph
opens a database directory or accepts an existing Database through
from_database. It re-exports Database and Backend so a host can share the same
owner with other BBG storage views. The optional legacy-redb-migration feature
enables explicit application-store import through BBG; no migration occurs on
normal open.

The application contract remains content closure validation, conditional heads,
request fingerprints and global claims. The existing corruption regression now
modifies the pinned Fjall partition after all graph handles close. A new
regression creates two application views and a shard view from the same
database, checks shared visibility and receipt lookup, commits a shard without
changing the application head, and checks exclusive ownership.

Cybergraph's path requirements for BBG, Zheng, Nox and Lens were aligned with
their current sibling package versions. The unconditional redb test dependency
was replaced by the same vendored Fjall used by BBG.

## Validation

Validation ran on macOS arm64 against the actual repository and sibling paths.
The first normal check stopped because Foculus required BBG ^0.2 while the
sibling package was 0.3.0. Its path requirements were aligned before testing;
Cargo.lock records the resolved versions.

All five application tests passed with local-storage, and the same five passed
with legacy-redb-migration. The production local-storage dependency tree
contains vendored Fjall 2.11.2 and no redb dependency. Scoped rustfmt and
git diff --check passed.

Clippy completed successfully with two existing warnings in src/api.rs:
subscriber type complexity and a collapsible network check. Three existing
Fjall compiler warnings remain. No warning was reported in the changed
application module or its tests; strict whole-library Clippy is not claimed.

Executed commands from this repository:

```sh
cargo test --offline --locked --features local-storage --test applications
cargo test --offline --locked --features legacy-redb-migration --test applications
cargo clippy --offline --locked --features legacy-redb-migration --lib --test applications --no-deps
cargo tree --offline --locked --features local-storage --edges normal --prefix none
```

This consumer migration does not establish native SignalChain publication,
network finality, or atomic composition of two independently committed calls.
Real legacy-file import and backend failure tests belong to BBG's shared
database suite. Cell's consumer evidence is recorded in its
[storage migration audit](../../cell/audit/shared-bbg-database.md).
