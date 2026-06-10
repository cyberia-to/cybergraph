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

cybergraph does not construct the proof and does not re-execute anything — it calls `decide(σ)`, $O(\log n)$, and reads the verdict. construction, the circuit, and the proof taxonomy are [[zheng]]'s; what σ attests is the [[proof]] field.

mechanism, not decision: validate is a fixed check, the same for every signal. the loop that *decided* to produce this signal — and what to compute before sealing it — is the [[soma]] control loop above cybergraph, not part of this verb.

see [[proof]] for the field σ attests · [[order]] for the next verb · [[zheng]] for verification.
