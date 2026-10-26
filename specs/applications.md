---
title: local application graph sessions
tags: cybergraph, cell, spec
status: implementation
---
# application graph sessions

Semantic migration uses `commit_migration`: the proposal fingerprint additionally
binds the manifest and sorted source namespaces/exact heads. BBG publishes the
target write and source writer fences atomically. Read/export and exact historical
request resolution stay available. Content/schema/authority checks precede this
transaction; mutable source-head validation and fence activation occur inside it.

The local-storage feature exposes application namespaces over BBG's atomic
ApplicationStore and enables its Fjall SSD backend. ApplicationGraph::open
accepts a database directory. ApplicationGraph::from_database accepts an
already opened BBG Database, sharing its writer lock, failure state and backend
with other views. Database and Backend are re-exported by this module; Backend
selects Ssd or Hdd explicitly when the corresponding BBG feature is enabled.
Backend selection belongs to the database owner. Application records and
polynomial shards retain distinct schemas within the shared transaction engine.
Sharing an owner does not itself combine separate calls into one transaction.

Content identity uses the existing nox Model B encoding for
data nodes (8-byte field, 64-byte child particles) and hemera byte hashing for
blob artifacts. A stored framing byte selects the codec and is outside content
identity. Noncanonical field limbs in child particles are rejected.

Commit validates bounded content, identities, required roots and every new
node's child availability. The application's supplied validator checks its
schema, lifecycle and policy. BBG conditionally commits content, history, request
fingerprint, optional store-wide unique claims and selected head together. Claims
bind application identities across namespaces without introducing schema logic
into BBG. Unchanged retries return the original
head; changed requests conflict. Local graph reads recheck content identity.

The process opening the exclusively locked local database is the trusted local
authority. Application adapters authenticate inputs and enforce ward policy.
These receipts advertise local-authority selection and local durability. They
do not advertise signed remote authorship or append a public SignalChain. Network
publication remains through cybergraph's existing signal/writer protocol; a
local application head is never substituted for a neuron SignalChain step.

History range reads have explicit limits. Content reads are limited to 16 MiB;
commit batches to 131072 entries and 16 MiB encoded content. Required references
are retained; the initial implementation performs no garbage collection.
The content store is private to the local authority. Exposing queries to remote
readers requires a namespace/disclosure adapter and is unsupported by this API.

## legacy import

The optional legacy-redb-migration feature enables the HDD backend in addition
to local-storage. ApplicationGraph::migrate_redb(source, destination) imports
an old application redb file into a fresh Fjall directory through BBG's bounded
migration API. The source is retained. Content, conditional heads, history,
global claims and request fingerprints retain their identities. Incomplete
destinations cannot be opened as successful sessions; completion is recorded
only after import succeeds. Normal open never performs implicit migration or
overwrites an existing file of another format.
# Fresh publication

`commit_fresh` validates the canonical proposal like `commit`, and requires a
fresh request at BBG's atomic boundary. Identical prior receipts conflict. It is
used before an external dispatch where replaying a successful commit receipt
must never execute a callback again. This changes no content or hash encoding.
