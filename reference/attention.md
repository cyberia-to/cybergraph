---
alias: cyber/attention, attention
tags: cyber, core
crystal-type: measure
crystal-domain: cyber
crystal-size: bridge
---
# attention

the result of `query(from=neuron, to=particle)` — how much [[focus]] a given [[neuron]] projects onto a target [[particle]] or [[axon]].

not a stored field. not a property of the graph. a derived read: attention is what the query engine returns when you ask "how much of this neuron's weight lands here?"

## as a query

```
query(from: ν, to: p)  →  attention(ν → p)
query(from: ν)         →  attention distribution across all targets
query(to: p)           →  all neurons attending to p (backlink set with weights)
```

produced by two write paths inside `submit()`:
- [[staking]] via [[will]] — broad, auto-distributed across all of a neuron's cyberlinks
- fine-tuning — explicit per-target weight in the cyberlink's `amount` field

see [[cybergraph/reference/staking]] for how attention is written. see [[tru/reference/focus]] for how individual attention aggregates into collective $\phi^*$.
