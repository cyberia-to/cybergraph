---
tags: cyber, cybergraph
crystal-type: entity
crystal-domain: cyber
alias: proof, signal proof, submission proof, zheng proof
---
# proof

the σ field of a [[signal]]: a single [[zheng]] proof covering the whole batch atomically. it is what makes a signal checkable without re-execution — a verifier holding only the root decides `decide(σ)` in $O(\log n)$.

## what σ attests — the validation criteria

one verification certifies all of these together. these are *what must hold* for a signal to be valid — not separate checks cybergraph runs, but the conditions the proof carries. each has a precise criterion and a complete-form owner:

| criterion | what must hold | owner |
|---|---|---|
| content addressing | every particle in $\vec\ell$ resolves to its preimage — $p = H(\text{bytes})$ | [[hemera]] |
| signature | valid signature from ν over $H(\vec\ell \,\|\, \Delta\phi^* \,\|\, t)$ | [[mudra]] |
| cyberlink validity | each $\ell \in \vec\ell$ has valid particle references and is well-formed | [[hemera]] |
| box unspent | each spent [[box]]'s nullifier is live — $N(n) \neq 0$ (opening into $N(x)$) | [[bbg]] |
| box ownership | the spender owns it — $A(c)$ resolves to ν's key (opening into $A(x)$) | [[bbg]] |
| conservation | $\sum$ box inputs $= \sum$ outputs $+$ fee, across all box movements | [[tru]] |
| focus sufficiency | $\text{focus}(\nu) \geq \sum_{\ell} \text{cost}(\ell)$, where $\text{cost}(\ell) = a(\ell) + \text{base\_fee}$, against $\text{BBG\_poly}(\text{neurons}, \nu, t)$ | [[tru]] |
| impulse correctness | $\Delta\phi^*$ is the true tri-kernel shift computed against the current root | [[tru]] ([[impulse]]) |

box unspent is the structural double-spend invariant ($N(n)=0 \Rightarrow$ reject); box ownership is the semantic layer on top — the *right* neuron is spending, not just that the box is unspent.

## boundary

cybergraph never constructs proofs — that is [[zheng]]'s (the circuit, the proof system, the [[zheng/proof-types]] taxonomy). checking this field is the [[validate]] verb; verifying σ **is** the signal's validation. the one thing σ does not cover — chain position — is the [[order]] verb's, routed to [[sync]].

see [[validate]] for the verb that checks σ · [[zheng]] for construction · [[signal]] for the field's slot.
