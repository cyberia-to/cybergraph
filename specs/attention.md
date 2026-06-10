---
alias: cyber/attention query, attention query
tags: cyber, cybergraph
crystal-type: measure
crystal-domain: cyber
---
# attention (query)

`query(from, to)` reads how much [[focus]] a [[neuron]] projects onto a target — its [[attention]]. this page is the query-interface view; the quantity itself is a focus concept defined in [[tru]] (see [[tru/specs/attention]]).

a derived read, never a stored field: cybergraph runs the query over committed state, the value comes from tru's focus computation.

```
query(from: ν, to: p)  →  attention(ν → p)        one neuron's weight on one target
query(from: ν)         →  attention distribution across all targets
query(to: p)           →  all neurons attending to p (weighted backlink set)
```

the two write paths that produce attention — [[will]] (broad, auto-distributed) and per-link conviction ([[box]] in the cyberlink's `amount`) — are described in [[staking]]. how individual attention aggregates into collective $\phi^*$ is in [[tru/specs/attention]].
