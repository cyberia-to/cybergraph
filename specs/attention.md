---
alias: attention, neuron attention
tags: cyber, cybergraph, core
crystal-type: measure
crystal-domain: cyber
---
# attention

a read-emergent: how much [[focus]] a [[neuron]] projects onto a target [[particle]] or [[axon]]. not a field of the structure — it appears when you read the structure. `query(from, to)` returns it.

```
query(from: ν, to: p)  →  attention(ν → p)        one neuron's weight on one target
query(from: ν)         →  attention distribution across all targets
query(to: p)           →  all neurons attending to p (weighted backlink set)
```

attention is written by two paths the structure already carries: [[will]] (broad, auto-distributed across a neuron's links) and per-link conviction (the [[box]] in a cyberlink's [[amount]]). cybergraph exposes the read; the focus quantity it returns — and how individual attention aggregates into collective $\phi^*$ — is computed by the [[tru]] (see [[focus-flow]]).

see [[axon]] for the other emergent · [[query]] for the read interface · [[tru]] for the focus computation.
