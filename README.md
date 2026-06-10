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

five verbs over a local cyberlink processor:

```
// lifecycle (discrete, ordered phases)
intend(scope)            declare an unsealed intent: signed scope, no STARK yet
seal(key, signal)        finalize an intent into a complete signal with STARK
link(signal)             atomic one-shot submit for discrete local statements
                         (no separate intent phase required)

// interaction (read / observe)
subscribe(filter)        register a handler over an event filter
query(inf_script)        run an [[inf]] (CozoScript) query over relations
```

the query language is [[inf]] — datalog over the cybergraph, implemented via CozoDB. cybergraph exposes a stable set of stored relations (cyberlinks, particles, neurons, signals, focus, karma) and delegates query execution to inf. see [specs/query.md](specs/query.md) for the relation schema and [[inf/README]] for the language.

internal fan-out: `link`/`seal` order the signal through `cyber_sync::SignalChain`, then apply its cyberlinks to authenticated state via `bbg.insert` (particle energy, axon weights, focus debit), then record the signal header. `intend` persists through `bbg.apply_intent`. soma sees a single import surface.

Release 0 scope (local-first): cyberlinks land in bbg state and move `BBG_root`; no network, no STARK validation, no conviction `box_moves` yet (the local `CyberlinkRecord` carries none). `query` is wired to inf in a later release.

UTXO management is internal to `submit()`: conviction UTXOs created and spent per cyberlink, token movements trigger adjacency updates that tru reads.

## specs

cybergraph specs cover exactly three things: the data structure, the signal lifecycle, and the query interface. dynamics ([[focus]], [[tri-kernel]], [[karma]], rewards) live in [[tru]]; the value layer (token types, PLUMB ops) in [[tok]]/[[plumb]]; identity crypto in [[mudra]]; authenticated state in [[bbg]]. cybergraph references them, it does not restate them.

### the graph

- [cybergraph.md](specs/cybergraph.md) — formal triple $(P, N, L)$, six axioms, derived adjacency. dynamics → tru

### primitives

- [particle.md](specs/particle.md) — content-addressed node. Hemera hash, permanence, domain separation
- [cyberlink.md](specs/cyberlink.md) — the atomic unit. five fields → four concepts $(from, to, box, valence)$
- [box.md](specs/box.md) — the conviction unit $(token, a)$. lifecycle: move / transfer / withdraw / spend
- [token.md](specs/token.md) — the $\tau$ field: denomination + rate $r(\tau)$
- [amount.md](specs/amount.md) — the $a$ field: box magnitude
- [valence.md](specs/valence.md) — the $v$ field: epistemic prediction $\{-1,0,+1\}$
- [neuron.md](specs/neuron.md) — the agent. identity $= \text{Hemera}(\text{pk})$
- [axon.md](specs/axon.md) — derived: the bundle of all links between two particles (axiom A6)

### signal lifecycle

- [signal.md](specs/signal.md) — the broadcast unit a neuron commits
- [intent.md](specs/intent.md) — an unsealed signal: declared, identity-proven, not yet sealed
- [validation.md](specs/validation.md) — what `submit()` checks before bbg; the six-axiom gate + proof boundary
- [network.md](specs/network.md) — signal routing: a network is a card; private-by-default

### query interface

- [query.md](specs/query.md) — relations cybergraph exposes to [[inf]]. schema only; language lives in [[inf/README]]
- [attention.md](specs/attention.md) — the result of `query(from, to)`: how much focus a neuron projects onto a target. value comes from [[tru]]
