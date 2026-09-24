---
title: streamed file content
tags: cybergraph, file, storage, spec
status: implementation
---
# streamed file content

`files::Files` is the local streamed-content adapter over BBG's shared Database.
`ApplicationGraph::files()` attaches it to the same owner as application history,
requests and heads. It preserves the existing `Content::Blob` particle computed
by Hemera over exact bytes. Structured identities and transferable range proofs
remain subject to [[soft3/roadmap/storage/identity|S1]].

## write, resume and verify

`begin(Upload { namespace, request }, particle, length, part_bytes)` durably
binds the operation. Physical part size stays within BBG's per-call budget;
total length can span any number of parts. `write_part` accepts exact positions
in any order. The final part binds the remainder of the declared length.
Idempotent repeats preserve progress; changed specifications or bytes conflict.

`progress`, `uploads` and `coverage` expose bounded resumable state. Coverage
describes durable arrivals. A full-stream `verify` session checks every part
and the canonical Blob particle before the file becomes readable. `step`
bounds verification work and supports interruption between calls. Restarted
sessions rehash retained parts from the beginning with bounded memory.

`import` is a convenience over these operations for a local `Read` stream with
a known expected particle and length. It rejects short and trailing input and
leaves durable progress on failure. The lower-level interfaces expose individual
bounded steps for schedulers and transport adapters.

## read and retain

`info` returns a sealed descriptor in the requested namespace. `read_range`
validates local part checksums within a bounded allocation. `reader` implements
`Read` and `Seek` with one cached physical part. These local integrity checks
are distinct from a transferable proof authenticating a partial file.

`commit_with_blobs(proposal, blobs, validate)` requires sorted, unique Blob
particles sealed in the proposal's namespace. The list and verifier profile
are bound into the request fingerprint. Blob references can satisfy
`proposal.required` and structural children. The application head remains a
small ordinary graph Content value referencing its streamed files.

The usual validation callback and conditional head checks apply. BBG records
each file's retention obligation under the committed application root in the
same transaction as history, head and retry receipt. Failure publishes none of
them; identical retries return the original head even after successors. A changed
Blob list under the same request conflicts. A namespace label supplies scope;
the trusted host provides authorization before exposing any file operation.

## lifecycle boundaries

`cancel` reclaims unselected staging parts in bounded pages and retains a request
tombstone. Canonical sealed files remain protected. Releasing sealed content,
retention-aware GC and remote replication are separate implementation packages.
Existing application-only archives and namespace migration must reject streamed
content they cannot carry; successful export cannot silently omit file parts.

[[fs]] names, channel/patch metadata and private [[vault]] records use these
shared storage primitives through their application contracts. This adapter
does not itself implement filesystem name resolution or a public network API.
