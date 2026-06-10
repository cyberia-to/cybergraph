---
tags: cyber, docs
alias: cybergraph specs, cybergraph structure
---
# cybergraph specs

cybergraph is exactly its structure, nothing more and nothing less:

```
cybergraph  ←  signals  ←  cyberlinks
```

a cybergraph is built from signals; a signal is built from cyberlinks; each has a fixed, finite set of fields. these specs define those fields — one article per field — plus the two emergents that appear when you read or group the structure ([[axon]], [[attention]]) and the [[query]] interface that reads it. deep mechanics of a field live in the repo that owns them ([[zheng]] proof construction, [[tru]] focus/impulse, [[tok]] tokens / [[plumb]] value ops, [[mudra]] identity, [[sync]] ordering); cybergraph defines the field and references them.

## the structure

- [cybergraph.md](cybergraph.md) — the triple $(P, N, L)$, six axioms, derived adjacency
- [signal.md](signal.md) — the broadcast unit a neuron commits
- [cyberlink.md](cyberlink.md) — the atomic unit a signal carries
- [intent.md](intent.md) — an unsealed signal: declared and identity-proven, not yet sealed

## signal fields

$$s = (\nu,\; \mathit{net},\; \vec\ell,\; \Delta\phi^*,\; \sigma,\; h_0,\; h_1)$$

| field | name | article | deep mechanics |
|---|---|---|---|
| $\nu$ | subject | [neuron.md](neuron.md) | identity in [[mudra]] |
| $\mathit{net}$ | network | [network.md](network.md) | — |
| $\vec\ell$ | links | [cyberlink.md](cyberlink.md) | — |
| $\Delta\phi^*$ | impulse | [impulse.md](impulse.md) | computed/rewarded in [[tru]] |
| $\sigma$ | proof | [proof.md](proof.md) | constructed by [[zheng]] |
| $h_0, h_1$ | inception, sealing | [signal.md](signal.md) / [intent.md](intent.md) | ordering in [[sync]] |

## cyberlink — link + box + valence

the five stored fields group into three concepts: a [[link]] (the edge), a [[box]] (the conviction), a [[valence]] (the prediction).

$$\text{cyberlink} \;=\; \text{link}(from,\,to) \;+\; \text{box}(coin,\,amount) \;+\; valence$$

| concept | fields | article | type / deep mechanics |
|---|---|---|---|
| link | $from, to$ | [link.md](link.md) | [[card]] → [[card]] — a [[particle]] is a knowledge card |
| box | $coin, amount$ | [box.md](box.md) | [[coin]] (token layer [[tok]]) × amount; mutator set in [[bbg]] |
| └ amount | $a$ | [amount.md](amount.md) | the box magnitude |
| └ coin | $token$ | [token.md](token.md) | the denomination (a coin; a card for transfers) |
| valence | $v$ | [valence.md](valence.md) | BTS / markets in [[tru]] |

the endpoints are [[card|cards]]; the content-addressed node a card can be is the [[particle]] ([particle.md](particle.md)).

## emergents — what appears when you read or group the structure

- [axon.md](axon.md) — the bundle of all cyberlinks between two particles (groups links; homoiconic, axiom A6)
- [attention.md](attention.md) — the focus a neuron projects onto a target (a read; `query(from, to)`). complete form in [[tru]]

## process — what cybergraph does (mechanism)

the structure above is what cybergraph *is*; these four verbs are what it *does*. each is a static handoff to one companion — deterministic mechanism, no decisions.

| verb | does | routes to |
|---|---|---|
| [validate.md](validate.md) | verify the [[proof]] σ against the root | [[zheng]] |
| [order.md](order.md) | place in the neuron's chain; reject equivocation | [[sync]] |
| [apply.md](apply.md) | move each [[box]] into state | [[bbg]] |
| [expose.md](expose.md) | answer reads over the committed graph | [[inf]], [[tru]] |

validate→order→apply is the write path (inside `link`/`seal`); expose is the read (`query`).

## what cybergraph does NOT do — orchestration is soma's

the *decision* loop — read an [[intent]], collect recomputed state from [[bbg]], run it through [[nox]], judge what is left to compute, make internal calls, iterate, and only then emit a signal — is dynamic control, not a static pipeline. it falls out, upward, to the [[soma]] cognitive loop. soma *calls* cybergraph's four verbs; it is not one of them. cybergraph is the fast, correct, stateless processor; soma is the mind that drives it.

## staking — placing weight

- [staking.md](staking.md) — the two write paths that produce attention: [[will]] (broad, → tru) and conviction (per-link [[box]])

## query

- [query.md](query.md) — the relations cybergraph exposes to [[inf]]. schema only; the language lives in [[inf/README]]
