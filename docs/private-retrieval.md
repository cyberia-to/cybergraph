---
tags: cyber, cybergraph, inf, privacy, recovery
crystal-type: explanation
crystal-domain: cyber
status: design
date: 2026-09-12
alias: verifiable private retrieval, private wallet recovery
---
# verifiable private retrieval

Private retrieval is a standard capability of Cyber's read path: a client asks
for encrypted records over committed history and verifies that the requested
computation covered the complete declared scope. Wallet recovery is its first
application; private inboxes, notifications and agent subscriptions share the
same service boundary.

The service composes existing responsibilities: Cybergraph exposes the snapshot,
Inf defines the query, BBG authenticates its input domain, Mudra supplies private
retrieval operations, and Zheng proves their execution. Cyber runs the service
locally or delegates it to compatible workers. UnifOMR is a candidate retrieval
profile within this architecture.

This note records the architectural direction agreed on 2026-09-12. Concrete
wire encodings, cryptographic parameters and deployment limits are selected by
their owning specifications before implementation.

## why this belongs in the read path

[Inf's proof contract](../../inf/specs/proof.md) already asserts
`result = eval(query, committed_state)`, including completeness. A complete
private recovery result is an application of that contract. The additional
work is to express the encrypted computation and authenticate its entire input
domain while preserving the retrieval protocol's privacy.

The [UnifOMR assessment](../../mudra/audit/signature-optimality/unifomr-evidence.md)
identified a distinction between privacy against a malicious detector and
complete recovery from that detector. Cyber supplies the natural integration
point for an execution proof. The remaining engineering questions concern the
exact relation, private-protocol composition and its measured cost.

## component ownership

| owner | responsibility |
|---|---|
| Cybergraph `expose` | Snapshot-scoped query service, relation/view definitions, request/response binding and verification handoff |
| BBG / Lens | Committed records, domain sizes, authenticated range/index coverage and state roots |
| Inf | Complete relational evaluation, scope and termination semantics, query plan and cost bounds |
| Mudra | Note encryption, clue/detection-key formats, homomorphic detection and PIR profiles, privacy/error parameters |
| Zheng and execution backends | Proof of the exact declared computation over authenticated inputs, with the declared disclosure profile |
| Cyber / Soft3 / workers | Product capability, configuration, scheduling, resource limits and local or remote execution through shared interfaces |
| Foculus / network verification | Canonical history, checkpoint/finality and availability policy |
| cyb / wallet | Keys, local candidate decoding, proof verification, durable recovery cursor and spendable-state reconstruction |

Cybergraph keeps its existing role as the read boundary and verifier caller;
cryptographic algorithms and proof construction remain in their component
libraries. A full node can host a retrieval worker. An archival or specialized
provider can execute the same contract on another machine. Delegation changes
placement and capacity while retaining the client's verification rules.

## the statement to prove

The client pins a statement containing at least:

```text
network / genesis and accepted checkpoint
BBG root and authenticated notification-board descriptor
epoch or contiguous record range, with its committed cardinality
query / executable identity and cryptographic profile version
bindings to this request's encrypted keys and encrypted query
binding to the complete encrypted response and its declared status
```

These names describe semantic fields rather than a frozen wire format. A board
descriptor commits to the ordering, size and clue/payload association. If the
board is a derived view, its complete derivation from the BBG snapshot must also
be authenticated. A server-selected subset of valid records cannot stand in
for the requested board.

For an UnifOMR-shaped profile, the two execution statements are schematically:

```text
board = complete_view(BBG_state_at_root, requested_scope)
encrypted_digest = Detect(profile, board.clues, encrypted_detection_key)
encrypted_answer = PIR.Answer(profile, board.payloads, encrypted_query)
```

The proof binds each output to the corresponding request and the same board.
Canonical preprocessing, packing, modulus switching and any randomness follow
the chosen algorithm's specified semantics. Intermediate indexes or transformed
databases carry a proof of complete derivation from that board; outsourcing
them as an unchecked host witness would prove only a conditional result.

The server can prove these ciphertext computations without the recipient's
decryption key or plaintext matching indices. The recipient decrypts the first
response locally, issues the padded PIR query, verifies the second response,
and authenticates the recovered payloads. Public proof inputs use the profile's
permitted disclosures and request bindings; the protocol must avoid introducing
a permanent recipient identifier or exposing the private PIR indices.

## completeness over a declared scope

For each request, the proof establishes all three properties:

1. The input is the entire declared domain at the pinned root. Coverage can use
   full authenticated enumeration, complete range proofs, or a sound indexed
   selection argument that also accounts for excluded records.
2. The declared program evaluates that domain exactly. Valid point openings
   establish individual row authenticity; domain coverage establishes that the
   evaluation includes every required row.
3. The response commitment binds the exact result, including order, length,
   pagination or overflow status. An authenticated empty response still proves
   evaluation of the complete requested scope.

Partitions can distribute the work. Their proof composition must establish
contiguous coverage without gaps or double counting, one snapshot, and exact
combination of outputs. A client checks an uninterrupted sequence of epochs
from its chosen recovery start through the claimed endpoint.

Complete execution preserves the chosen algorithm's own error probabilities.
UnifOMR's false negatives, false positives, PIR capacity and decoding failures
therefore retain an explicit whole-recovery error budget. A valid proof of a
truncating algorithm would still allow lost notes; an accepted recovery profile
needs sufficient capacity and a privacy-preserving overflow protocol. Sender
clue validity and durable clue/payload availability also belong in that profile.

## node and client behavior

The product exposes this as a named read capability with a versioned profile,
available history, maximum scope, proving limits and estimated cost. The node
may execute it through an embedded worker or route it to a configured provider.
A pruned node can advertise its actual history and a delegation option. Every
provider's response is checked against the same caller-pinned statement.

A request can finish with a verified complete response, a bounded partial
response with authenticated continuation, or an explicit unavailable/failed
status. Only a complete response advances the corresponding complete-recovery
cursor. The client persists that cursor together with accepted results and the
canonical root so interruption or reorganization can resume or roll back safely.

Verified retrieval establishes the notes found in the requested history. A
spendable balance additionally authenticates inclusion, nullifier/spent status,
policy and current membership witnesses. Those can use further proven queries
against the appropriate accepted state. The node's recovery service composes
these queries; the wallet advances its balance only after their requirements
are satisfied.

Proofs make incorrect completed responses rejectable. Availability, timeouts
and refusal require replication, retries and durable archives. Query privacy
also covers the composition: proof fields, verification failures, retry timing,
capacity and fallback behavior must fit an explicit leakage policy. A proof of
correct server computation addresses integrity; recipient privacy requires the
combined retrieval/proof protocol to retain its own argument.

## delivery and acceptance

Work is tracked as [C2.1 in the node roadmap](../../cyber/roadmap/c-network.md#c21-verifiable-private-retrieval).
It depends on authenticated query execution and canonical synchronization, and
reuses the normal worker/proof path. Readiness evidence belongs in `audit/`.

Acceptance requires a ground-truth local scan and an independent verifying
client, covering:

- Identical recovered notes and spent state for full restore and incremental
  catch-up over the same declared history and key scope.
- Rejection of omitted or duplicated records, gaps between proven partitions,
  wrong cardinalities, substituted roots/programs/keys/queries and forged empty
  responses.
- Authenticated PIR output from the same board, including preprocessing/index
  derivation, rather than a proof of only the detection phase.
- Defined handling of bad clues, detection errors, capacity overflow and
  incomplete history; complete status follows the selected recovery contract.
- Restart and reorg recovery without skipping a range or keeping invalid spent
  state; identical verification when workers run locally or remotely.
- Measured prover time/RAM, server cost per client, bootstrap material,
  communication and client verification on declared targets. Proving lattice
  and PIR arithmetic can dominate cost even when the statement is straightforward.

## insight trace

1. [UnifOMR, ePrint 2026/910](https://eprint.iacr.org/2026/910) supplies the
   candidate private-discovery construction and its stated integrity boundary.
2. [Mudra's assessment](../../mudra/audit/signature-optimality.md#unifomr-what-the-new-paper-changes)
   records paper/version evidence, recovery costs and the completeness gap.
3. The owner connected that gap to Inf's existing complete-query semantics on
   2026-09-12 and selected ordinary node or adjacent infrastructure as its home.
4. This note records that composition, linked from Cybergraph's
   [query](../specs/query.md) and [expose](../specs/expose.md) contracts and
   [Inf's proof contract](../../inf/specs/proof.md#complete-input-coverage).
5. The [node roadmap](../../cyber/roadmap/c-network.md#c21-verifiable-private-retrieval)
   carries the implementation gates; [Mudra's follow-up](../../mudra/audit/signature-optimality.md#cyber-integration-follow-up-2026-09-12)
   records the architectural interpretation alongside the original research.
