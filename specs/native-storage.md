---
title: durable native acceptance
tags: cybergraph, storage, spec
---
# durable native acceptance

The local chaosnet profile has one coordinator over BBG's shared Database.
Fjall is the SSD authority and RAM is the published working state. Native
history, exact BBG records, economics, block metadata and request receipts
share one durable transaction. Genesis bytes and the execution profile are
pinned when the store is created. A different genesis or profile prevents open.

## requests and publication

A request contains a stable 32-byte identity and a canonical operation. Link
and pay commands bind normalized identities and amounts before a chain step
is allocated. Native events bind every encoded field, including destination,
proof, height and collection lengths. Receipts are checked before economic or
ordering validation. A matching retry returns its original receipt; different
content with the same identity is a conflict. A caller omitting identity gets
a distinct server identity for each submission and must retain the returned
identity for safe retries.

One request contains at most 64 native events. Every event is validated and
prepared under exclusive ownership. Signal events each finalize one block;
intent events persist without finalizing. An invalid event rejects the entire
request. Prepared BBG state is inaccessible to readers and subscribers until
the database confirms durability. Preparation uses an undo set of touched
records, including pruning changes, rather than cloning the graph.

The transaction stores the complete operation, resulting BBG records, ledger
changes, monotonically increasing request position, per-block metadata and
original receipt. Publication after commit is infallible. Failed preparation
or commit restores the working state. An unknown commit outcome freezes the
coordinator; reopening resolves receipts before readiness. No callback or
successful HTTP response precedes the commit.

Signal headers use a global history index, independent of each neuron's chain
step. Proof hashes bind the retained proof encoding. The local profile
preserves authenticated proof bytes; its acceptance does not assert proof
verification, identity authorization or network consensus finality.

## economics

The versioned local chaosnet rule processes matching `zheng` to `pussy` links
as test subsidy, then transfers in signal order. Subsidy and transfer sums
use checked arithmetic. A native insufficient transfer retains the historical
skip rule; a pay command rejects zero amounts or insufficient funds before
creating a signal. JSON and native admission use the same rule. Balances,
supply and block observations are durable records. Observation time is supplied
by the host and excluded from request identity.

## recovery and legacy import

Open strictly replays contiguous committed operations using their pinned
profile, checks every receipt and block root, then compares the entire exact
BBG record set and ledger with disk. Missing, extra, malformed or inconsistent
records prevent readiness. No record is skipped. Recovery does not acknowledge
new operations or emit events.

Legacy tape import is explicit. The complete input must decode strictly before
copying; historical complete encodings are accepted and malformed, unknown or
truncated frames fail. The import pins a source digest and leaves a durable
in-progress marker until all source events and final state are checked. An
interrupted import can resume only with the same source. Normal open rejects
an incomplete import. The source remains unchanged.

Legacy frames do not contain network or proof: import uses their historical
SELF_NETWORK and absent-proof profile. Their original bytes form the unchanged
prefix of the compatibility byte-offset log. New full-fidelity native signals
use Foculus's versioned codec. A compatibility export refuses to discard a
network or proof that its old encoding cannot represent.

Legacy imports produce the current profile's root. In particular, globally
indexed signal headers replace the historical per-neuron-step collisions;
an old multi-neuron root is not a commitment to the corrected state. Unknown
historical observation time is stored as zero. Intent bytes are exact durable
records; the current BBG root format does not include its intent map.

## canonical command wire

`encode_operation` and `decode_operation` define the bounded command wire also
returned by native history export. The prefix is ASCII `CGOP` followed by byte
1 and a kind byte. Integers are little endian; particles are exact 32 bytes.
Kind 0 carries neuron/from/to/token, amount u64 and valence i8. Kind 1 carries
sender/recipient and amount u64. Kind 2 carries an event count u32, followed by
events: tag 0 plus a u32 byte length and a complete Foculus signal, or tag 1
plus neuron, inception height u64, scope hash and 64 signature bytes.
The total command is at most 4 MiB and contains at most 64 events. Empty event
batches, unknown tags/versions, truncated records and trailing bytes fail.

The command fingerprint is Hemera of `cybergraph/native-command/v1\0`, the
instance and command; each part has a u64 length prefix. Instance identity
binds the exact canonical genesis and profile bytes. The HTTP adapter accepts
the returned 64-hex-digit request identity directly; other explicit textual
IDs use the adapter's documented domain-separated mapping.

This contract covers local SSD durability. Archive tier replication, power-cut
qualification, authorization and distributed consensus have separate release
criteria.
# Local host credit events

The local unsigned profile also records explicit `LocalCredit` events containing
neuron, token, amount, focus increment and reason. They are host-only economic
adjustments, not peer wire events or consensus mint proofs. The caller verifies
settlement authorization before admission. Request IDs bind the reason and
recipient; exact retries return the receipt, conflicting reuse rejects. Public
balance/focus changes, refreshed root/checkpoint, history and request receipt
publish in the same transaction. Overflow rolls back. No new block is implied.
Legacy tape export stops before such an event, requiring full native history for
complete state recovery. A/N contents and native chaosnet subsidy ledger remain
separate. Existing signal/intent encodings are unchanged; event kind 2 encodes
the bounded local credit fields. Old readers reject the unknown kind.

`from_database(Database,genesis)` reuses the shared storage owner; `database()`
returns a clone for application namespaces. It does not create a second native
writer. A stale independently opened coordinator fails its existing head CAS.
