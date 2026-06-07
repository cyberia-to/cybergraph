# cybergraph

signal in → [[inf]] queries out. the glue between [[neuron]]s and the compute stack.

takes [[signal]]s from [[BBG]], validates [[proof]]s, persists [[cyberlink]]s, manages conviction boxes, and routes events to [[tru]] and [[glia]] via subscriptions. exposes the cybergraph relations to [[inf]] (CozoDB-backed datalog) as the link query layer.

```
neurons  →  signals  →  cybergraph  ─── inf queries ─── ▶  tru
                              ↓                              glia
                         subscriptions  ──────────────────▶  tru / glia / mir
                              ↓
                         BBG (state)
```

## API

```
submit(signal)            validate σ, extract cyberlinks, persist, emit events
query(inf_script)         run an [[inf]] (CozoScript) query over cybergraph relations
subscribe(filter)         stream cyberlinks to tru / glia consumers
```

the query language is [[inf]] — datalog over the cybergraph, implemented via CozoDB. cybergraph exposes a stable set of stored relations (cyberlinks, particles, neurons, signals, focus, karma) and delegates query execution to inf. see [specs/query.md](specs/query.md) for the relation schema and [[inf/README]] for the language.

UTXO management is internal to `submit()`: conviction UTXOs created and spent per cyberlink, token movements trigger adjacency updates that tru reads.

## specs

### the graph

- [cybergraph.md](specs/cybergraph.md) — formal triple $(P, N, L)$, six axioms, derived structures, four theorems

### primitives

- [particle.md](specs/particle.md) — content-addressed node. Hemera hash, permanence, domain separation
- [cyberlink.md](specs/cyberlink.md) — the unit routed. five fields, three layers, UTXO semantics
- [neuron.md](specs/neuron.md) — the sender. identity, stake, focus, karma
- [token.md](specs/token.md) — conviction denomination. four types, rate function, multi-token staking
- [signal.md](specs/signal.md) — the input to `submit()`. cyberlink batch + impulse + proof

### signal internals

- [proof.md](specs/proof.md) — what `submit()` validates. validation interface; construction is [[zheng]]'s
- [impulse.md](specs/impulse.md) — $\Delta\phi^*$: the proven focus shift carried by a signal

### query interface

- [query.md](specs/query.md) — relations cybergraph exposes to [[inf]]. schema only; language spec lives in [[inf/README]]

### neuron

- [identity.md](specs/identity.md) — Hemera hash of public key. no registration, no authority
- [staking.md](specs/staking.md) — token movement as graph event. conviction UTXOs, adjacency updates

### cyberlink fields

- [amount.md](specs/amount.md) — $a$: stake quantity. conviction UTXO, adjacency weight
- [valence.md](specs/valence.md) — $v \in \{-1,0,+1\}$: epistemic prediction. BTS, ICBS markets
- [time.md](specs/time.md) — $t$: block height. ordering, decay, discovery premium

### derived

- [axon.md](specs/axon.md) — bundle of all cyberlinks between two particles. homoiconic, rankable

## docs

- [overview.md](docs/overview.md) — what the cybergraph is and why it exists
