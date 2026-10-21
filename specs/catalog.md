---
title: local versioned file names
tags: cybergraph, fs, spec
status: implementation
---
# local versioned file names

`Catalog` binds relative paths to sealed files within one application namespace.
It borrows the existing `ApplicationGraph`; its immutable pages, selected head,
history and retry receipts use the same BBG database owner and selected backend.
[[soft3/specs/storage]] owns the cross-component storage contract and
[[fs/patch/spec]] owns the wider filesystem patch/channel model.

Paths are exact UTF-8, case-sensitive relative paths separated by `/`. Empty
segments, `.`, `..`, NUL, leading and trailing separators are rejected. Unicode
normalization is explicit caller work. Each path has a 4096-byte operation
budget. Collections and retained history have no fixed total-count limit.

A binding contains its creation identity, payload particle and content revision
identity. Creation and edit require the payload to be sealed in this same
namespace. Rename moves the binding atomically, preserves all three identities
and requires an absent destination. Remove withdraws the selected name.
Earlier states and their retained payloads remain readable.

Every mutation supplies a request identity and the exact expected application
head. Edit, rename and remove also supply the resolved binding. A stale head,
changed binding or occupied destination conflicts. A concurrent path change
cannot redirect a checked operation. Identical retries return the original
head; changed requests conflict. The operation record includes namespace,
expected head and all operation fields. Creation uses that immutable record's
particle as binding and revision identity; edit advances only the revision and
payload. Rename leaves the content revision unchanged.

The catalog root binds namespace, history index, previous head, operation record
and an immutable ordered binary radix tree. Leaves contain paths and bindings;
branches contain the first differing bit position and two child particles.
The ordered path encoding uses a one-bit continuation marker before each UTF-8
byte and a zero-bit terminator, preserving bytewise path order. Updates replace
only the search paths, sharing unchanged content. Branches and leaves are
existing `Content::Blob` values. Required application references make their
availability part of the conditional publication, and `commit_with_blobs`
retains newly published payloads with the same head transaction.

Reads select an explicit immutable head, with `None` selecting the empty state.
`head` obtains the current selection. `resolve` reads one binding; `list` returns
bounded lexicographic pages. A continuation names the last path returned in
that same state and must still resolve there. A caller keeps the head across
pages. `history` pages the namespace's application head records. Roots verify
their namespace and index on read. Names and operation metadata stay in the
private application store; callers provide namespace authorization.

This contract covers trusted local conditional publication and retained local
history. Signed remote patches, aliases, channel fork/merge, concurrent patch
conflict objects, directory-prefix indexes, retention release and content-aware
device migration remain separate integration work. Application heads retain
their existing meaning and do not substitute for neuron SignalChain steps.
