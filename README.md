# cybergraph

the local-first cyberlink processor. signal in, graph out.

cybergraph is the component that turns a neuron's signed, proven [[signal]]s into an authenticated knowledge graph and lets anything read it. it validates the proof on each signal, orders signals into a per-neuron causal chain, applies their [[cyberlink]]s to state, and exposes the result to queries. it is local-first: one instance processes whatever cyberlinks it is pointed at — a single neuron, an avatar, a shared network — at whatever scope it is configured for.

it owns the *lifecycle* of a cyberlink. it does not compute focus (that is [[tru]]), store state (that is [[bbg]]), move bytes (that is [[sync]] / [[radio]]), or define value (that is [[plumb]]). it is the spine those attach to.

```
                          soma / apps / agents
                                   │
                                   ▼   intend · seal · link · subscribe · query
                             cybergraph
                          (validate · order · apply · expose)
                                   │
                   ┌───────────────┼───────────────┐
                   ▼               ▼               ▼
                 bbg             sync            radio
               (store)          (order,         (transmit)
                              distribute)
```

## what it does

| step | cybergraph | hands off to |
|---|---|---|
| validate | check the [[proof]] σ covers the whole signal against the current root | verify only — [[zheng]] constructs |
| order | append to the neuron's [[signal]] chain; reject equivocation | [[sync]] (hash chain, VDF) |
| apply | move each cyberlink's [[box]] into state — particle energy, axon weights, focus debit | [[bbg]] (`insert`, the mutator set) |
| expose | answer reads over the committed graph | [[inf]] (datalog), [[tru]] (focus) |

## API

five verbs. three write the lifecycle, two read.

```
intend(scope)            declare an unsealed intent — signed scope, no proof yet
seal(key, signal)        finalize a declared intent into a proven signal
link(signal)             atomic one-shot submit (no intent phase) for a local statement

subscribe(filter)        stream events as intents, seals, and links land
query(inf_script)        run an inf query over the graph's relations
```

writes route `validate → order → apply`; intent persists ahead of its proof so a declaration is on the record before it is sealed (or abandoned). reads run [[inf]] (datalog) over a snapshot of [[bbg]] state; the focus a query returns is [[tru]]'s.

## status — Release 0 (local-first)

working today: `intend` / `seal` / `link` apply cyberlinks to [[bbg]] state and advance the root; `query` runs the [[inf]] engine over local aggregate relations; `subscribe` delivers events in-process.

not yet: network distribution, STARK proof enforcement at seal, conviction `box_moves` (the local path carries cyberlinks only), provable queries (Lens openings over the root). these arrive as the stack matures around the same API.

## structure

cybergraph is exactly its structure — `cybergraph ← signals ← cyberlinks`, a fixed set of fields plus the emergents that appear when you read it. the [specs/](specs/) directory has one article per field, mapped in [specs/README.md](specs/README.md):

- the structure — [cybergraph](specs/cybergraph.md), [signal](specs/signal.md), [cyberlink](specs/cyberlink.md), [intent](specs/intent.md)
- signal fields — [neuron](specs/neuron.md), [network](specs/network.md), [impulse](specs/impulse.md), [proof](specs/proof.md)
- cyberlink fields — [particle](specs/particle.md), [token](specs/token.md), [amount](specs/amount.md), [box](specs/box.md), [valence](specs/valence.md)
- emergents — [axon](specs/axon.md), [attention](specs/attention.md)
- reads — [query](specs/query.md), [staking](specs/staking.md)
- the umbrella operation — [validation](specs/validation.md): routes each check to its owning component, then `bbg.insert`

## companion repos

| repo | owns |
|---|---|
| [[bbg]] | authenticated state, the mutator set, query proofs |
| [[sync]] | signal ordering, hash chain, VDF, distribution |
| [[tru]] | focus φ\*, the tri-kernel, karma, rewards |
| [[zheng]] | proof construction and verification |
| [[inf]] | the query language over cybergraph relations |
| [[plumb]] | the token/value layer (TSP-1/2, the five operations) |
| [[mudra]] | identity, the crypto primitives |

cybergraph defines the graph; the companions give it state, order, dynamics, proofs, queries, value, and identity.
