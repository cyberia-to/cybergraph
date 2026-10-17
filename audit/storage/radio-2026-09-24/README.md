---
title: Radio transfer and BBG restart validation
tags: cybergraph, bbg, radio, audit
status: partial
---
# Radio transfer and BBG restart validation

Cybergraph `c4854a24e90df1f4d88d6e25ed6ef854b2714a11` adds the
`cybergraph-radio` adapter. It uses Radio
`fe72dbd2f15370bfc333709d4870cb2f2b1d0d15` and the BBG revision pinned in
[sources.json](sources.json). The [receipt](checks.json) includes exact commands
and normalized/raw log checksums; the adapter's [lock](../../../radio/Cargo.lock)
fixes dependency resolution. Commands ran against those committed sources.

## executable checks

Run from Cybergraph with the separate
`CARGO_TARGET_DIR=/tmp/cyber-content-storage-20260924/target-adapter`.

| command after `cargo` | result | evidence |
|---|---|---|
| `test --manifest-path radio/Cargo.toml --release --features hdd --test transfer --locked --offline` | 3 passed | [real transfer](logs/cybergraph-radio-transfer.log) |
| `clippy --manifest-path radio/Cargo.toml --all-targets --features hdd --locked --offline --no-deps -- -D warnings` | pass | [lint](logs/cybergraph-radio-clippy.log) |
| `check --manifest-path radio/Cargo.toml --locked --offline` | pass | [default Fjall build](logs/cybergraph-radio-default.log) |

No compiler warnings were emitted. These are adapter checks, alongside
[[radio/audit/storage/radio-2026-09-24/README|Radio's framing/capability tests]].

## observed workflow

`restart_resumes_only_missing_parts_and_retains_through_the_shared_owner` in the
transfer command above runs all source/receiver combinations of Fjall and redb,
including Fjall → redb and redb → Fjall. At the pinned revision it uses a
1 MiB + 57 byte fixture, source parts of 31 KiB and receiver parts of 64 KiB.

The source blocks a live request after two complete receiver parts. The test
stops the Radio router, observes the transfer error, then closes both physical
BBG owners. Reopening confirms the accepted parts remain present and the file
remains unpublished. New endpoints resume the same upload from persisted coverage.
An instrumented source counts requested payload bytes: completed parts are not
requested again. This counter measures source payload reads, not QUIC wire bytes
or retransmission overhead.

After complete verification, publication commits the application head and
retention. The test stops transport again, reopens the receiver and checks the
retained head and exact output. The fixture root contains only the two BBG owner
paths; this protocol creates no Radio database/blob directory.

The other tests run on each backend: an unauthorized endpoint receives no bytes
and advances no coverage; a source returning substituted bytes fills staging but
fails final Hemera verification and cannot publish a head. Explicit cancellation
then prevents reattaching that upload.

## scope and remaining work

The host provider grants an exact file/profile capability after authenticating
the endpoint; the namespace stays local. Blocking jobs contain only local BBG
operations. Each network frame finishes before its sink write, and the next
request waits for that write. Cancellation may leave the current bounded storage
job finishing; its persisted request and coverage determine restart behavior.

Both profiles ran on the recorded local APFS solid-state volume. These are
loopback transfer and application-reopen tests, not independent failure domains,
physical HDD tests, power-loss qualification or performance measurements.

The adapter is ready for host integration under [its contract](../../../specs/file-transfer.md).
Foculus scheduling, product APIs, legacy-store migration, authenticated range
proofs, retention release/GC and remote protection receipts remain open. Existing
sealed content stays protected by BBG. Source credentials use the existing Radio
handshake; Vault custody and neuron authorization retain their separate contracts.
