---
title: Native format compatibility
tags: cybergraph, audit, storage
date: 2026-09-23
---
# Native format compatibility

BBG's new native metadata version 2 identifies the current root semantics.
Previously, version 1 alone could not distinguish older commitment encodings
from the currently released source profile. The coordinator now checks the
metadata framing/version before replay and allows legacy version 1 only after
complete recovery matches every receipt, root and exact record. The sole
comparison exception is the version word itself.

Read-only recovery and exact retries preserve the old records and transaction
identity. The next new operation promotes metadata in its durable transaction,
including an intent whose root does not change. The coordinator retains the
verified metadata and compares it inside each write transaction, so an external
format or pruning-policy change cannot be overwritten by a stale writer.

Injected failure after staging proves that promotion, history, receipts and
working state roll back together on both Fjall and redb. Retrying the same
request succeeds once and returns the same receipt after reopen. Tests also
cover unknown versions, truncated/malformed metadata, incompatible roots,
missing legacy history and metadata changes after recovery; rejection preserves
all seven native record domains and the last transaction identity.

## Previous executable evidence

Cyber's `examples/node_storage_compat.rs` ran the actual previous executable
(source `87f8359e`, SHA-256
`5cb3b61388d2232738a56187f01e7846af59a582a1939971cd2958af936e3181`)
and the new candidate in separate processes over temporary Fjall homes.
Both empty and populated stores passed: old writer creates, new reader preserves
state and exact retries, old reader still opens after a read-only visit, new
writer commits, old writer refuses upgraded metadata, new writer reopens and
both old/new receipts remain stable. Configuration bytes remain unchanged.

The existing product genesis/profile and native instance derivation are
unchanged. This qualifies the explicit predecessor's commitment semantics.
It does not qualify every historical version-1 store or rewrite old commitments.
Incompatible legacy data is preserved and refused for diagnosis or explicit
recovery using its original writer. Rejection preserves logical records;
storage-engine file housekeeping is not a byte-for-byte disk-image guarantee.

## Validation scope

The native format integration tests passed (3), the library suite passed (25),
and native storage tests passed (10). The acceptance invocation also exercises
native import, application storage and text archives with redb enabled:

```text
cargo test --features legacy-redb-migration --locked --lib \
  --test native_format --test native_storage --test native_import \
  --test applications --test text_archive --test text_archive_boundaries
```

The unrestricted `cargo test --features local-storage --locked` currently fails
to compile the existing `stack_*` proof experiments: their common helpers refer
to unpublished `zheng::execution::state`, and `stack_call` additionally requires
`execution::private`. Those APIs are intentionally outside the pinned node
dependency profile. No tests were disabled or ignored to hide this separate
integration failure. This report claims the explicitly listed storage coverage,
not a passing unrestricted Cybergraph suite. Existing vendored Fjall warnings
remain. Product release qualification and final artifact hashes belong to Cyber.
