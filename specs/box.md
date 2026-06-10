---
tags: cyber, core
alias: box, conviction box, boxes
crystal-type: entity
crystal-domain: cyber
---
# box

a box is conviction made concrete: `amount` units of a [[coin]] denomination, owned by a [[neuron]], bound to a [[cyberlink]] record.

$$\text{box} \;=\; (coin,\; amount) \;\in\; \text{Coin} \times \mathbb{R}_+$$

the pair is the unit. [[amount]] alone is a context-free number; a [[coin]] alone is a denomination. neither is conviction until paired. the pair is the conviction, and the conviction is the box.

a [[cyberlink]] is a [[link]] carrying a box, with a [[valence]]:

$$(from,\; to,\; coin,\; amount,\; v) \;\longrightarrow\; \text{link}(from,\,to) + \underbrace{(coin,\,amount)}_{\text{box}} + \underbrace{v}_{\text{valence}}$$

(a non-fungible transfer is the special case where the box holds a [[card]] with `amount = 1` rather than a coin — the same move, a unique token instead of a fungible quantity.)

## moving a box

creating a cyberlink is a transaction: the author moves a box from the source `from` to a new output bound to the cyberlink record. value always moves from one object to another — a box leaves one place and arrives in another, never appears from nothing. this is the whole model: boxes move.

```
from ──[ box = (token, a) ]──▶ output bound to ℓ
```

## lifecycle

a box bound to a record can itself be spent:

| operation | what moves | what stays |
|-----------|-----------|------------|
| create   | `a·token` from author's wallet → output bound to `ℓ` | the assertion enters `L` |
| transfer | the box → a new owner | the structural record stays in `L`; beneficial ownership moves |
| withdraw | the box → back to the author's wallet | the economic position closes; the record remains |
| spend    | a nullifier is published, retiring the box | double-spend = structural rejection |

transfer is how a [[card]]'s transferability works at the protocol level. withdraw is how economic `delete` works — the assertion is permanent, the position is liquid.

## the box is not the assertion

| | non-fungible | fungible |
|---|---|---|
| object | the [[card]] — 5-tuple content + signal provenance | the box — `(token, a)` |
| meaning | the assertion, permanent (axiom A3) | the economic position, liquid |

the assertion cannot move or close; the box can be transferred, withdrawn, or spent. the two coexist in one record and are separable.

## what a box weighs in the graph

a box's magnitude is the economic force that shapes the [[tri-kernel]] fixed point `φ*`. it feeds the adjacency operator:

$$A_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

where `r: TokenId → ℝ₊` converts denomination to a common scale. higher box value → higher edge weight → more [[focus]] through the link → higher [[cyberank]] for the target.

with [[karma]] and [[inversely coupled bonding surface|ICBS]] markets active, the box is one of three factors in effective adjacency:

$$A^{\text{eff}}_{pq} = \sum_\ell a(\ell) \cdot \kappa(\nu(\ell)) \cdot f(m(\ell))$$

a large box with low [[karma]] or low market price contributes little — all three must align.

## costly signaling

every box makes a [[cyberlink]] a [[costly signal]]. a [[neuron]] with finite [[will]] and [[tokens]] must choose where to place boxes — directing value to one claim directs it away from all others. this scarcity is what makes the [[cybergraph]] accumulate weighted commitments rather than cheap assertions.

## yield

a box earns in proportion to how much its target gains [[focus]]:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\phi^*(to, t)\, dt$$

where `w(t)` includes the box value `a`. the first-mover premium applies: the same box placed early (when `φ*(to)` is low) earns more than placed late. early discovery is maximally rewarded.

## the conviction spectrum

| box value | meaning |
|-----------|---------|
| `a = 0`   | no box. bare assertion — structural presence, no economic exposure |
| `a` small | low-conviction box. the neuron acknowledges the connection but risks little |
| `a` large | high-conviction box. the neuron bets real capital that this link matters |
| `a` → burn | permanent conviction — tokens destroyed for [[eternal cyberlinks]], an unspendable box |

## authenticated realization

the conceptual box is realized in [[bbg]] as a mutator-set record: a commitment in the private polynomial `A(x)` (the box exists) and a nullifier in `N(x)` (the box was spent). `bbg`'s `BoxMove { nullifier, commitment }` is exactly one box leaving (`nullifier`) and one arriving (`commitment`). double-spend is `N(n) = 0` — structural rejection, no balance check needed. see [[cyber/bbg]] `box structure` for the cryptographic mechanics.

see [[token]] for the denomination `τ` · [[amount]] for the magnitude `a` · [[cyberlink]] for the record the box binds to · [[will]] for the budget that constrains where boxes go · [[staking]] for box placement as a graph act.
