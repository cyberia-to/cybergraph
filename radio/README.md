# Cybergraph file transfer

Transfer files over Radio while keeping their bytes and resumable progress in
the same BBG database as application history. The receiving file becomes readable
after complete identity verification. Publishing an application head retains its
referenced files through the existing atomic operation.

The adapter is a separate crate so applications can select networking explicitly.
It uses the sibling `radio/iroh` transport and `cybergraph` local-storage API.

| API | purpose |
|---|---|
| `FileSource::open` | Give the host provider a reader for an authorized, sealed file |
| `FileProtocol` | Serve bounded ranges through Radio's existing router |
| `FileSink::open` | Reattach to an admitted BBG upload, including after restart |
| `receive_page` | Transfer only the missing parts in a bounded coverage page |
| `FileSink::seal` | Verify complete Blob identity before application publication |

The host provider checks the authenticated endpoint and requested particle/profile
for every range. Private namespaces and storage paths stay local. The application
supplies expected particle/length and an upload request, then publishes through
`ApplicationGraph::commit_with_blobs` after seal. Automatic peer discovery, replica
policy and remote protection receipts remain Foculus/application responsibilities.

Run the real transport and persistence tests from the Cybergraph repository:

```sh
CARGO_TARGET_DIR=radio/target cargo test --manifest-path radio/Cargo.toml --release --features hdd --test transfer --locked
```

Use this crate's own target directory: its dependency lock is independent of
the Radio workspace's lock. `hdd` enables redb alongside the default Fjall profile.
The fixtures create temporary stores and authenticated loopback endpoints,
interrupt a live transfer, reopen the stores and resume missing parts.

See the [contract](../specs/file-transfer.md), [executable integration](tests/transfer.rs)
and [[soft3/roadmap/storage/README|delivery project]]. Structured range proofs,
retention-aware GC, transport migration and remote recovery remain open work.
