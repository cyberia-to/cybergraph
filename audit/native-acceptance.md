---
title: native acceptance and recovery validation
tags: cybergraph, audit, storage
date: 2026-09-12
---
# native acceptance and recovery validation

`src/native/`, behind `local-storage`, consumes BBG's shared Database and
isolated Prepared transition. Soft3 calls it for JSON links/payments and both
native frame versions. The [contract](../specs/native-storage.md) defines the
local profile.

## checks

Environment: macOS 26.4.1 / Darwin 25.4.0, aarch64, APFS on the internal SSD,
Rust 1.95.0. Commands:

```sh
cargo test --offline --locked --features local-storage --lib \
  --test native_storage --test native_import --test applications
cargo clippy --offline --features local-storage --lib --test native_storage
```

The targeted suite contains 20 library, 5 application, 10 native storage and
11 import tests. It uses actual Fjall files and reopens with an empty working
graph. Clippy reports two existing API and three vendored Fjall warnings;
none belong to the new native path.

Tests cover original payment receipts after later commits/reopen, conflicting
IDs, distinct identity-less commands, global positions for multiple neurons,
whole-batch double-spend rejection, checked economics, native intents/subsidies,
explicit networks, and a real Zheng proof that verifies after durable recovery.
A test-only error after all writes are staged restores graph, ledger, chain,
block, export, history and receipt. The product contains no test fault hook.

Recovery rejects independently corrupted state, balances, blocks, receipts,
history and export, missing/extra records, and a changed genesis. Runtime reads
report missing committed blocks/history as corruption. Import cases include
truncation, historical complete encodings, exact byte cursors, marker tampering,
copied completion markers, wrong-genesis nonmutation, interrupted prefixes,
64-event bounds and a source larger than the 4 MiB command limit.

Source identity is Hemera of `cybergraph/legacy-source/v1\0` followed by exact
tape bytes. Completion checks original frame bytes, source-bound batch IDs,
zero historical timestamps and exact event count. Resume skips only a verified
committed prefix. Batches shrink only after known resource-limit failures.

## wider suite and remaining gates

The complete `cargo test --offline --locked --features local-storage` was
attempted and fails compiling existing stack tests against current Nox.
`tests/common/mod.rs` imports removed `OrderId`/`Tag` and treats `Order` as a
generic allocator; other stack tests import `NounId`/`Tag`. This is not a
passing complete Cybergraph suite. Those computation test migrations remain
separate work.

[BBG composition tests](../../bbg/audit/native-record-composition.md) cover
actual redb read/write/sync faults and lost sync acknowledgement across native
records, shards and applications. [Cyber artifact tests](../../cyber/audit/native-acceptance.md)
cover dropped HTTP responses and SIGKILL. Physical power cuts, a full filesystem,
archive movement, consensus and authenticated reward policy remain separate.

Working graph/chains remain in RAM; recovery replays history and recomputes
roots. Pruning and root construction still scan global state. Bounded undo
avoids graph cloning; billion-entry startup and incremental-root performance
were not qualified. Concurrent BBG proof/query/state/package and neuron work
was preserved; artifact provenance records the actual dirty source closure.
