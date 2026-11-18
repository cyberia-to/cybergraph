---
tags: cyber, cybergraph, signal, intent, project
crystal-type: process
crystal-domain: cyber
status: proposed
date: 2026-11-16
product: soft3
---
# the signal is multiparty

one signal shape everywhere in the stack, in which the subjects are the authors of its links: per-author authorization and ownership, one conservation equation over the union of their boxes, one `prev` and one equivocation check per subject, and a whole-signal reject — so that two neurons acting together is the ordinary signal, not a protocol beside it.

## why

cybergraph's specs described a signal as one neuron's commitment: one $\nu$ in `specs/signal.md`, one signature and one owner in `specs/validate.md`, one chain head in `specs/order.md`. every act of two neurons — a trade, a settlement, a demand and its filling — then needed a mechanism beside the signal (cascade, an escrow, a second signal), and the atomicity each of those acts needs had no owner. the multiparty signal makes the joint act the base case and leaves the single-subject signal as what it always was, the common one.

## what it covers

| # | sub-task | where | state |
|---|---|---|---|
| 1 | the multiparty signal in cybergraph's specs — $\vec\nu$, per-author checks, conservation over the union, per-subject `prev` and equivocation, atomic reject | `specs/signal.md`, `specs/validate.md`, `specs/order.md`, `specs/README.md` | [cybergraph#24](https://github.com/cyberia-to/cybergraph/pull/24) |
| 2 | the intent demand face — the `sealer` and `offer` scope fields, the signal's $i$ field, the seal nullifier in [[bbg]], the `BadSeal` reject | `specs/intent.md`, `specs/validate.md` | written in cybergraph#24 and taken out of it on review, so that 1 can merge alone; the text as written is in commit `fe8948e`. its own pull request, after the open question below |
| 3 | the chain envelope's multi-signer form — one `from` and one NSIG1 per transaction today | soft3 `specs/chain.md` §2.1 | not started |
| 4 | the implementation under the specs, once 1–3 are decided | cybergraph, foculus, bbg | not started |

the project is over when a signal of many subjects is one object from the spec down to the wire.

sub-task 2 has one question open before it can be written again: what authorizes the declarer's offer leg. every subject of a signal signs the whole signal, and on the demand face the sealing signal is built by the filler, over links and an impulse the declarer never saw — so the declarer cannot sign it and cannot be an ordinary subject. the candidate answer is the intent's identity proof standing as the declarer's standing authorization over the box its scope committed; the alternative is that the declarer is never a subject and the offer moves some other way. whichever is chosen must be written before the demand face is proposed again.

## status

`proposed` — the decision is the owner's. the pull request that opened this work cites an instruction of the owner of 2026-11-16, *the idea of signal is truly multiparty by design — upgrade*; that instruction is recorded nowhere in the tree, the registry or any project page, so this page is the declaration the work rests on until the owner accepts or rejects it. nothing here is implemented: every sub-task above is specification.

see [[cybergraph/specs/signal|signal]] · [[cybergraph/specs/intent|intent]] · [[cybergraph/specs/validate|validate]] · [[cybergraph/specs/order|order]]
