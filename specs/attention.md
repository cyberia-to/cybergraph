---
tags: cyber, cybergraph
crystal-type: measure
crystal-domain: cyber
alias: attention read, attention query
---
# attention (read)

a read-emergent: it appears when you read the structure. `query(from, to)` returns how much [[focus]] a [[neuron]] projects onto a target.

```
query(from: ν, to: p)  →  attention(ν → p)        one neuron's weight on one target
query(from: ν)         →  attention distribution across all targets
query(to: p)           →  all neurons attending to p (weighted backlink set)
```

cybergraph exposes the read and names the two write paths that produce it — [[will]] (broad) and per-link conviction (the [[box]] in [[amount]]). the complete definition of the quantity — the focus projection and how individual attention aggregates into collective $\phi^*$ — is the [[tru]]'s. see [[attention]] for the full specification.

see [[axon]] for the other emergent · [[query]] for the read interface.
