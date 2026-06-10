---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: validation, submit validation, the submit gate
---
# validation (routing)

validation is not a field — it is cybergraph's umbrella operation. when a signal arrives (`link` or `seal`), cybergraph routes each check to the component that owns its complete criterion, aggregates the verdict, and only then calls `bbg.insert`. cybergraph holds no complex check logic of its own; it routes, and rejects on the first failure.

```
cybergraph.submit(signal) -> Result<root, ValidationError>:
  route every check below → accept or reject
  on accept:  bbg.insert(signal) → new root ; advance the signal chain
  on reject:  drop, return the error ; bbg never sees an invalid signal
```

## the routing table

each row is a check cybergraph performs by routing to an owner. the *complete* criterion lives in that owner; cybergraph only asks the question and reads the answer.

| check | what must hold | routed to (complete criterion) |
|---|---|---|
| A1 content addressing | particles in $\vec\ell$ resolve to their preimage | [[hemera]] (the hash) |
| A2 authentication | the [[proof]] σ covers the whole signal: cyberlink validity, box movements, and $\Delta\phi^*$ correctness | [[zheng]] (verify) + [[tru]] ($\Delta\phi^*$ correctness criterion) |
| A3 append-only / equivocation | step is sequential, prev matches, no fork from this ν | [[sync]] (hash chain, VDF) |
| network routing | the resolved $\mathit{net}$ matches the node's network | [[network]] |
| focus sufficiency | $\text{focus}(\nu) \geq \sum_\ell \text{cost}(\ell)$ | [[tru]] (focus accounting) |
| box ownership | each spent [[box]] is unspent ($N(n)\neq 0$) and owned by ν ($A(c)$ resolves to ν) | [[bbg]] (mutator set A(x)/N(x)) |
| conservation | $\sum$ box inputs $= \sum$ outputs $+$ fee | [[tru]] / proved atomically by σ |
| temporal ordering | inception/sealing consistent with the chain | [[sync]] |

A4 (entry) and A6 (homoiconicity) are structural — they hold by construction of the graph (see [[cybergraph]]). A2's proof is one σ covering everything atomically; the verifier runs `decide(σ)` in $O(\log n)$ with no re-execution.

## error taxonomy

cybergraph's reject reasons — the verdicts the umbrella returns:

| error | cause |
|---|---|
| `Unresolved` | A1 — a particle does not resolve to its preimage |
| `BadProof` | A2 — σ fails verification |
| `Equivocation` / `StepNotSequential` / `PrevMismatch` | A3 — chain violations (from [[sync]]) |
| `WrongNetwork` | net does not match the serving node |
| `InsufficientFocus` | focus sufficiency fails |
| `BadOwnership` / `DoubleSpend` | box ownership / N(x) reuse (the last from [[bbg]]) |

## signal lifecycle

```
signal arrives
  │
  ├─ route checks A1–A3, network, focus, box ownership, conservation, temporal
  │    first failure → ValidationError, signal dropped (bbg never sees it)
  ▼
bbg.insert(signal)        apply cyberlinks, extend A(x)/N(x), recompute root
  ▼
cybergraph advances the signal chain (prev = H(signal))   [[sync]]
```

## proof boundary

cybergraph validates the proof; it never constructs one. construction, the circuit, and the proof-type taxonomy are [[zheng]]'s — see [[proof]] for the field and [[zheng/proof-types]] for the taxonomy.

see [[sync]] for ordering, [[bbg/state]] for the insert effects, [[bbg/privacy]] for the mutator set, [[tru]] for the focus/conservation criteria.
