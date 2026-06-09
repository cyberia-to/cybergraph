# cyberlink

the atomic unit of the [[cybergraph]]. two [[tokens]] bound by a staked assertion — from and to can be [[particles]], [[neurons]], [[cards]], or [[coins]]. every cyberlink is simultaneously an economic act, a semantic assertion, and an epistemic prediction. who made it and when belong to the [[signal]] that carries it.

## the five fields

$$\ell \;=\; (from,\; to,\; token,\; a,\; v) \;\in\; \text{TokenId} \times \text{TokenId} \times \text{TokenId} \times \mathbb{R}_{+} \times \{-1,\,0,\,+1\}$$

| field | name | type | layer | semantics | question |
|-------|------|------|-------|-----------|----------|
| $from$ | source | $\text{TokenId}$ | structural | source token — authorizes the spend (plumb auth on this token's leaf) | what initiates? |
| $to$ | destination | $\text{TokenId}$ | structural | destination token — receives the moved token | what receives? |
| $token$ | moved token | $\text{TokenId}$ | economic | what moves (coin denomination or card id) | what moves? |
| $a$ | [[amount]] | $\mathbb{R}_+$ | economic | stake amount (1 for non-fungible cards) | how much conviction? |
| $v$ | [[valence]] | $\{-1,0,+1\}$ | epistemic | [[Bayesian Truth Serum\|BTS]] meta-prediction | what is the epistemic prediction? |

three layers in one atomic record. structural $(from, to)$ is binary — the connection either exists or it does not. epistemic $v$ is ternary — the neuron's prediction of how the [[inversely coupled bonding surface|ICBS]] market on this edge will converge. economic $(token, a)$ is continuous over $\mathbb{R}_+$. see [[two three paradox]] for why this layering is not arbitrary

conviction = ($token$, $a$): the pair that turns an assertion into a bet. token selects what moves, [[amount]] declares the stake. a link with zero conviction is structurally identical to a link with maximum conviction — the structural layer is binary. the conviction layer prices it

cyberlinks are bundled into [[signal|signals]] for broadcast. the [[signal]] adds the provenance layer: the signing [[neuron]] $\nu$, block height $t$, the destination [[network]] $\mathit{net}$, an [[cyber/impulse]] ($\Delta\phi^*$ — the proven [[focus]] shift), and a recursive [[stark]] proof covering the entire batch. the network is a signal-envelope concern — where the assertion is delivered — not a cyberlink field.

the [[cybergraph]] is append-only. [[time]] (block height from the containing signal) distinguishes every record: the same author linking from→to in two signals at $t_1$ and $t_2 > t_1$ produces two separate entries in $L$. this enables reinforcement (higher [[amount]] on a new record), [[valence]] updates (new $v$ at a new signal), and multi-denomination staking (same structural link in different tokens)

see [[token]] for token types and standards. see [[amount]] for conviction mechanics. see [[valence]] for epistemic prediction. see [[time]] for temporal ordering.

## examples

```
(neuron_alice, particle_A, CYB, 500, +1)      alice stakes conviction on a particle
(particle_A, particle_B, CYB, 100, +1)        particle routes value to related particle
(neuron_alice, neuron_bob, card_X, 1, 0)      card transfer between neurons
(neuron_alice, pool_card, HYDROGEN, 1000, 0)  stake into a liquidity pool
(particle_A, particle_B, particle_A, 1, +1)  particle asserts relation to another
```

the from/to pair forms the structural edge in the knowledge graph. the token/amount pair is the economic weight. valence is the epistemic layer.

## box model

conviction is a box. creating a cyberlink is a transaction: the author moves $a$ units of $token$ from the source ($from$) box to a new output bound to the cyberlink record. funds always move from one object to another

the conviction output can itself be spent:

- transfer: spend the conviction box to a new owner. the structural record stays in $L$; beneficial ownership moves. this is how the card's transferability operates at the protocol level
- withdraw: spend the conviction box back to the author's wallet. the economic position closes. the structural record remains

the non-fungibility of the card (5-tuple content + signal provenance) and the fungibility of the [[token]] (transferable box) coexist: the assertion is non-fungible, the economic position is a standard box

## CRUD operations

the append-only graph expresses all four operations through cyberlinks:

| operation | cyberlink action | what changes |
|-----------|-----------------|--------------|
| create | first record for structural pair $(from, to)$ | relation enters $L$ |
| read | query $\phi^*$ at any block — no link required | nothing |
| update | new record with new $(token, a, v)$ for the same pair | any mutable dimension |
| delete | withdraw conviction box + new record with $v = -1$ | economic position closed, epistemic signal negated |

the two mutable dimensions — epistemic ($v$) and economic ($a$) — vary independently. temporal context comes from the containing signal. every combination is meaningful:

| $v$ | $a$ | reading |
|-----|-----|---------|
| $+1$ | high | funded affirmation — bet the market confirms |
| $+1$ | zero | unfunded affirmation — structural + epistemic signal, no economic exposure |
| $0$ | high | funded agnostic — stake without prediction |
| $0$ | zero | bare assertion — structural fact only |
| $-1$ | high | funded short — bet the market rejects |
| $-1$ | zero | logical retraction — epistemic negation, no economic exposure |

$v = -1$ is the neuron's prediction that the [[inversely coupled bonding surface|ICBS]] market on this edge will converge to FALSE — a funded short when $a > 0$, a pure retraction when $a = 0$

delete in the graph is never erasure. the assertion $(from, to)$ committed at $t_{\text{first}}$ stays in $L$ permanently. economic close and epistemic retraction are separable operations

## the card

every cyberlink is also a card — an epistemic asset with four properties:

immutable — axiom A3 (append-only) guarantees the record is permanent once published. the assertion cannot be altered or retracted. the author's conviction, [[valence]], and timestamp are locked into the graph's history forever

unique — each card is non-fungible. two signals with the same 5-tuple content submitted at different block heights produce distinct records in $L$ — the signal's $\nu$ and $t$ are what identify them. the 5-tuple is the assertion; the signal is the provenance.

transferable — ownership of a cyberlink and its yield rights can be transferred between [[neurons]]. the structural record stays in $L$ forever; beneficial ownership moves

yield-bearing — a cyberlink earns in proportion to how much the target gains [[focus]]:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\phi^*(to, t)\, dt$$

where $w(t)$ is the conviction weight at [[time]] $t$ and $\Delta\phi^*(to, t)$ is the increment in the destination token's focus. early discovery is maximally rewarded; late consensus-following earns little

## edge labeling

a cyberlink has no built-in type field. labeling works through the graph itself: every directed edge induces an [[axon]]-[[particle]] via axiom A6 ($H(from, to) \in P$). to label an edge, create a cyberlink from a type-[[particle]] to the [[axon]]-[[particle]]:

```
A --cyberlink--> B                  the assertion
"is-a" --cyberlink--> axon(A, B)    the label
```

any [[particle]] can serve as a label: `is-a`, `contradicts`, `extends`, `cites`, `created-by`. the label itself has [[cyberank]], [[karma]], market price — the graph weights the importance of relation types the same way it weights everything else
