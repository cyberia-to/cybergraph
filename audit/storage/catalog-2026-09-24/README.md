# Retained local names and CLI integration

Implementation: `56680f589727e52bb5803e470b0fb206f0fe550b`.
[sources.json](sources.json) pins the joint CLI/catalog dependency closure;
[checks.json](checks.json) gives complete commands and hashed outputs.
The [catalog contract](../../../specs/catalog.md) defines the local semantics.

The catalog uses immutable radix pages and the existing application publication
boundary. Names, selected heads, request receipts, history and payload retention
share one BBG owner. Point changes rewrite search paths and share unchanged pages.
Rename preserves binding, payload and content revision; edit changes content
revision while preserving binding. Stale or competing mutations conflict.

| Exercise | Command (Cybergraph working directory) | Observed result |
|---|---|---|
| Catalog behavior and growth | `cargo test --release --features legacy-redb-migration --test catalog --locked --offline` | All 4 test entries passed on both backend profiles; 4,100 retained revisions per profile cross a history page boundary and survive reopen |
| Final page/collision assertions | Filtered catalog command in [checks.json](checks.json) | Ordered pages, maximum-budget paths, prefixes and occupied rename destination passed |
| Existing application behavior | `cargo test --release --lib --test applications --features legacy-redb-migration --locked --offline` | 24 library and 6 application entries passed |
| Existing live file transfer | `cargo test --manifest-path radio/Cargo.toml --release --features hdd --test transfer --locked --offline` | All 3 entries passed; interrupted transfers resume across same/mixed profiles, denied reads and wrong bytes remain rejected |
| Strict lint | Catalog command in [checks.json](checks.json) | Passed without warnings |

The catalog suite includes same-head rename/edit competition, replay after later
commits, distinct name bindings sharing one payload, pinned old-state paging,
old-content retention, invalid paths/cursors and cross-namespace root rejection.
The CLI also exercises these APIs through real processes; its receipt is
[[radio/audit/storage/cli-2026-09-24/README]].

Both software backends ran on local SSD hardware. Growth checks cross the
history page boundary while retaining old states; they are not a latency or
memory scaling benchmark. Physical power loss, full remote synchronization,
channel/alias semantics and GC remain outside this result. Catalog pages encode
application references within Blob content; existing application-only archive
and transfer code does not establish recovery of a complete catalog-plus-payload
closure. That needs the content-aware migration work already tracked in S7.
