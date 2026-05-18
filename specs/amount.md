# amount

the fourth field of a [[cyberlink]]: $a \in \mathbb{R}_+$. the quantity of [[token]] staked as conviction on the assertion

amount is what separates a structural claim from an economic commitment. a link with $a = 0$ is a bare assertion — it exists in the graph but carries no economic weight. a link with $a > 0$ is a funded position — the [[neuron]] has moved real [[tokens]] into a conviction box bound to the claim

## definition

$$a(\ell) \in \mathbb{R}_+$$

the non-negative stake amount in denomination $\tau(\ell)$. combined with [[token]], the pair $(\tau, a)$ forms the conviction — the economic layer of the [[cyberlink]]

## role in adjacency

amount feeds directly into the adjacency operator:

$$A_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

higher amount → higher edge weight → more [[focus]] flows through the link → higher [[cyberank]] for the target [[particle]]. amount is the economic force that shapes the [[tri-kernel]] fixed point $\phi^*$

## role in effective adjacency

with [[karma]] and [[inversely coupled bonding surface|ICBS]] markets active:

$$A^{\text{eff}}_{pq} = \sum_\ell a(\ell) \cdot \kappa(\nu(\ell)) \cdot f(m(\ell))$$

amount is one of three factors. a link with high amount but low [[karma]] or low market price contributes little. all three must align for maximum effective weight

## conviction as box

creating a cyberlink moves $a$ tokens of denomination $\tau$ from the author's wallet box to a new output bound to the cyberlink record. this box is:

- transferable — spend to a new owner. the structural link remains; beneficial ownership moves
- withdrawable — spend back to the author's wallet. the economic position closes; the structural record remains
- yield-bearing — earns proportionally to $\Delta\phi^*(q, t)$ over time

the conviction box is the financial instrument. the cyberlink record is the assertion. they are separable

## amount and costly signaling

amount makes every [[cyberlink]] a [[costly signal]]. a [[neuron]] with finite [[will]] and [[tokens]] must choose where to allocate. each allocation is a real economic decision — directing stake to one claim is directing it away from all others. this scarcity ensures the [[cybergraph]] accumulates weighted commitments rather than cheap assertions

## the amount spectrum

| amount | meaning |
|--------|---------|
| $a = 0$ | bare assertion. structural presence, no economic exposure |
| $a$ small | low-conviction signal. the neuron acknowledges the connection but risks little |
| $a$ large | high-conviction position. the neuron bets real capital that this link matters |
| $a$ → burn | permanent conviction. tokens destroyed for [[eternal cyberlinks]] — irreversible commitment |

## reward proportionality

link yield is proportional to conviction weight:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\phi^*(q, t)\, dt$$

where $w(t)$ includes $a(\ell)$. higher amount → higher yield when the target [[particle]] gains [[focus]]. the first-mover premium applies: the same amount staked early (when $\phi^*(q)$ is low) earns more than staked late (when $\phi^*(q)$ is already high)

see [[cyberlink]] for the full 5-tuple. see [[token]] for the denomination. see [[valence]] for the epistemic prediction. see [[will]] for the budget that constrains amount allocation
