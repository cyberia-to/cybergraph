---
title: local application graph sessions
tags: cybergraph, cell, spec
status: implementation
---
# application graph sessions

The local-storage feature exposes application namespaces over BBG's atomic
ApplicationStore. Content identity uses the existing nox Model B encoding for
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
