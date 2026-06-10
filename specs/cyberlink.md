# cyberlink

the atomic unit of the [[cybergraph]]. a cyberlink **links** two [[card|cards]] and **moves a box** of conviction between them, carrying an epistemic **valence**:

$$\text{cyberlink} \;=\; \underbrace{\text{link}(from,\,to)}_{\text{structural}} \;+\; \underbrace{\text{box}(coin,\,amount)}_{\text{economic}} \;+\; \underbrace{valence}_{\text{epistemic}}$$

| part | fields | type | what it is |
|---|---|---|---|
| [[link]] | $from$, $to$ | [[card]] → [[card]] | the directed edge — what connects to what (a [[particle]] is a knowledge card) |
| [[box]] | $coin$, $amount$ | [[coin]] × $\mathbb{R}_+$ | the conviction moved — a denomination and a quantity |
| [[valence]] | $v$ | $\{-1,0,+1\}$ | the [[Bayesian Truth Serum\|BTS]] prediction |

five stored fields, three concepts. who made it and when belong to the [[signal]] that carries it.

## the three layers

- **structural** — the [[link]] $(from, to)$ is binary: the connection between two [[card|cards]] either exists or it does not. its endpoints authorize the move (plumb auth on each card's leaf).
- **economic** — the [[box]] $(coin, amount)$ is the bet: the [[coin]] selects which denomination moves, [[amount]] declares how much. a link with a zero box is structurally identical to one with a maximum box — the link is binary, the box prices it. (a non-fungible transfer is the special case where the box holds a [[card]] with $amount = 1$ instead of a coin.)
- **epistemic** — the [[valence]] is ternary: the neuron's prediction of how the [[inversely coupled bonding surface|ICBS]] market on this edge converges.

see [[two three paradox]] for why this layering is not arbitrary. see [[box]] for the conviction model, [[link]] for the edge, [[valence]] for the prediction.

cyberlinks are bundled into [[signal|signals]] for broadcast. the [[signal]] adds the provenance layer: the signing [[neuron]] $\nu$, block height $t$, the destination [[network]] $\mathit{net}$, an [[cyber/impulse]] ($\Delta\phi^*$ — the proven [[focus]] shift), and a recursive [[stark]] proof covering the entire batch. the network is a signal-envelope concern — where the assertion is delivered — not a cyberlink field.

the [[cybergraph]] is append-only. [[time]] (block height from the containing signal) distinguishes every record: the same author linking from→to in two signals at $t_1$ and $t_2 > t_1$ produces two separate entries in $L$. this enables reinforcement (higher [[amount]] on a new record), [[valence]] updates (new $v$ at a new signal), and multi-denomination staking (same structural link in different tokens)

see [[token]] for token types and standards. see [[box]] for conviction mechanics. see [[amount]] for the magnitude field. see [[valence]] for epistemic prediction. see [[time]] for temporal ordering.

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

the economic pair $(token, a)$ a cyberlink carries is a [[box]]. creating a cyberlink moves that box from the source $from$ to a new output bound to the record; the box can later be transferred to a new owner or withdrawn to the author's wallet. the non-fungible assertion and the fungible box coexist in one record and are separable. see [[box]] for the full lifecycle and graph effects.

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
