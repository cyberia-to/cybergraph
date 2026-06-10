# cybergraph

the local-first cyberlink processor. signal in, graph out.

cybergraph turns a neuron's signed, proven [[signal]]s into an authenticated knowledge graph and lets anything read it. the unit it processes is the **signal**; the [[cyberlink]]s a signal carries are what land in the graph. it validates each signal's proof, orders signals into a per-neuron causal chain, applies their cyberlinks to state, and exposes the result to queries. local-first: one instance processes whatever it is pointed at — a single neuron, an avatar, a shared network — at whatever scope it is configured for.

it owns the **signal lifecycle** — declare, complete, commit. it does not compute focus ([[tru]]), store state ([[bbg]]), move bytes ([[sync]] / [[radio]]), define value ([[tok]] via the [[plumb]] operations), or construct proofs ([[zheng]]). it is the spine those attach to.

## cybergraph is the dumb half of a processor

cybergraph and [[soma]] are one machine. cybergraph is the **dumb half** — a store you can read, an event source, and a commit port that only accepts *proven* results. soma is the **smart half** — the runtime that decides what to do, computes it, and proves it. a signal's life is a **fetch → execute → prove → commit** cycle, and soma drives it through cybergraph's minimal interface:

```
                 soma — the runtime (decides · executes on nox · proves with zheng)
                   │  ▲
          intend   │  │  subscribe   ← events are the clock: "an intent was declared"
          seal     │  │  query       ← read operands from committed state
                   ▼  │
              cybergraph — dumb: store · events · commit-gate
                   │
        ┌──────────┼──────────┐
        ▼          ▼          ▼
       bbg        sync       radio
      (store)   (order,    (transmit)
              distribute)
```

cybergraph never decides. it emits events, serves reads, and gates commits on proof. all intelligence is soma's. the interface stays small *because* the dumb/smart split is clean: a dumb store can drive a smart runtime through one channel — events.

## the signal lifecycle — promise, then proof

a signal is a deferred computation that becomes a proven one:

```
intend(scope)   declare what will be computed — a signed commitment, nothing run yet
     │
     │  soma executes the scope on nox, reading state via query, iterating until it
     │  converges → cyberlinks + impulse Δφ* ; zheng proves the run → σ
     ▼
seal(signal)    commit — accepted only if σ proves the declared scope
```

**the seal binding:** `seal(i, s)` is accepted **iff** `σ(s) ⊢ scope_hash(i)`. the intent is a *promise* (a committed scope); the signal is its *proven fulfillment*. a sealed signal is cryptographic proof the neuron did exactly what it declared — not a claim asking to be trusted. this binding is what makes intent and signal **one transaction**, and it is the alignment property enforced at the commit port: prove the policy was followed, don't trust that it was.

`link(signal)` is the one-shot path — no separate intent phase, for a discrete local statement.

## what it does — four verbs of mechanism

each is a static handoff to one companion. no decisions.

| step | cybergraph | hands off to |
|---|---|---|
| [validate](specs/validate.md) | check the [[proof]] σ covers the signal against the root | verify only — [[zheng]] constructs |
| [order](specs/order.md) | append to the neuron's [[signal]] chain; reject equivocation | [[sync]] (hash chain, VDF) |
| [apply](specs/apply.md) | move each cyberlink's [[box]] into state — particle energy, axon weights, focus debit | [[bbg]] (`insert`, the mutator set) |
| [expose](specs/expose.md) | answer reads over the committed graph | [[inf]] (datalog), [[tru]] (focus) |

## what it does NOT do — orchestration is soma's

the decision loop — read an [[intent]], collect recomputed state from [[bbg]], run it through [[nox]], judge what is left to compute, iterate, and only then emit a signal — is dynamic control, not a static pipeline. that orchestration is the [[soma]] cognitive loop, above cybergraph. soma *calls* the four verbs; it is not one of them. cybergraph is the fast, correct, stateless processor; soma is the mind that drives it.

## API

five verbs. three write the lifecycle, two read.

```
intend(scope)            declare an unsealed intent — signed scope, no proof yet
seal(key, signal)        commit a proven signal against its intent (σ ⊢ scope)
link(signal)             atomic one-shot submit (no intent phase) for a local statement

subscribe(filter)        stream events as intents, seals, and links land
query(inf_script)        run an inf query over the graph's relations
```

reads run [[inf]] (datalog) over a snapshot of [[bbg]] state; the focus a query returns is [[tru]]'s.

## status — Release 0 (local-first)

working today: `intend` / `seal` / `link` apply cyberlinks to [[bbg]] state and advance the root; `query` runs the [[inf]] engine over local aggregate relations; `subscribe` delivers events in-process.

not yet: network distribution, the seal binding (`σ ⊢ scope` enforcement) and STARK verification at seal, conviction `box_moves` (the local path carries cyberlinks only), provable queries (Lens openings over the root). these arrive as the stack matures around the same API.

## structure

cybergraph is exactly its structure — `cybergraph ← signals ← cyberlinks`, a fixed set of fields plus the emergents that appear when you read it, plus the verbs it does. one article per thing, mapped in [specs/README.md](specs/README.md):

- the structure — [cybergraph](specs/cybergraph.md), [signal](specs/signal.md), [cyberlink](specs/cyberlink.md), [intent](specs/intent.md)
- a cyberlink is `link + box + valence` — [link](specs/link.md) (from [card] → to [card]), [box](specs/box.md) ([coin] + [amount](specs/amount.md)), [valence](specs/valence.md); the node a card can be is the [particle](specs/particle.md)
- signal fields — [neuron](specs/neuron.md), [network](specs/network.md), [impulse](specs/impulse.md), [proof](specs/proof.md)
- emergents — [axon](specs/axon.md), [attention](specs/attention.md)
- process (mechanism) — [validate](specs/validate.md), [order](specs/order.md), [apply](specs/apply.md), [expose](specs/expose.md)
- reads — [query](specs/query.md), [staking](specs/staking.md)

## companion repos

cybergraph defines the graph; the companions give it state, order, dynamics, proofs, queries, value, identity — and the mind that drives it.

| repo | owns |
|---|---|
| [[soma]] | the runtime — decides, executes, proves, drives the lifecycle |
| [[bbg]] | authenticated state, the mutator set, query proofs |
| [[sync]] | signal ordering, hash chain, VDF, distribution |
| [[tru]] | focus φ\*, the tri-kernel, karma, rewards |
| [[zheng]] | proof construction and verification |
| [[nox]] | execution — runs a scope to a proven result |
| [[inf]] | the query language over cybergraph relations |
| [[tok]] | the token/value layer — TSP-1 coins, TSP-2 cards (operations via the [[plumb]] framework) |
| [[mudra]] | identity, the crypto primitives |
