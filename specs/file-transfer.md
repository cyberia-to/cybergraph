---
title: BBG file transfer through Radio
tags: cybergraph, radio, storage, spec
status: implementation
---
# BBG file transfer through Radio

The `cybergraph-radio` adapter in `radio/` composes the
[local file API](files.md) with [[radio/specs/file-stream|Radio file streams]].
Cybergraph's core library retains its existing transport-independent build.
The adapter depends on the existing Radio transport and local Cybergraph API;
the transport library receives storage capabilities.

`FileSource::open` binds a sealed Blob in a local namespace. The host's Provider
authorizes the remote endpoint before opening that source. The source exposes
only its descriptor and bounded reads. The namespace stays local.

`FileSink::open` binds an existing BBG upload. Its descriptor and physical part
size come from persisted progress. Each accepted range must be exactly one
physical part at its declared offset. Disk work runs outside the async network
executor through bounded blocking jobs. Dropping an await may leave its current
bounded job completing; restart discovers the outcome through the same request.

`receive_page` inspects a bounded page of durable coverage and requests only its
missing parts. It returns a cursor for the next page; there is no total-file
or collection limit. This operation is the mechanism a Foculus scheduler can
call. Peer selection, automatic retries, replica policy and advertisements
remain scheduler responsibilities.

Transport interruption leaves completed parts durable. A new endpoint/client
and the same upload can resume. Already present parts are staging bytes until
`seal` checks complete Blob identity through the existing verifier, using bounded
steps. Failed verification leaves the file unpublished; discarding invalid
staging requires explicit cancellation and a new upload request.

`seal` establishes local verified availability. Application publication uses
`ApplicationGraph::commit_with_blobs` to establish retention with the head and
retry receipt. This adapter emits no remote protection receipt and makes no
device-loss guarantee by itself. Private content arrives as ciphertext under
the application's custody contract. Transport encrypts requests in flight;
size, timing and access patterns remain part of the privacy profile.
