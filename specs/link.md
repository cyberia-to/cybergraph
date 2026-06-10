---
tags: cyber, cybergraph, core
crystal-type: entity
crystal-domain: cyber
alias: link, structural edge, the edge
---
# link

the structural part of a [[cyberlink]]: a directed edge from one [[card]] to another.

$$\text{link} = (from,\; to) \;\in\; \text{Card} \times \text{Card}$$

| field | name | type | what it is |
|---|---|---|---|
| $from$ | source | [[card]] | what the edge starts at — authorizes the move (plumb auth on this card's leaf) |
| $to$ | destination | [[card]] | what the edge points to |

a [[cyberlink]] is a link plus a [[box]] (conviction) plus a [[valence]] (prediction). the link alone is the bare connection: it exists or it does not — binary, no magnitude. the box prices it; the valence predicts it.

both endpoints are [[card|cards]] — the addressable token type. a [[particle]] (content-addressed knowledge node) is a card; a [[neuron]] is a card; a [[network]] is a card. so a link can connect any two of them, and the same machinery covers a knowledge edge, a transfer, or a stake.

## what groups links

- [[axon]] — the bundle of all links between the same $(from, to)$ pair, across all neurons and time. links are individual; the axon is their aggregate, and is itself a [[particle]] (axiom A6).

see [[cyberlink]] for the full record · [[card]] for the endpoint type · [[box]] for the conviction it carries · [[axon]] for the bundle.
