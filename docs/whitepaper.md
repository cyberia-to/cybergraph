---
tags: cyber, cybergraph, whitepaper
crystal-type: explanation
crystal-domain: cyber
---
# cybergraph — a processor for proven knowledge

## the idea

a knowledge graph where every change is a *proven computation*. not "someone wrote a fact and we trust them" — but "a neuron declared what it would do, ran it, and published a proof that it did exactly that." the graph is the shared, authenticated memory; cybergraph is the component that admits changes to it, and admits only changes that come with proof.

## cybergraph is the dumb half of a processor

the instinct is to make the graph engine *smart* — let it decide what to compute, when, how. that instinct is wrong, and it is why earlier designs sprawl. the clean architecture splits a mind from a machine:

- **cybergraph** is the dumb half — a store you can read, an event source, and a commit port that accepts only proven results. it holds no goals and makes no decisions.
- **[[soma]]** is the smart half — the runtime that decides what to do, computes it, and proves it.

together they are one processor. a change to the graph is a **fetch → execute → prove → commit** cycle: fetch an intent (an event), read operands (query), execute the computation ([[nox]]), prove it ([[zheng]]), commit the result (seal). soma drives the cycle; cybergraph is the memory and the commit gate. the interface between them is small *because* the split is clean — a dumb store drives a smart runtime through one channel: events.

this is why the hard part — "what should I compute next, and how" — never belongs in cybergraph. it is control, and control is the mind's. cybergraph stays fast, correct, and stateless on purpose.

## the signal: a promise, then a proof

the unit cybergraph processes is the **[[signal]]**; the [[cyberlink]]s a signal carries are what land in the graph. a signal has a two-phase life:

1. **intend** — the neuron declares a *scope*: a signed commitment to what it will compute. nothing has run. the scope is not a description; it is an executable specification.
2. **seal** — the neuron has run the scope (on nox), proven the run (with zheng), and now commits the result.

between the two, soma orchestrates: collect state, execute, see what is left, iterate, prove. the intent is a *promise*; the signal is its *proven fulfillment*.

## the seal binding — alignment at the commit port

what makes intent and signal one thing rather than two unrelated records is a single rule at the commit port:

> **seal is accepted iff the proof attests the declared scope:** `σ(s) ⊢ scope_hash(i)`.

a sealed signal is therefore cryptographic proof that the neuron did *exactly what it declared* — not a claim asking to be believed. this is the alignment property made concrete: prove the policy was followed; don't trust that it was. an intent that is never sealed stays on the record as an unkept promise — there is no silent cancel.

## what cybergraph actually does — four verbs of mechanism

once soma hands over a proven signal, cybergraph's own work is four static handoffs, each to one companion, with no decisions:

- **validate** — verify the proof σ against the root (→ [[zheng]])
- **order** — place the signal in the neuron's causal chain; reject equivocation (→ [[sync]])
- **apply** — move each cyberlink's [[box]] into authenticated state (→ [[bbg]])
- **expose** — answer reads over the committed graph (→ [[inf]], [[tru]])

that is the whole machine. everything else — focus, state, transport, value, identity, proof construction, and the control loop that decides what to compute — is a companion's, referenced and never restated.

## local-first, any scope

cybergraph is local-first: one instance processes whatever cyberlinks it is pointed at — a single neuron, an avatar, a shared network — at whatever scope it is configured for. the same four verbs and the same seal binding hold at every scope. distribution, when present, is [[sync]]'s; finality, when present, is [[foculus]]'s. cybergraph itself does not change.

## the boundary, stated plainly

cybergraph is the spine. it defines the graph and admits proven change to it. it is dumb by design, so the mind that drives it — soma — can be as smart as it needs to be without the spine ever having to decide anything.

see [README](../README.md) for the product surface and [specs/](../specs/) for the precise structure.
