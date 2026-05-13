---
tags: cyber, cip
crystal-type: entity
crystal-domain: cyber
alias: proof validation, signal proof, submission proof, zheng proof
---
# proof

the $\sigma$ field of a [[signal]]. a single [[zheng]] proof that `submit()` validates before any [[cyberlink]] enters the graph.

## what cybergraph validates

cybergraph does not construct proofs — that is [[zheng]]'s domain. it enforces one rule at the submission boundary:

> a signal is accepted if and only if $\sigma$ is a valid [[zheng]] proof covering the entire signal atomically

the proof must certify three things together in one verification:

| what | why |
|------|-----|
| every [[cyberlink]] in $\vec\ell$ is valid | correct signatures, valid particle references, non-expired |
| all conviction UTXO movements are backed | each $(τ, a)$ spend has an unspent output; new UTXOs are created |
| the [[impulse]] $\Delta\phi^*$ is correct | tri-kernel computation against current BBG root |

one proof for all three. any verifier runs `decide(σ)` in $O(\log n)$ — no re-execution.

## boundary

cybergraph owns: the validation check (`verify(σ, signal, bbg_root) → bool`)

[[zheng]] owns: proof construction, the circuit, the proof system, the taxonomy of all proof types

see [[zheng/specs/proof-types]] for the full taxonomy of all proof types the protocol generates.
