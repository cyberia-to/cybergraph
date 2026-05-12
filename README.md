# cybergraph

a directed authenticated multigraph over content-addressed nodes, carrying an emergent [[probability]] measure — the shared memory of the planet

five primitives: [[particle]], [[cyberlink]], [[neuron]], [[token]], [[focus]]

## reference

implementable specifications — formal definitions, data structures, formulas

### the graph

- [cybergraph.md](reference/cybergraph.md) — formal triple $(P, N, L)$, six axioms, derived structures, four theorems, information geometry

### primitives

- [particle.md](reference/particle.md) — content-addressed node. Hemera hash, tree structure, domain separation, endofunction, permanence
- [cyberlink.md](reference/cyberlink.md) — atomic unit of knowledge. seven fields, three layers, UTXO semantics, CRUD, the card

### cyberlink arguments

- [token.md](reference/token.md) — $\tau$: denomination of conviction. rate function, four token types, multi-denomination staking
- [amount.md](reference/amount.md) — $a$: quantity of stake. role in adjacency, conviction UTXO, costly signaling, reward proportionality
- [valence.md](reference/valence.md) — $v$: epistemic prediction $\{-1, 0, +1\}$. BTS meta-prediction, role in ICBS markets, karma compounding
- [time.md](reference/time.md) — $t$: block height. discovery premium, temporal decay, consensus ordering, machine time

### participation

- [signal.md](reference/signal.md) — how a neuron acts. cyberlink batch + impulse + STARK proof. decomposes into cyberlinks but is the fundamental unit of what happens

### derived structures

- [axon.md](reference/axon.md) — bundle of all cyberlinks between two particles. weight computation, homoiconicity

## docs

explainers — narrative, intuition, motivation

- [overview.md](docs/overview.md) — what the cybergraph is, the five primitives, why content addressing and append-only matter
