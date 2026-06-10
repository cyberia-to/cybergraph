---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: order, ordering, equivocation check
---
# order

the second verb: place a validated signal in its [[neuron]]'s causal chain, and reject equivocation.

```
order(signal) → ok | reject
  step is sequential · prev matches the chain head · no fork from this ν at this step
  append to the neuron's signal chain
```

this is the only check the [[proof]] σ does not cover — a position-in-chain fact, not a computation. cybergraph routes the chain mechanics (hash chain, VDF, equivocation detection) to [[sync]]; ordering logic lives there, cybergraph calls it.

mechanism, not decision: order is a fixed rule over chain position. it does not choose what to order — [[soma]] does, by producing signals.

see [[validate]] for the prior verb · [[apply]] for the next · [[sync]] for the chain · [[signal]] / [[intent]] for the temporal fields.
