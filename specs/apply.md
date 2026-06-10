---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: apply, commit, state application
---
# apply

the third verb: move each cyberlink's [[box]] into authenticated state.

```
apply(signal) → new root
  for each cyberlink: move the box (coin, amount) into state
    particle energy, axon weights, focus debit, box ownership
  extend A(x) / N(x) ; recompute the root
```

cybergraph routes the state mutation to [[bbg]] (`insert`, the mutator set). bbg holds the polynomial state and enforces the structural double-spend invariant ($N(n)=0$); cybergraph hands it the validated, ordered signal and takes back the new root.

mechanism, not decision: apply is deterministic — the same signal yields the same state move. it runs only after [[validate]] and [[order]] pass.

see [[order]] for the prior verb · [[expose]] for reads · [[box]] for what moves · [[bbg]] for the state.
