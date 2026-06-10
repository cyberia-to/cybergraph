---
tags: cyber, cybergraph
crystal-type: process
crystal-domain: cyber
alias: validate, validation, the gate
---
# validate

the first verb of cybergraph's mechanism, and the gate: decide whether a [[signal]] may enter the graph. nothing invalid reaches [[bbg]]. verifying the [[proof]] σ is the core of it; a few conditions sit outside σ and are checked alongside.

## what must be validated

the full set of conditions a signal must satisfy. each is checked one way — attested by the [[proof]] σ, by an availability proof, by the chain, or by routing — and each has a complete-form owner. cybergraph runs none of the deep logic; it asks each owner and aggregates.

| criterion | what must hold | checked by | owner |
|---|---|---|---|
| availability | each referenced particle's content is published and retrievable (and hashes to its id) | availability proof (DAS) | [[sync]] (DAS), [[bbg]] files |
| signature | valid signature from ν over $H(\vec\ell \,\|\, \Delta\phi^* \,\|\, t)$ | σ | [[mudra]] |
| cyberlink validity | each $\ell \in \vec\ell$ has valid particle references and is well-formed | σ | [[hemera]] |
| box unspent | each spent [[box]]'s nullifier is live — $N(n) \neq 0$ (opening into $N(x)$) | σ | [[bbg]] |
| box ownership | the spender owns it — $A(c)$ resolves to ν's key (opening into $A(x)$) | σ | [[bbg]] |
| conservation | $\sum$ box inputs $= \sum$ outputs $+$ fee | σ | [[tru]] |
| focus sufficiency | $\text{focus}(\nu) \geq \sum_\ell \text{cost}(\ell)$, $\text{cost}(\ell) = a(\ell) + \text{base\_fee}$, against $\text{BBG\_poly}(\text{neurons}, \nu, t)$ | σ | [[tru]] |
| impulse correctness | $\Delta\phi^*$ is the true tri-kernel shift against the current root | σ | [[tru]] ([[impulse]]) |
| ordering | height = current, prev links, no equivocation, valid VDF | the chain | [[order]] → [[sync]] |
| network | the resolved $\mathit{net}$ matches the serving node | routing | [[network]] |

the bulk is one verification — `decide(σ)`, $O(\log n)$, no re-execution. availability, ordering, and network are the conditions σ does not cover; their detail lives in [[order]] and [[network]], availability in the [[sync]] DA layer.

box unspent is the structural double-spend invariant ($N(n)=0 \Rightarrow$ reject); box ownership is the semantic layer on top — the *right* neuron is spending, not just that the box is unspent.

## reject reasons

every criterion that can fail is a reject reason — the verdicts the gate returns:

| reason | failed criterion |
|---|---|
| `Unavailable` | a referenced particle's content is not available |
| `BadProof` | σ does not verify, or any condition it attests fails |
| `InsufficientFocus` | focus sufficiency fails |
| `BadOwnership` / `DoubleSpend` | box not owned by ν / nullifier reused |
| `Equivocation` / `StepNotSequential` / `PrevMismatch` | chain violations |
| `WrongNetwork` | resolved net ≠ the serving node's |

reject is total: bbg never sees an invalid signal.

## mechanism, not decision

validate is a fixed gate — the same conditions for every signal. the loop that *decided* to produce this signal, and what to compute before sealing it, is the [[soma]] control loop above cybergraph, not part of this verb.

see [[proof]] for the σ object · [[order]] / [[network]] for the non-σ checks · [[zheng]] for verification.
