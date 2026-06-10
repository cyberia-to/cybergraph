---
tags: cyber, docs
alias: cybergraph specs, cybergraph structure
---
# cybergraph specs

cybergraph is exactly its structure, nothing more and nothing less:

```
cybergraph  ←  signals  ←  cyberlinks
```

a cybergraph is built from signals; a signal is built from cyberlinks; each has a fixed, finite set of fields. these specs define those fields — one article per field — plus the two emergents that appear when you read or group the structure ([[axon]], [[attention]]) and the [[query]] interface that reads it. deep mechanics of a field live in the repo that owns them ([[zheng]] proof construction, [[tru]] focus/impulse, [[plumb]] value ops, [[mudra]] identity, [[sync]] ordering); cybergraph defines the field and references them.

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

## cyberlink fields

$$\ell = (from,\; to,\; token,\; a,\; v) \;\longrightarrow\; (from,\; to,\; \underbrace{(token,\,a)}_{\text{box}},\; v)$$

| field | name | article | deep mechanics |
|---|---|---|---|
| $from, to$ | source, destination | [particle.md](particle.md) | hash in [[hemera]] |
| $token$ | denomination | [token.md](token.md) | value layer in [[plumb]] |
| $a$ | amount | [amount.md](amount.md) | — |
| $(token, a)$ | box | [box.md](box.md) | mutator set in [[bbg]] |
| $v$ | valence | [valence.md](valence.md) | BTS/markets in [[tru]] |

## emergents — what appears when you read or group the structure

- [axon.md](axon.md) — the bundle of all cyberlinks between two particles (groups links; homoiconic, axiom A6)
- [attention.md](attention.md) — the focus a neuron projects onto a target (a read; `query(from, to)`). complete form in [[tru]]

## the umbrella operation — routing

cybergraph is umbrella and routing: it names the checks and routes each to the component that owns the complete criterion.

- [validation.md](validation.md) — the submit gate. routes A1–A6 + focus/box/conservation checks to [[hemera]], [[zheng]], [[sync]], [[tru]], [[bbg]]; aggregates the verdict; then `bbg.insert`. not a field — the routing operation itself.

## staking — placing weight

- [staking.md](staking.md) — the two write paths that produce attention: [[will]] (broad, → tru) and conviction (per-link [[box]])

## query

- [query.md](query.md) — the relations cybergraph exposes to [[inf]]. schema only; the language lives in [[inf/README]]
