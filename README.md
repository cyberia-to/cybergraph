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

### derived structures

- [signal.md](reference/signal.md) — atomic broadcast unit. cyberlink batch + impulse + STARK proof
- [axon.md](reference/axon.md) — bundle of all cyberlinks between two particles. weight computation, homoiconicity

### computation

- [tri-kernel.md](reference/tri-kernel.md) — diffusion, springs, heat. composite operator, free energy, convergence proof, locality
- [focus-flow.md](reference/focus-flow.md) — focus flow computation, local update rule, compiled transformer derivation, cyberank
- [clifford.md](reference/clifford.md) — multivector primitive extensions. axon + $A^{\mathrm{eff}}$ as scalar + bivector, shifted geometric product, CT-1.0 compatibility contract

### rendering

- [render.md](reference/render.md) — deterministic 3d rendering. five tiers T0–T∞, graph-as-transformer neural field at T∞, honeycrisp backend, topology-stable determinism contract

## docs

explainers — narrative, intuition, motivation

- [overview.md](docs/overview.md) — what the cybergraph is, the five primitives, why content addressing and append-only matter
- [tri-kernel.md](docs/tri-kernel.md) — why three operators, the locality filter discovery, universal patterns, phase transitions
- [convergence.md](docs/convergence.md) — what convergence means, spectral gap, collective focus theorem in plain language
- [inference.md](docs/inference.md) — cybergraph as generative model, comparison to transformers, random walks
- [metrics.md](docs/metrics.md) — attention vs focus, transformer connection, gravity metric, ranking in practice
