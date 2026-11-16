---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: order, ordering, equivocation check
---
# order

the second verb: place a validated signal in the causal chain of each of its [[neurons]], and reject equivocation. this is what the [[proof]] σ does not cover — a position-in-chain fact, not a computation — so it is checked here.

## criteria — what must hold

| criterion | what must hold |
|---|---|
| height | the signal's height $t$ equals the current block height; past-height signals are stale |
| chain link | for every subject $\nu_k \in \vec\nu$: `prev`$_k$ = $H(\nu_k$'s previous signal$)$ — the signal extends every subject's chain head at once |
| no equivocation | no other signal from any subject in $\vec\nu$ at the same step/height |
| temporal | a valid VDF proof: `vdf_proof` = sequential squaring over Goldilocks for $T$ iterations, $T$ proportional to time elapsed since the previous signal |

equivocation is not just rejected — both conflicting signals are rejected, the equivocating neuron's focus is slashed, and the evidence is committed to the signals dimension. in a signal of many subjects one subject's equivocation rejects the whole signal and slashes that subject alone; the others' chains are untouched, their heads unchanged. the VDF is what prevents timestamp manipulation: a signal claiming $t \gg$ actual time cannot produce a valid proof without actually waiting.

cybergraph routes all of this to [[foculus]] (the SignalChain — hash chain, VDF, equivocation detection); the mechanics live there, cybergraph calls them and reads the verdict.

mechanism, not decision: order is a fixed rule over chain position. it does not choose what to order — [[soma]] does, by producing signals.

see [[validate]] for the prior verb · [[apply]] for the next · [[foculus]] for the chain · [[signal]] / [[intent]] for the temporal fields.
