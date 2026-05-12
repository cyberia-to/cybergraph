# cyberlink

the atomic unit of [[knowledge]]. two [[particles]] bound by a staked assertion — every cyberlink is simultaneously a [[learning]] act and an economic commitment. who made it and when belong to the [[signal]] that carries it.

## the five fields

$$\ell \;=\; (p,\; q,\; \tau,\; a,\; v) \;\in\; P \times P \times \mathcal{T} \times \mathbb{R}_{+} \times \{-1,\,0,\,+1\}$$

| field | name | type | layer | semantics | question |
|-------|------|------|-------|-----------|----------|
| $p$ | from | $P$ | structural | source [[particle]] | [[what]] is the source? |
| $q$ | to | $P$ | structural | target [[particle]] | [[what]] is the target? |
| $\tau$ | [[token]] | $\mathcal{T}$ | economic | token denomination | in what denomination? |
| $a$ | [[amount]] | $\mathbb{R}_+$ | economic | stake amount | how much conviction? |
| $v$ | [[valence]] | $\{-1,0,+1\}$ | epistemic | [[Bayesian Truth Serum\|BTS]] meta-prediction | what is the epistemic prediction? |

three layers in one atomic record. structural $(p, q)$ is binary — the connection either exists or it does not. epistemic $v$ is ternary — the neuron's prediction of how the [[inversely coupled bonding surface|ICBS]] market on this edge will converge. economic $(\tau, a)$ is continuous over $\mathbb{R}_+$. see [[two three paradox]] for why this layering is not arbitrary

conviction = ($\tau$, $a$): the pair that turns an assertion into a bet. denomination selects the [[token]], [[amount]] declares the stake. a link with zero conviction is structurally identical to a link with maximum conviction — the structural layer is binary. the conviction layer prices it

cyberlinks are bundled into [[signal|signals]] for broadcast. the [[signal]] adds the provenance layer: the signing [[neuron]] $\nu$, block height $t$, an [[cyber/impulse]] ($\pi_\Delta$ — the proven [[focus]] shift), and a recursive [[stark]] proof covering the entire batch.

the [[cybergraph]] is append-only. [[time]] (block height from the containing signal) distinguishes every record: the same author linking from→to in two signals at $t_1$ and $t_2 > t_1$ produces two separate entries in $L$. this enables reinforcement (higher [[amount]] on a new record), [[valence]] updates (new $v$ at a new signal), and multi-denomination staking (same structural link in different [[tokens]])

see [[particle]] for content addressing. see [[token]] for denomination. see [[amount]] for conviction mechanics. see [[valence]] for epistemic prediction. see [[time]] for temporal ordering.

## UTXO semantics

conviction is a [[UTXO]]. creating a cyberlink is a transaction: the author moves $a$ tokens of denomination $\tau$ from a wallet UTXO to a new output bound to the cyberlink record. funds always move from one object to another

the conviction output can itself be spent:

- transfer: spend the conviction UTXO to a new owner. the structural record stays in $L$; beneficial ownership moves. this is how the card's transferability operates at the protocol level
- withdraw: spend the conviction UTXO back to the author's wallet. the economic position closes. the structural record remains

the non-fungibility of the card (5-tuple content + signal provenance) and the fungibility of the [[token]] (transferable UTXO) coexist: the assertion is non-fungible, the economic position is a standard UTXO output

## CRUD operations

the append-only graph expresses all four operations through cyberlinks:

| operation | cyberlink action | what changes |
|-----------|-----------------|--------------|
| create | first record for structural pair $(p, q)$ | relation enters $L$ |
| read | query $\pi^*$ at any block — no link required | nothing |
| update | new record with new $(\tau, a, v)$ for the same pair | any mutable dimension |
| delete | withdraw conviction UTXO + new record with $v = -1$ | economic position closed, epistemic signal negated |

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

delete in the graph is never erasure. the assertion $(p, q)$ committed at $t_{\text{first}}$ stays in $L$ permanently. economic close and epistemic retraction are separable operations

## the card

every cyberlink is also a card — an epistemic asset with four properties:

immutable — axiom A3 (append-only) guarantees the record is permanent once published. the assertion cannot be altered or retracted. the author's conviction, [[valence]], and timestamp are locked into the graph's history forever

unique — each card is non-fungible. two signals with the same 5-tuple content submitted at different block heights produce distinct records in $L$ — the signal's $\nu$ and $t$ are what identify them. the 5-tuple is the assertion; the signal is the provenance.

transferable — ownership of a cyberlink and its yield rights can be transferred between [[neurons]]. the structural record stays in $L$ forever; beneficial ownership moves

yield-bearing — a cyberlink earns in proportion to how much the target [[particle]] gains [[focus]]:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\pi^*(q, t)\, dt$$

where $w(t)$ is the conviction weight at [[time]] $t$ and $\Delta\pi^*(q, t)$ is the increment in the target particle's focus. early discovery is maximally rewarded; late consensus-following earns little

## edge labeling

a cyberlink has no built-in type field. labeling works through the graph itself: every directed edge induces an [[axon]]-[[particle]] via axiom A6 ($H(p, q) \in P$). to label an edge, create a cyberlink from a type-[[particle]] to the [[axon]]-[[particle]]:

```
A --cyberlink--> B                  the assertion
"is-a" --cyberlink--> axon(A, B)    the label
```

any [[particle]] can serve as a label: `is-a`, `contradicts`, `extends`, `cites`, `created-by`. the label itself has [[cyberank]], [[karma]], market price — the graph weights the importance of relation types the same way it weights everything else
