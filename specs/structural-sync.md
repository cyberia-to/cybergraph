---
tags: cyber, soft3, protocol, cross-cutting
crystal-type: spec
crystal-domain: cyber
---
# structural sync

one mechanism at three scales. a [[signal]] is the universal unit of state change — created on a neuron, merged between neurons, submitted to the network, queried by clients. the signal structure is identical at every scale. only the merge function changes.

this is a **cross-cutting protocol**. it spans five repos — no single component owns it. each layer maps to a specific owner:

| layer | mechanism | owner |
|-------|-----------|-------|
| 1 validity | zheng proof per signal | [[cybergraph]] — validates before storing |
| 2 ordering | hash chain + VDF + Merkle clock + step | [[cybergraph]] — signal lifecycle, `SignalChain` |
| 3 completeness | Lens per source | [[bbg]] — serves opening proofs against BBG_root |
| 4 availability | algebraic DAS + erasure coding | [[radio]] / [[cyb/sync]] — content distribution |
| 5 merge local | CRDT G-Set | [[cyb/sync]] — device-level convergence |
| 5 merge global | foculus φ* convergence | [[foculus]] — network-level finality |

layers 1-4 are identical at all scales. layer 5 is the only scale-dependent component.

## repo boundaries

**[[bbg]]** is a pure state library: `apply(tx)` → state transition, `prove(key)` → Lens opening, `root()` → BBG_root. no network, no sync, no signal structure. bbg is to cybergraph what a database engine is to a schema: **cybergraph defines WHAT, bbg implements HOW**.

**[[cybergraph]]** owns the signal lifecycle: receives signals, validates (layers 1–2), calls `bbg.apply()`, routes events to tru/glia/mir. it owns `Signal`, `SignalChain`, hash chain, equivocation detection, and the local sync protocol. signal ordering logic does not belong in bbg.

**[[cyb/sync]]** is layer 4 only: erasure-coded blob storage across a neuron's device set. no signal awareness, no BBG_root knowledge. called by cybergraph when distributing content chunks.

**[[radio]]** is pure transport: QUIC, BAO, gossip. no semantic awareness.

**[[foculus]]** is layer 5 global: φ* convergence, finality from topology.

## signal lifecycle

```
creation          neuron creates cyberlinks → signal batch with zheng proof
                  signal = (ν, l⃗, Δφ*, σ, prev, mc, vdf, step)

ordering          hash chain (prev) establishes per-neuron sequence
                  VDF proves minimum wall-clock time since previous signal
                  Merkle clock captures full causal state
                  step counter provides monotonic logical clock

completeness      per-neuron polynomial commits signal set — withholding detectable
                  same guarantee at local and global scale

availability      algebraic DAS + 2D Reed-Solomon erasure coding
                  any k-of-n chunks reconstruct original
                  O(√n) Lens opening samples verify availability

merge             LOCAL:  CRDT (G-Set union) — 1-20 neurons, same identity
                  GLOBAL: foculus (φ* convergence) — 10³-10⁹ neurons, different identities

finalization      network includes signal in block → assigns t (block height)
                  signal enters BBG_poly(signals, step, t)
                  state transitions applied to BBG_poly evaluation dimensions

query             client requests namespace from BBG_root
                  Lens opening proof — provably complete response
```

## three scales

| | local | global | query |
|---|---|---|---|
| **who** | 1-20 neurons (same identity) | 10^3-10^9 neurons (different identities) | light client ↔ peer |
| **direction** | bidirectional merge | neuron → network | pull (client reads) |
| **data** | private cyberlinks, files, names | signals (public aggregate) | BBG state (public) |
| **privacy** | private (individual records) | public (aggregate) | public (Lens proofs) |
| **trust** | same identity, semi-trusted | different identities, untrusted | peer untrusted, only BBG_root |
| **merge** | CRDT (G-Set union) | foculus (φ* convergence) | N/A (read-only) |
| **ordering** | VDF + hash chain + Merkle clock | VDF + hash chain + Merkle clock | block height (t) |
| **statefulness** | ongoing DAG | ongoing accumulator | stateless query-response |
| **signal structure** | identical | identical | N/A (queries BBG_root) |

the unit is always neuron. local sync = small group of neurons (1-20, same identity). global sync = large group of neurons (10^3-10^9, different identities). the five verification layers apply identically. only the merge function varies.

## signal structure

the [[signal]] is s = (ν, l⃗, Δφ*, σ, prev, mc, vdf, step, t). the ordering fields are part of the signal — not a separate envelope. the same fields serve local sync and global [[foculus]] consensus.

```
signal = {
  // payload — what the signal means
  ν:              neuron_id                   subject (signing neuron)
  l⃗:              [cyberlink]                 links (L⁺), each a 7-tuple (ν, p, q, τ, a, v, t)
  Δφ*:            [(particle, F_p)]*          impulse: sparse focus update
  σ:              zheng_proof                 recursive proof covering impulse + conviction

  // ordering — where the signal sits in causal and physical time
  device:         device_id                   which instance within ν (local sync only)
  prev:           H(author's previous signal) per-neuron hash chain
  merkle_clock:   H(causal DAG root)          compact causal state
  vdf_proof:      VDF(prev, T)               physical time proof
  step:           u64                         monotonic logical clock

  // finalization
  t:              u64                         block height (assigned at network inclusion)

  hash:           H(all above)
}

lifecycle:
  created on neuron:    (ν, l⃗, Δφ*, σ, prev, mc, vdf, step)
  synced between peers: full signal
  submitted to network: full signal (ordering fields preserved)
  included in block:    network assigns t (block height)

signal size: ~2 KiB proof + impulse + 160 bytes ordering metadata
```

## five verification layers

every signal passes five layers. all layers apply at both local and global scale.

```
layer           mechanism               guarantee               owner
─────           ─────────               ─────────               ─────
1. validity     zheng proof per signal  invalid state → rejected  cybergraph
2. ordering     hash chain + VDF        reordering → prevented    cybergraph
3. completeness Lens per source         withholding → detectable  bbg
4. availability algebraic DAS + erasure data loss → recoverable   radio/cyb-sync
5. merge        CRDT or foculus         convergence → deterministic  cyb-sync/foculus
```

### layer 1: validity

```
each signal carries σ = zheng proof

the proof covers:
  - cyberlinks well-formed (7-tuple structure)
  - focus sufficient for conviction weight
  - impulse Δφ* consistent with cyberlinks
  - neuron signature valid

invalid signal → proof verification fails → signal rejected by any peer
cost: ~5 μs verification (zheng)
owner: cybergraph validates before calling bbg.apply()
```

### layer 2: ordering

four mechanisms establish temporal structure without consensus. owned by cybergraph — `SignalChain` lives there, not in bbg.

**hash chain** — each neuron chains signals via the `prev` field:

```
neuron's chain: s1 ← s3 ← s5 ← s8
  prev(s3) = H(s1)
  prev(s5) = H(s3)

properties:
  immutable: cannot insert, remove, or reorder past signals
  verifiable: any peer can walk the chain and verify continuity
  fork-evident: two signals with same prev = cryptographic equivocation proof
```

**VDF** — physical time without clocks:

```
signal.vdf_proof = VDF(prev_signal_hash, T_min)

T_min: minimum sequential computation between signals
proves: "at least T_min wall-clock time elapsed since prev signal"
no NTP, no clock sync, no trusted timestamps

rate limiting:  flood of N signals costs N × T_min sequential time
fork cost:      equivocation requires computing VDF twice from same prev
```

**Merkle clock** — causal history as Merkle DAG:

```
each signal's merkle_clock = H(root of all signals the neuron has seen)

comparison:   O(1) — single hash comparison (equal = in sync)
divergence:   O(log n) — walk DAG to find first difference
merge:        union of both DAGs → H(merged root) — deterministic
```

**step** — monotonic logical clock:

```
gap-free counter per source
gap in step sequence = missing signal = detectable
```

**deterministic total order:**

```
1. causal order:   A in B's deps → A before B
2. VDF order:      A.vdf_time < B.vdf_time (if not causally related) → A before B
3. hash tiebreak:  concurrent signals same VDF epoch → H(A) < H(B) → A before B

no negotiation, no leader, no timestamps
```

### layer 3: completeness

owned by bbg. two distinct signal commitments at different scales:

**per-neuron signal commitment (local):** each neuron commits its own signal chain to a local polynomial indexed by step. per-neuron Lens commitment used in the sync protocol — not part of BBG_poly.

**BBG_poly(signals) (network-level):** the signals dimension of BBG_poly is the finalized index maintained by validators after block inclusion.

```
LOCAL (per-neuron):
  each neuron commits its signal chain via its own Lens commitment
  proves: "these are ALL signals from source S in step range [a, b]"
  Lens binding: source cannot hide a signal in the requested range

NETWORK (BBG_poly dimension):
  BBG_poly(signals, step, t) = finalized signal batch at step
  maintained by validators after block inclusion
  proves: "signal batch at step S was accepted at block height t"
```

### layer 4: availability

owned by radio / cyb/sync.

```
2D Reed-Solomon erasure coding over Goldilocks field.

  original data: √n × √n grid
  extended: 2×(√n × √n) with parity rows and columns
  any √n × √n submatrix sufficient for reconstruction

sampling: O(√n) random cells with Lens openings (algebraic DAS)
  ~1.5 KiB for 20 samples
  O(1) field verifications per sample
  99.9999% confidence at 20 samples
```

### layer 5: merge

the only scale-dependent layer.

**local (CRDT) — cyb/sync:**
```
mechanism: G-Set union (grow-only set of signals)
  commutative, associative, idempotent

conflict resolution for mutable state (names):
  1. causal order (A in B's deps → A before B)
  2. VDF time (lower VDF → earlier)
  3. hash tiebreak (H(A) < H(B) → A before B)
  → deterministic total order → replay produces identical state
```

**global (foculus):**
```
mechanism: φ* convergence (stake-weighted attention)
  φ* = stationary distribution of tri-kernel (diffusion + springs + heat)
  convergence: φ* stabilizes to fixed point in 1-3 seconds
  manipulation costs real tokens (focus)
```

## local sync protocol

two neurons of the same identity reconnect. owned by cybergraph.

```
1. COMPARE merkle_clock roots                    O(1)
   equal → done (already in sync)
   different → continue

2. EXCHANGE signal polynomial commitments          O(1)
   each neuron sends its current signal commitment (Lens)

3. REQUEST missing step ranges                   O(1) per range
   with Lens batch opening proofs
   → provably ALL signals in range received
   → no withholding possible

4. DAS SAMPLE content chunks                     O(√n)
   algebraic DAS — Lens openings per sample (~200 bytes each)
   verify content availability

5. VERIFY each received signal:
   a) zheng proof valid?                         layer 1: validity
   b) hash chain intact? (prev links)            layer 2: ordering
   c) no equivocation? (no duplicate prev)       layer 2: ordering
   d) VDF proof valid?                           layer 2: ordering
   e) step counter monotonic?                    layer 2: ordering
   f) Lens opening proof valid?                  layer 3: completeness

6. MERGE signal DAGs                             O(signals)
   compute deterministic total order (CRDT)

7. REPLAY ordered signals → bbg.apply(tx)        O(signals)
   apply state transitions → identical BBG_root

FAST SYNC (snapshot available):
   find most recent snapshot step in common
   replay only signals after snapshot
```

## global sync: signal submission

```
1. neuron creates signals across local instances
2. local neurons sync (protocol above)
3. neuron submits finalized signals to network
4. network verifies (layers 1-4)
5. foculus merges (layer 5): φ*-weighted convergence
6. block producer includes signals → assigns t (block height)
7. state transitions applied to BBG_poly evaluation dimensions
8. BBG_root = H(Lens.commit(BBG_poly) ‖ Lens.commit(A) ‖ Lens.commit(N)) updated
```

## query sync protocol

light clients and full nodes query BBG state. read-only. served by cybergraph over the query wire protocol; proofs built by bbg.

**outgoing axons from particle P:**
```
client → peer: (axons_out, key=P, state_root=BBG_root)
peer → client: Lens batch opening + all axon entries for P
client verifies: Lens.verify(BBG_root, (axons_out, P, t), entries, proof)
guarantee: "ALL outgoing axons from P. nothing hidden."
proof size: ~200 bytes
```

**incoming axons to particle Q:**
```
client → peer: (axons_in, key=Q, state_root=BBG_root)
peer → client: Lens batch opening + all axon entries for Q
```

**neuron public state:**
```
client → peer: (neurons, key=N, state_root=BBG_root)
peer → client: Lens opening + neuron data (focus, karma, stake)
```

**particle data:**
```
client → peer: (particles, key=P, state_root=BBG_root)
peer → client: Lens opening + particle data (energy, φ*, axon fields)
```

**state at time T:**
```
client → peer: (index, key, t=T, state_root=BBG_root)
peer → client: Lens opening at (index, key, T)
guarantee: "authenticated state at time T."
```

**incremental sync:**
```
client has state at height h₁, wants updates through h₂:

1. REQUEST time range [h₁, h₂] with BBG_root at h₂
2. RESPONSE: polynomial update deltas + batch Lens opening at height h₂
3. VERIFY: batch Lens opening against BBG_root at h₂

data transferred: O(|changes since h₁|)
```

## light client protocol

```
new client joins with no history:

1. OBTAIN checkpoint = (BBG_root, folding_acc, height) from any peer
   ~232 bytes

2. VERIFY folding_acc:
   final_proof = decide(folding_acc)        zheng decider
   verify(final_proof, BBG_root)
   → ONE verification proves ALL history from genesis valid
   → cost: ~5 μs

3. SYNC namespaces of interest

4. MAINTAIN:
   - fold each new block into local folding_acc (~30 field ops)
   - update Lens proofs for monitored namespaces

join cost:     ONE zheng verification + namespace sync (~200 bytes per namespace)
ongoing cost:  O(1) per block
storage:       O(|monitored_namespaces| + |owned_records|)

trust: only BBG_root (from consensus). peer is completely untrusted.
```

## content sync

file blobs are content-addressed [[particles]]. owned by cyb/sync + radio.

```
layer           mechanism       guarantees
─────           ─────────       ──────────
merge           CRDT (G-Set)    convergence, commutative, idempotent
completeness    Lens opening    provable completeness, withholding impossible
availability    algebraic DAS   data survives neuron failure, O(√n) verification
```

## fault handling

```
fault class          mechanism               guarantee
═════════════        ═════════               ═════════

FORGING              zheng proof per signal  proof fails → rejected
EQUIVOCATION         hash chain + VDF        two signals same prev → misbehavior proof
REORDERING           hash chain              prev hashes break → detectable by any peer
WITHHOLDING          Lens + algebraic DAS    Lens completeness proof → withholding detectable
FLOODING             VDF rate limiting       each signal costs T_min wall time
COMPROMISED NEURON   key revocation signal   future signals rejected; past signals immutable
STALE NEURON         snapshot + fast sync    reconnect → find common snapshot → replay
```

## polynomial state summary

BBG_root = H(Lens.commit(BBG_poly) ‖ Lens.commit(A) ‖ Lens.commit(N))

every public query is a Lens opening against BBG_poly. every private query is a Lens opening against A(x) or N(x). every verification is O(1) field operations.

light client join: < 10 KiB total.

---

see [[bbg]] for state structure and polynomial commitments. see [[cybergraph]] for signal validation and local sync implementation. see [[foculus]] for global merge. see [[radio]] for transport. see [[cyb/sync]] for device-level content availability.
