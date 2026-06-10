---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: validate, validation, the gate
---
# validate

the first verb of cybergraph's mechanism: verify a signal's [[proof]] field σ against the current root, and accept or reject.

```
validate(signal) → ok | reject
  verify σ covers the whole signal atomically (links, boxes, conservation,
  focus, impulse) against the committed root
```

cybergraph does not construct the proof and does not re-execute anything — it calls `decide(σ)`, $O(\log n)$, and reads the verdict. construction, the circuit, and the proof taxonomy are [[zheng]]'s; the conditions σ attests — content addressing, signature, box unspent/owned, conservation, focus sufficiency, impulse correctness — are spelled out in the [[proof]] field. any one failing rejects the signal.

## reject reasons

every criterion that can fail is a reject reason — the verdicts the gate returns:

| reason | failed criterion | from |
|---|---|---|
| `BadProof` | σ does not verify, or any condition it attests fails | [[proof]] / [[zheng]] |
| `Unresolved` | a particle does not resolve to its preimage | [[proof]] (content addressing) |
| `InsufficientFocus` | $\text{focus}(\nu) < \sum_\ell \text{cost}(\ell)$ | [[proof]] (focus sufficiency) |
| `BadOwnership` / `DoubleSpend` | box not owned by ν / nullifier reused | [[proof]] / [[bbg]] |
| `Equivocation` / `StepNotSequential` / `PrevMismatch` | chain violations | [[order]] / [[sync]] |
| `WrongNetwork` | resolved net ≠ the serving node's | [[network]] |

reject is total: bbg never sees an invalid signal.

mechanism, not decision: validate is a fixed check, the same for every signal. the loop that *decided* to produce this signal — and what to compute before sealing it — is the [[soma]] control loop above cybergraph, not part of this verb.

see [[proof]] for the field σ attests · [[order]] for the next verb · [[zheng]] for verification.
