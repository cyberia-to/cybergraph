# validation

cybergraph is the validation layer between the network and [[bbg]]. it enforces the six axioms of [[cybergraph]] before forwarding a signal to bbg for state insertion. bbg never sees an invalid signal.

## component contract

```
cybergraph.submit(signal) -> Result<Cid, ValidationError>:
  1. validate(signal)    ← all checks below
  2. bbg.insert(signal)  ← update polynomial state, returns new BBG_root
  3. return BBG_root

bbg.insert(signal) -> Result<Cid, DoubleSpend>:
  ← only structural check: N(nullifier) = 0 → reject
  ← all semantic rejection happens before this call
```

## validation checks

### A1 — content addressing

particle CIDs in ℓ⃗ match H(particle_bytes). a CID that does not resolve to its
preimage fails here. cybergraph caches resolved CIDs; unknown CIDs are fetched
before validation.

### A2 — authentication

signal carries a valid zheng proof σ covering the entire batch atomically:

- valid signature from ν over H(ℓ⃗ ‖ Δφ* ‖ t)
- correctness of each cyberlink ℓ ∈ ℓ⃗ (valid particle references)
- validity of all conviction UTXO movements (each (τ, a) spend is backed by unspent output)
- correctness of Δφ* (tri-kernel computation against current BBG_root from latest header)

one proof covers everything. verification: `decide(σ)` in O(log n) where n = |ℓ⃗|.

### A3 — append-only / equivocation detection

signal height t must equal current block height. signals at past heights are stale.
equivocation: two signals from the same ν at the same t:
  - detected via SignalChain (hash chain + VDF ordering, owned by cybergraph)
  - both signals rejected; the neuron's focus is slashed
  - evidence committed to signals dimension in BBG_poly

### focus sufficiency

for the signal as a whole: neurons[ν].focus >= Σ cost(ℓ) for all ℓ ∈ ℓ⃗.
cost(ℓ) = a (conviction amount) + base_fee.
checked against current BBG_poly(neurons, ν, t) via Lens opening.

### UTXO ownership

every conviction UTXO referenced in ℓ⃗ must be:
  - unspent: N(nullifier) ≠ 0 (Lens opening into N(x))
  - owned by ν: A(commitment) resolves to ν's key (Lens opening into A(x))

note: BBG also checks double-spend structurally via N(x). cybergraph's ownership
check is the semantic layer on top — it confirms the right neuron is spending,
not just that the UTXO hasn't been spent by anyone.

### conservation

for all conviction UTXO movements in the signal:
  Σ inputs = Σ outputs + fee

verified by the zheng proof σ — it covers UTXO validity and conservation atomically.

### temporal ordering

each signal carries a VDF proof from cybergraph's SignalChain:
  - prev_hash: hemera(previous signal from ν)
  - vdf_proof: sequential squaring over Goldilocks, T iterations
  - T is proportional to the time elapsed since the previous signal

prevents timestamp manipulation. a signal claiming t >> actual time cannot
produce a valid VDF proof without actually waiting.

## validation error taxonomy

| error | axiom | who checks |
|-------|-------|-----------|
| UnknownParticle | A1 | cybergraph |
| InvalidProof | A2 | cybergraph |
| InvalidSignature | A2 | cybergraph |
| Equivocation | A3 | cybergraph (SignalChain) |
| InsufficientFocus | — | cybergraph |
| UtxoNotOwned | — | cybergraph |
| UtxoAlreadySpent | — | bbg (N(x) structural check) |
| ConservationViolation | — | cybergraph (via zheng proof) |
| InvalidVdf | — | cybergraph (SignalChain) |
| StaleHeight | — | cybergraph |

## signal lifecycle

```
network
  │
  ▼
cybergraph.submit(signal)
  │
  ├─ validate: A1, A2, A3, focus, UTXO ownership, conservation, VDF
  │    any failure → ValidationError, signal dropped
  │
  ▼
bbg.insert(signal)
  │
  ├─ check: N(nullifier) = 0 → DoubleSpend (structural rejection)
  │
  ├─ apply effects to BBG_poly (all 10 dimensions as needed)
  │
  ├─ extend A(x), N(x)
  │
  └─ recompute BBG_root
       │
       ▼
  new BBG_root returned to cybergraph
  │
  ▼
cybergraph advances SignalChain (prev_hash = H(signal))
```

see [[cybergraph/structural-sync]] for the 5-layer sync protocol. see [[bbg/state]] for the BBG_poly update effects. see [[cybergraph/chain]] for the SignalChain (hash chain + VDF). see [[bbg/privacy]] for the polynomial mutator set (A(x), N(x)).
