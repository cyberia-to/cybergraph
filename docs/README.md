---
tags: cyber, cybergraph, whitepaper
crystal-type: explanation
crystal-domain: cyber
alias: cybergraph whitepaper
---
# cybergraph

a knowledge graph where every change is a proven computation.

not "someone wrote a fact and we trust them," and not "a contract moved tokens and we re-execute to check." instead: a neuron declares what it will do, runs it, and publishes a proof that it did exactly that — and anyone holding a 32-byte root verifies the proof in microseconds, without the graph, without trusting whoever produced it. the graph is the shared, authenticated memory of the planet; cybergraph is the component that admits change to it, and admits only change that comes with proof.

this document explains the architecture. the product surface is [README](../README.md); the precise structure is [specs/](../specs/).

---

## 1. the idea: a processor, split into a mind and a machine

the instinct is to make a graph engine smart — let it decide what to compute, when, and how. that instinct is what makes such systems sprawl. cybergraph takes the opposite path and splits a mind from a machine:

- **cybergraph** is the machine — a store you can read, a source of events, and a commit port that accepts only proven results. it holds no goals and makes no decisions. it is fast, correct, and stateless on purpose.
- **[[soma]]** is the mind — the runtime that decides what to do, computes it, and proves it.

together they are one processor. a change to the graph is a cycle:

```
        soma — the runtime: decides · executes on nox · proves with zheng
          │  ▲
   intend │  │  subscribe   ← events are the clock: "an intent was declared"
   seal   │  │  query       ← read operands from committed state
          ▼  │
      cybergraph — the machine: store · events · commit-gate
          │
   ┌──────┼──────┐
   ▼      ▼      ▼
  bbg    sync   radio
 store   order  transmit
```

fetch → execute → prove → commit. soma drives the cycle; cybergraph is the memory and the commit gate. the interface between them is small precisely because the split is clean: a dumb store can drive a smart runtime through a single channel — events. all intelligence lives in the mind; the machine never has to decide anything. this is why the hard part — "what should I compute next, and how" — never belongs in cybergraph. that is control, and control is the mind's.

---

## 2. what flows: signals, made of cyberlinks

the unit cybergraph processes is the [[signal]]; the [[cyberlink]]s a signal carries are what land in the graph. cybergraph is exactly this structure, nothing more:

```
cybergraph  ←  signals  ←  cyberlinks
```

a cyberlink is the atomic unit of knowledge. its five fields group into three concepts:

$$\text{cyberlink} \;=\; \text{link}(from,\,to) \;+\; \text{box}(coin,\,amount) \;+\; valence$$

- a [[link]] connects two [[card|cards]] — `from → to`. a [[particle]] (content-addressed knowledge node) is a card; a [[neuron]] is a card; a network is a card. one mechanism covers a knowledge edge, a transfer, or a stake.
- a [[box]] is the conviction moved: `amount` units of a [[coin]] denomination. boxes move from one object to another — created, transferred, withdrawn, spent — never conjured. this is the economic layer.
- a [[valence]] is the epistemic prediction (`-1 / 0 / +1`): where the neuron forecasts collective belief will settle.

two structures emerge when you read the graph rather than write it: the [[axon]] (the bundle of all links between two particles — itself a particle, so the graph ranks its own structure) and [[attention]] (the focus a neuron projects onto a target, returned by a query). the field-by-field detail is in [specs/](../specs/).

---

## 3. the signal lifecycle: a promise, then a proof

a signal does not arrive finished. it has a two-phase life, and the phase between the two is where all the work is:

```
intend(scope)   declare what will be computed — a signed commitment, nothing run yet
     │
     │  soma executes the scope on nox, reading state via query, iterating
     │  ("what is left to compute?") until it converges → cyberlinks + impulse Δφ* ;
     │  zheng proves the run → σ
     ▼
seal(signal)    commit the proven run
```

an [[intent]] is a deferred computation. its scope is not a passive description — it is an executable specification of what the neuron will compute. the intent commits to it (a signed `scope_hash`); the signal proves it was run. completion fills the three things an intent lacks — the cyberlinks, the impulse Δφ\*, and the proof σ — by executing the scope ([[nox]]) and proving the execution ([[zheng]]). the orchestration of that loop — collect, run, judge what remains, iterate — is the mind's; cybergraph stores the intent, serves the reads, and gates the commit.

`link(signal)` is the one-shot path: an atomic local statement that needs no separate intent phase.

---

## 4. the seal binding: alignment at the commit port

what makes an intent and a signal one thing rather than two unrelated records is a single rule at the commit gate:

> **seal is accepted iff the proof attests the declared scope:**  $\sigma(s) \vdash \text{scope\_hash}(i)$.

the intent is a promise (a committed scope); the signal is its proven fulfillment. a sealed signal is therefore cryptographic proof that the neuron did exactly what it declared — a thing you verify, not a claim you believe. an intent that is never sealed remains on the record as an unkept promise; there is no silent cancel.

this is the property the whole stack exists for. prove the policy was followed; do not trust that it was. for an autonomous agent — a soma avatar acting on its own — this is the difference between an audit trail and a guarantee: the agent cannot seal a computation it did not run as declared.

---

## 5. what the machine does: four verbs

once the mind hands over a proven signal, cybergraph's own work is four static handoffs, each to one companion, with no decisions:

| verb | the machine | hands to |
|---|---|---|
| [validate](../specs/validate.md) | check the conditions a signal must satisfy | the proof σ ([[zheng]]), an availability proof ([[sync]] DAS), the chain ([[sync]]), routing ([[network]]) |
| [order](../specs/order.md) | place the signal in the neuron's causal chain; reject equivocation | [[sync]] (hash chain, VDF) |
| [apply](../specs/apply.md) | move each cyberlink's box into authenticated state | [[bbg]] (`insert`, the mutator set) |
| [expose](../specs/expose.md) | answer reads over the committed graph | [[inf]] (datalog), [[tru]] (focus) |

validation is the gate. most conditions are carried atomically by the proof σ — signature, cyberlink validity, box ownership, conservation, focus sufficiency, impulse correctness; the rest are checked alongside — availability (the referenced content is published, by DAS), ordering (chain position and a VDF that defeats timestamp manipulation), and network routing. one verification, `decide(σ)`, in $O(\log n)$, no re-execution. the full criteria table is on the [validation page](../specs/validate.md).

reads are the other half: a query is an [[inf]] program (datalog) over a committed snapshot; the focus a query returns is computed by [[tru]]. provable queries compile, through [[zheng]], into proofs whose reads are [[bbg]] openings — the same answer, now portable to anyone with the root.

---

## 6. what the machine does not do

the decision loop — read an intent, collect recomputed state from [[bbg]], run it through [[nox]], judge what is left to compute, iterate, and only then emit a signal — is dynamic control, not a static pipeline. it is the [[soma]] cognitive loop, above cybergraph. soma calls the four verbs; it is not one of them. focus, state, transport, value, identity, and proof construction each belong to a companion, referenced and never restated. cybergraph defines the graph and admits proven change to it; everything else attaches.

---

## 7. local-first, at any scope

cybergraph is local-first: one instance processes whatever cyberlinks it is pointed at — a single neuron, an avatar across bodies, a regional aggregate, the planetary union — at whatever scope it is configured for. the four verbs and the seal binding hold identically at every scope. distribution, when present, is [[sync]]'s; finality, when present, is [[foculus]]'s. cybergraph itself does not change. a researcher querying their own links and a planetary validator run the same machine over different amounts of data.

---

## 8. the stack

cybergraph is the spine. it stays dumb by design so the mind that drives it can be as smart as it needs to be without the spine ever deciding anything.

| repo | owns |
|---|---|
| [[soma]] | the runtime — decides, executes, proves, drives the lifecycle |
| [[bbg]] | authenticated state, the mutator set, query proofs |
| [[sync]] | signal ordering, hash chain, VDF, availability, distribution |
| [[tru]] | focus φ\*, the tri-kernel, karma, rewards |
| [[zheng]] | proof construction and verification |
| [[nox]] | execution — runs a scope to a proven result |
| [[inf]] | the query language over cybergraph relations |
| [[tok]] | the token/value layer — coins, cards (operations via the [[plumb]] framework) |
| [[mudra]] | identity, the crypto primitives |
| [[foculus]] | finality from graph topology |

cybergraph defines the graph; the companions give it state, order, dynamics, proofs, queries, value, identity — and the mind that drives it.

---

## the shape of it

a dumb store with one smart neighbor. the store admits only proven change and remembers it; the neighbor decides, computes, and proves. an intent is a promise, a signal is its proof, and the commit gate accepts the second only if it fulfills the first. that is cybergraph: the processor that turns a planet's intentions into proven, shared memory.
