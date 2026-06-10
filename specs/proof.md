---
tags: cyber, cybergraph
crystal-type: entity
crystal-domain: cyber
alias: proof, signal proof, submission proof, zheng proof, validation, submit gate
---
# proof

the $\sigma$ field of a [[signal]]: a single [[zheng]] proof that `submit` verifies before any [[cyberlink]] enters the graph. verifying this field **is** the signal's validation — it covers the whole batch atomically, so there is no separate validation step, only this one check plus two residual routed ones.

cybergraph does not construct proofs — that is [[zheng]]'s. it enforces one rule at the boundary:

> a signal is accepted iff $\sigma$ is a valid [[zheng]] proof over the whole signal, against the current root.

## what σ covers (the validation surface)

one verification certifies all of these together — they are not separate checks cybergraph runs, they are what the proof attests:

| covered by σ | meaning | complete criterion in |
|---|---|---|
| cyberlink validity | valid particle references, well-formed links | [[hemera]] (hashing) |
| box ownership + movement | each spent [[box]] is unspent and owned by ν; outputs created | [[bbg]] (mutator set A(x)/N(x)) |
| conservation | $\sum$ box inputs $= \sum$ outputs $+$ fee | [[tru]] |
| focus sufficiency | $\text{focus}(\nu) \geq \sum_\ell \text{cost}(\ell)$ | [[tru]] |
| impulse correctness | $\Delta\phi^*$ is the true tri-kernel shift against the root | [[tru]] ([[impulse]]) |

`decide(σ)` runs in $O(\log n)$ — no re-execution. one proof, everything at once.

## the two checks σ does not cover

| check | why it's separate | routed to |
|---|---|---|
| chain ordering | step sequential, prev matches, no equivocation — a position-in-chain fact, not a computation | [[sync]] (the signal chain, VDF) |
| network routing | the resolved $\mathit{net}$ matches the serving node | [[network]] |

## the gate

```
submit(signal):
  verify σ                         ← this field; covers the table above
  check chain ordering             → sync
  check network                    → network
  first failure → reject (bbg never sees it)
  all pass → bbg.insert(signal) → new root ; advance the chain
```

reject reasons: `BadProof` (σ fails — the common one), `Equivocation`/`StepNotSequential`/`PrevMismatch` (chain, from [[sync]]), `WrongNetwork`, and `DoubleSpend` (structural, from [[bbg]] at insert).

## boundary

cybergraph owns the check — `verify(σ, signal, root) → bool` — and the routing of the two residual checks. [[zheng]] owns proof construction, the circuit, and the proof-type taxonomy ([[zheng/proof-types]]). cybergraph never constructs, only decides accept/reject.
