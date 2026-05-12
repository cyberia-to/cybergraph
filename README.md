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

## reference

### the graph

- [cybergraph.md](reference/cybergraph.md) — formal triple $(P, N, L)$, six axioms, derived structures, four theorems

### primitives

- [particle.md](reference/particle.md) — content-addressed node. Hemera hash, permanence, domain separation
- [cyberlink.md](reference/cyberlink.md) — the unit routed. five fields, three layers, UTXO semantics
- [neuron.md](reference/neuron.md) — the sender. identity, stake, focus, karma
- [token.md](reference/token.md) — conviction denomination. four types, rate function, multi-token staking
- [signal.md](reference/signal.md) — the input to `submit()`. cyberlink batch + impulse + proof

### signal internals

- [proof.md](reference/proof.md) — what `submit()` validates. validation interface; construction is [[zheng]]'s
- [impulse.md](reference/impulse.md) — $\Delta\phi^*$: the proven focus shift carried by a signal

### neuron

- [identity.md](reference/identity.md) — Hemera hash of public key. no registration, no authority
- [staking.md](reference/staking.md) — token movement as graph event. conviction UTXOs, adjacency updates

### cyberlink fields

- [amount.md](reference/amount.md) — $a$: stake quantity. conviction UTXO, adjacency weight
- [valence.md](reference/valence.md) — $v \in \{-1,0,+1\}$: epistemic prediction. BTS, ICBS markets
- [time.md](reference/time.md) — $t$: block height. ordering, decay, discovery premium

### derived

- [axon.md](reference/axon.md) — bundle of all cyberlinks between two particles. homoiconic, rankable

## docs

- [overview.md](docs/overview.md) — what the cybergraph is and why it exists
