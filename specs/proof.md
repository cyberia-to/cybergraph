---
tags: cyber, cybergraph
crystal-type: entity
crystal-domain: cyber
alias: proof, signal proof, submission proof, zheng proof
---
# proof

the σ field of a [[signal]]: a single [[zheng]] proof covering the whole batch atomically. it is what makes a signal checkable without re-execution — a verifier holding only the root decides `decide(σ)` in $O(\log n)$.

## what σ attests

one verification certifies all of these together — they are what the field carries, not separate checks:

| attested | meaning | complete criterion in |
|---|---|---|
| cyberlink validity | valid particle references, well-formed links | [[hemera]] |
| box ownership + movement | each spent [[box]] is unspent and owned by ν; outputs created | [[bbg]] (mutator set) |
| conservation | $\sum$ box inputs $= \sum$ outputs $+$ fee | [[tru]] |
| focus sufficiency | $\text{focus}(\nu) \geq \sum_\ell \text{cost}(\ell)$ | [[tru]] |
| impulse correctness | $\Delta\phi^*$ is the true tri-kernel shift against the root | [[tru]] ([[impulse]]) |

## boundary

cybergraph never constructs proofs — that is [[zheng]]'s (the circuit, the proof system, the [[zheng/proof-types]] taxonomy). checking this field is the [[validate]] verb; verifying σ **is** the signal's validation. the one thing σ does not cover — chain position — is the [[order]] verb's, routed to [[sync]].

see [[validate]] for the verb that checks σ · [[zheng]] for construction · [[signal]] for the field's slot.
