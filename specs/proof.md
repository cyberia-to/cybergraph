---
tags: cyber, cybergraph
crystal-type: entity
crystal-domain: cyber
alias: proof, signal proof, submission proof, zheng proof
---
# proof

the σ field of a [[signal]]: a single [[zheng]] proof. it is the signal's **evidence** — one proof, covering the whole batch atomically, that lets anyone holding only the root accept the signal without re-executing it and without trusting the sender.

## what it is

σ is produced by the neuron at sealing, taken over the [[nox]] execution of the [[intent]]'s scope, and verified by anyone in $O(\log n)$ via `decide(σ)`. it is a **content proof** — it attests the signal's computation is correct against the committed root. it is not the identity signature (separate), and not a chain-position fact (that is [[order]]'s).

## what it attests

σ covers, atomically, the validity conditions a signal must satisfy — signature, cyberlink validity, box movements, conservation, focus sufficiency, impulse correctness. the precise list of *what must hold*, and how each condition is checked, is the validation page: see [[validate]]. **proof is the object; validation is the check.** a few conditions sit outside σ (availability, ordering, network) and are listed there too.

## boundary

cybergraph never constructs proofs — that is [[zheng]]'s (the circuit, the proof system, the [[zheng/proof-types]] taxonomy). cybergraph only verifies; checking σ is the [[validate]] verb.

see [[validate]] for the conditions σ must attest · [[zheng]] for construction · [[signal]] for the field's slot · [[intent]] for what σ is taken over.
