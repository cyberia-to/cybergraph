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
- [neuron.md](reference/neuron.md) — the one who links. identity, stake, focus, karma. source of all cyberlinks
- [token.md](reference/token.md) — $\tau$: denomination of conviction. rate function, four token types, multi-denomination staking
- [focus.md](reference/focus.md) — collective attention. $\pi^*$ probability distribution over particles. emerges from aggregate neuron signals via tri-kernel
- [signal.md](reference/signal.md) — the atomic act. cyberlink batch + impulse + STARK proof. decomposes into cyberlinks

### neuron

- [identity.md](reference/identity.md) — how a neuron gets its identity. Hemera hash of public key, signatureless proofs, hash-based addressing
- [staking.md](reference/staking.md) — directing economic weight toward particles and axons. will (broad) and fine-tuning (per-target)
- [attention.md](reference/attention.md) — what staking produces at the receiving end. measurable projection of a neuron onto a target

### signal

- [impulse.md](reference/impulse.md) — $\pi_\Delta$: the proven focus shift a signal delivers. sparse vector, locality theorem, self-minting proof
- [proofs.md](reference/proofs.md) — the full STARK proof system. recursive verification, proof types, aggregation, nox integration

### cyberlink arguments

- [amount.md](reference/amount.md) — $a$: quantity of stake. role in adjacency, conviction UTXO, costly signaling, reward proportionality
- [valence.md](reference/valence.md) — $v$: epistemic prediction $\{-1, 0, +1\}$. BTS meta-prediction, role in ICBS markets, karma compounding
- [time.md](reference/time.md) — $t$: block height. discovery premium, temporal decay, consensus ordering, machine time

### derived structures

- [axon.md](reference/axon.md) — bundle of all cyberlinks between two particles. weight computation, homoiconicity

## docs

- [overview.md](docs/overview.md) — what the cybergraph is, the five primitives, why content addressing and append-only matter
