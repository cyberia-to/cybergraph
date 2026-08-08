---
tags: cyber, docs
alias: cybergraph specs, cybergraph structure
---
# cybergraph specs

cybergraph is exactly its structure, nothing more and nothing less:

```
cybergraph  ←  signals  ←  cyberlinks
```

a cybergraph is built from signals; a signal is built from cyberlinks; each has a fixed, finite set of fields. these specs define those fields — one article per field — plus the two emergents that appear when you read or group the structure ([[axon]], [[attention]]) and the [[query]] interface that reads it. deep mechanics of a field live in the repo that owns them ([[zheng]] proof construction, [[tru]] focus/impulse, [[tok]] tokens / [[plumb]] value ops, [[mudra]] identity, [[foculus]] ordering); cybergraph defines the field and references them.

## the structure

- [cybergraph.md](cybergraph.md) — the triple $(P, N, L)$, six axioms, derived adjacency
- [signal.md](signal.md) — the broadcast unit a neuron commits
- [cyberlink.md](cyberlink.md) — the atomic unit a signal carries
- [intent.md](intent.md) — an unsealed signal: declared and identity-proven, not yet sealed

## signal fields

$$s = (\nu,\; \mathit{net},\; \vec\ell,\; \Delta\phi^*,\; \sigma,\; h_0,\; h_1)$$

| name | fields | deep mechanics |
|---|---|---|
| [subject](neuron.md) | $\nu$ | identity in [[mudra]] |
| [network](network.md) | $\mathit{net}$ | — |
| [links](cyberlink.md) | $\vec\ell$ | — |
| [impulse](impulse.md) | $\Delta\phi^*$ | computed/rewarded in [[tru]] |
| [proof](proof.md) | $\sigma$ | constructed by [[zheng]] |
| [inception, sealing](signal.md) | $h_0, h_1$ | ordering in [[foculus]]; pre-seal state in [[intent]] |

## cyberlink — link + box + valence

the five stored fields group into three concepts: a [[link]] (the edge), a [[box]] (the conviction), a [[valence]] (the prediction).

$$\text{cyberlink} \;=\; \text{link}(from,\,to) \;+\; \text{box}(coin,\,amount) \;+\; valence$$

| name | fields | deep mechanics |
|---|---|---|
| [link](link.md) | $from, to$ | [[card]] → [[card]] |
| └ [from](particle.md) | $from$ | the source — a [[card]] (a [[particle]] is a knowledge card) |
| └ [to](particle.md) | $to$ | the target — a [[card]] |
| [box](box.md) | $coin, amount$ | [[coin]] (token layer [[tok]]) × amount; mutator set in [[bbg]] |
| └ [amount](amount.md) | $a$ | the box magnitude |
| └ [coin](token.md) | $token$ | the denomination (a coin; a card for transfers) |
| [valence](valence.md) | $v$ | BTS / markets in [[tru]] |

## emergents — what appears when you read or group the structure

- [axon.md](axon.md) — the bundle of all cyberlinks between two particles (groups links; homoiconic, axiom A6)
- [attention.md](attention.md) — the focus a neuron projects onto a target (a read; `query(from, to)`). complete form in [[tru]]

## process — what cybergraph does (mechanism)

the structure above is what cybergraph *is*; these four verbs are what it *does*. each is a static handoff to one companion — deterministic mechanism, no decisions.

| verb | does | routes to |
|---|---|---|
| [validate.md](validate.md) | verify the [[proof]] σ against the root | [[zheng]] |
| [order.md](order.md) | place in the neuron's chain; reject equivocation | [[foculus]] |
| [apply.md](apply.md) | move each [[box]] into state | [[bbg]] |
| [expose.md](expose.md) | answer reads over the committed graph | [[inf]], [[tru]] |

validate→order→apply is the write path (inside `link`/`seal`); expose is the read (`query`).

## the seal binding — promise to proof

a signal's life is a promise then a proof: [intent.md](intent.md) `intend(scope)` commits a signed scope; the neuron runs it ([[nox]]) and proves it ([[zheng]]); `seal` commits the result. the commit gate enforces one rule — `seal(i, s)` accepted **iff** `σ(s) ⊢ scope_hash(i)`. that binding makes intent and signal one transaction, and is the alignment property at the commit port. see [intent.md](intent.md) `## completion`.

## what cybergraph does NOT do — orchestration is soma's

cybergraph and [[soma]] are one processor: cybergraph is the dumb half (store · events · commit-gate), soma the smart half (the runtime). the *decision* loop — read an [[intent]], collect recomputed state from [[bbg]], run it through [[nox]], judge what is left to compute, iterate, and only then emit a signal — is dynamic control, not a static pipeline. it falls out, upward, to soma. soma *calls* cybergraph's four verbs and drives the fetch→execute→prove→commit cycle; it is not one of the verbs. cybergraph is fast, correct, stateless; soma is the mind that drives it. see the [whitepaper](../docs/README.md).

## staking — placing weight

- [staking.md](staking.md) — the two write paths that produce attention: [[will]] (broad, → tru) and conviction (per-link [[box]])

## query

- [query.md](query.md) — the relations cybergraph exposes to [[inf]]. schema only; the language lives in [[inf/README]]
