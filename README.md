# cybergraph

signal in → .graph out. the glue between [[neuron]]s and the compute stack.

takes [[signal]]s from [[BBG]], validates [[proof]]s, persists [[cyberlink]]s, manages conviction UTXOs, and routes events to [[tru]] and [[glia]] via subscriptions. embeds a link query engine that speaks `.graph`.

```
neurons  →  signals  →  cybergraph  →  .graph queries  →  tru
                              ↓                            glia
                         subscriptions  ────────────────→  tru / glia / mir
                              ↓
                         BBG (state)
```

## API

```
submit(signal)            validate σ, extract cyberlinks, persist, emit events
query(.graph)             link query engine — from?, to?, neuron?, depth?
query(from=ν, to=p)       attention: how much focus ν projects onto p
subscribe(filter)         stream cyberlinks to tru / glia consumers
```

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
