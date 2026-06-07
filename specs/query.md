---
tags: cyber, soft3, cybergraph, query
crystal-type: spec
crystal-domain: cyber
---
# query

cybergraph does not define its own query language. it exposes a set of stored relations and delegates query execution to [[inf]] — cyber's declarative datalog layer, implemented via CozoDB.

this spec defines only the **relation schema** that cybergraph commits to exposing. the language semantics (rules, recursion, aggregation, fixed rules) live in [[inf/README]] and the deep-dive files [[inf/queries]], [[inf/algorithms]], [[inf/stored relations]], [[inf/functions]], [[inf/cybergraph]].

## scope split

| concern | owner |
|---|---|
| query language syntax + semantics | [[inf]] |
| graph algorithms (PageRank, BFS, Louvain, …) | [[inf]] (fixed rules) |
| query engine implementation (CozoDB) | [[inf]] |
| relation schema cybergraph exposes | this file |
| live relation contents | [[bbg]] (authenticated state) |
| query-result proofs | [[zheng]] (when query is provable) |

cybergraph guarantees that the relations below exist and are kept in sync with the signal log. inf can be pointed at them without further coupling.

## implementation status (Release 0)

`Cybergraph::query(inf_script)` is wired to the inf engine: `parse → plan → eval` over a `BbgSource` that projects local bbg aggregate state into inf relations. The bootstrap path runs the native inf evaluator (`inf-eval`), not CozoDB — CozoDB is inf's differential oracle, and shrinks to zero as inf self-hosts (see [[inf/README]]).

`BbgSource` exposes the relations bbg actually holds as public aggregate state:

| relation | columns | bbg source |
|---|---|---|
| `particles` | cid, energy, pi_star, weight | `particles` map |
| `neurons` | id, focus, karma, stake | `neurons` map |
| `axons` | from, to, weight_sum | `axon_edges` × `particles[axon].weight` |
| `focus` | particle, score | `particles[cid].pi_star` |
| `karma` | neuron, k | `neurons[id].karma` |
| `signals` | step, neuron, link_count, height | `signals` map |

`cyberlinks` is **not** exposed by the bbg source. Individual cyberlinks are the private layer (mutator set); bbg holds only the public aggregate, so a query referencing `cyberlinks` over the bbg source sees an empty relation. The private cyberlink source is a separate (future) `RelationSource`.

Release 0 reads run over the local snapshot and are not yet provable (`BbgSource::provable() == false`); Lens openings over the committed root arrive in a later release.

## exposed relations

every relation is a stored CozoDB relation keyed by content-addressed identifiers. types follow [[inf/stored relations]] conventions.

### `cyberlinks`

every cyberlink that has entered the authenticated record. one row per `(neuron, step)` position in a signal — the atomic assertion.

```datalog
:create cyberlinks {
    neuron:  Bytes,    // 32-byte NeuronId
    step:    Int,      // position within neuron's signal chain
    idx:     Int,      // position within signal's link batch
    =>
    from:    Bytes,    // source particle (32 bytes)
    to:      Bytes,    // target particle (32 bytes)
    token:   Bytes,    // token denomination particle (32 bytes)
    amount:  Int,      // conviction stake
    valence: Int,      // -1 | 0 | +1
    height:  Int,      // block height at finalization
}
```

### `particles`

every particle that has been referenced as `from` or `to` by at least one cyberlink (axiom A4 — entry by linking).

```datalog
:create particles {
    cid: Bytes  // 32-byte hemera digest
    =>
    first_seen: Int,   // block height of first reference
    content_type: String,
    size: Int,
}
```

### `neurons`

every neuron that has submitted at least one signal.

```datalog
:create neurons {
    id: Bytes
    =>
    focus:  Int,
    karma:  Int,
    stake:  Int,
}
```

### `signals`

every finalized signal (header only; the cyberlink batch is normalized into `cyberlinks`).

```datalog
:create signals {
    neuron: Bytes,
    step:   Int,
    =>
    prev:         Bytes,   // hash of previous signal in neuron's chain
    merkle_clock: Bytes,
    vdf_proof:    Bytes,
    height:       Int,
    hash:         Bytes,
}
```

### `axons` (derived)

bundle view of all cyberlinks between two particles. computed lazily from `cyberlinks` by axiom A6. exposed as a stored relation for query performance; recomputed when the underlying cyberlinks change.

```datalog
:create axons {
    from: Bytes,
    to:   Bytes,
    =>
    cid:           Bytes,    // hemera(from || to) — the axon-particle
    weight_sum:    Int,
    link_count:    Int,
}
```

### `focus` and `karma` (read-through to tru)

cybergraph does not compute focus or karma — that is [[tru]]'s job. inf can still query these as if they were stored relations; the cybergraph runtime forwards reads to tru's commitment over its current `.model`.

```datalog
:create focus {
    particle: Bytes
    =>
    score: Float,
}

:create karma {
    neuron: Bytes
    =>
    k: Float,
}
```

writes to `focus` and `karma` from inf are rejected — these are read-only projections of tru output.

## consistency model

- `cyberlinks`, `particles`, `neurons`, `signals` are committed atomically with every accepted signal. inf queries against them see a consistent snapshot at the latest accepted block height.
- `axons` is updated within the same transaction as `cyberlinks` writes.
- `focus`, `karma` lag by one tru cycle (the convergence step between blocks).

## query provability

an inf query is a derivation tree over the relations above. for queries that need cryptographic guarantees, the tree is compiled through [[zheng]] into a STARK proof: relation reads become [[bbg]] [[Lens]] openings; rule application becomes circuit constraints. see [[inf/cybergraph]] for the provable-query path and [[bbg/specs/query]] for the proof construction.

interactive queries skip the proof step and run directly through CozoDB. the same inf script produces the same answer either way — provability is an opt-in cost at submission time.

## what is NOT in this spec

- language syntax (rules, atoms, `:=`, `<~`, `?`, query options) — [[inf/queries]]
- built-in functions (math, string, vector, JSON, aggregation) — [[inf/functions]]
- fixed rules (PageRank, Dijkstra, Louvain, BFS) — [[inf/algorithms]]
- query-to-proof compilation — [[zheng]] + [[inf/cybergraph]]
- storage backend, persistence, time-travel — [[bbg]] for authenticated state, [[inf/stored relations]] for query-layer semantics
