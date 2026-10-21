---
title: streamed file application validation
tags: cybergraph, audit, storage
status: partial
---
# streamed file application validation

Cybergraph revision `597d09127b6237b3b8afdb1855ae817e220caf23` uses BBG
`36cc420b48c5af8be5afaa3679d271c7b90155e3`. The [source closure and
environment](sources.json), [commands and log checksums](checks.json), and
[dependency lock](dependencies.lock) capture the tested inputs. Commands ran
against the source committed at those revisions.

Run from the Cybergraph root with
`CARGO_TARGET_DIR=/tmp/cyber-content-storage-20260924/target`:

| command after `cargo` | outcome | evidence |
|---|---|---|
| `test --release --features local-storage,bbg/backend-hdd --test files --test applications --test text_archive --test native_storage --test native_import --offline` | 34 passed, no warnings | [test log](logs/cybergraph-final-tests.log) |
| `check --no-default-features --offline` | pass, no warnings | [base log](logs/cybergraph-check-base.log) |

## demonstrated behavior

The `files` target in that command exercises both database profiles:

- `streamed_blobs_keep_existing_identities_and_support_bounded_seek_reads` checks
  exact agreement with existing `Content::Blob`, empty/binary inputs, sponge and
  part boundaries, zero tails, short reads and seek semantics.
- `streaming_crosses_the_old_whole_value_limit` generates, imports, publishes,
  reopens and reads `MAX_CONTENT_BYTES + 17` bytes (16 MiB + 17 at this revision)
  through bounded buffers. The fixture never constructs the entire file in memory.
- `application_publication_requires_namespace_scoped_sealed_content_and_binds_retry`
  rejects missing/unsealed/wrong-scope references and application rejection,
  verifies retention after reopen, and binds retries to the sorted Blob list.
- `concurrent_head_updates_retain_only_the_winning_publication` synchronizes two
  competing writers after validation. Only the accepted head acquires retention.

The application, TextArchive, native persistence and import targets provide
regression coverage for other views sharing the owner. These checks do not
convert those consumers to the new streamed-file API.

## scope and open work

`ApplicationGraph::files()` shares the existing BBG owner. The initial verifier
preserves exact-byte Blob identity; the wider structural identity decision is
[[soft3/roadmap/storage/identity|S1]]. File operations rely on trusted host
authorization. A namespace is not an access credential.

This is a library integration. Radio adapters, FS names/channels/patches, node
and Cyb APIs, Vault migration and remote restoration remain to implement. Sealed
files remain protected until release/GC semantics land. The test records logical
buffer bounds, not measured peak RSS or throughput. Both backend profiles used
the recorded APFS solid-state volume, with no physical HDD/power-loss campaign.

To reproduce, arrange sources.json repositories at their exact revisions as
siblings, copy dependencies.lock to `Cargo.lock` in an isolated checkout, prepare
the Cargo cache, and run checks.json commands. See the [file contract](../../../specs/files.md),
[[bbg/audit/storage/content-2026-09-24/README|BBG failure evidence]] and
[[soft3/roadmap/storage/acceptance|open complete acceptance rows]].
