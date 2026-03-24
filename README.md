# cybergraph

A directed authenticated multigraph over content-addressed nodes, carrying an emergent probability measure -- the shared memory of the planet.

## Structure

### reference/

Implementable specifications -- formal definitions, axioms, proofs, data structures, formulas.

- [cybergraph.md](reference/cybergraph.md) -- formal definition of the cybergraph triple (P, N, L), six axioms, derived structures, four theorems, information geometry, category theory
- [primitives.md](reference/primitives.md) -- particle (Hemera addressing, tree structure, domain separation), cyberlink (seven fields, three layers, UTXO semantics, CRUD), signal (structure, STARK proof coverage, minting), axon (bundles, homoiconicity, meta-annotation)
- [tri-kernel.md](reference/tri-kernel.md) -- diffusion, springs, heat kernel specifications, composite operator, free energy functional, contraction lemmas, collective focus theorem (Parts I and II), completeness conjecture, complexity, implementation
- [focus-flow.md](reference/focus-flow.md) -- focus flow computation, two inference paths, local update rule, compiled transformer derivation, graph-derived architecture parameters, cyberank, compounding property

### docs/

Explainers -- narrative, intuition, motivation, analogies.

- [overview.md](docs/overview.md) -- what the cybergraph is, the five primitives explained intuitively, why content addressing and append-only matter, the cybergraph as shared memory
- [tri-kernel.md](docs/tri-kernel.md) -- why three operators (the locality filter discovery), what each does (diffusion = curiosity, springs = stability, heat = patience), universal patterns table, phase transitions, adversarial resistance, the Friston connection
- [convergence.md](docs/convergence.md) -- what convergence means, spectral gap explained, collective focus theorem in plain language, locality and bounded effects
- [inference.md](docs/inference.md) -- the cybergraph as generative model, comparison to transformers, random walks, query biasing, autoregressive generation, zero-cost proof-carrying
- [metrics.md](docs/metrics.md) -- attention vs focus, the transformer attention connection, attention as Bayesian query, multi-head and semcon types, gravity metric, ranking in practice
