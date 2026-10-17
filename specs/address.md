---
tags: cyber, docs
alias: address, address record, follow link, antenna link, socket link, locus link, FOLLOW, ANTENNA, SOCKET, LOCUS
---
# address

a [[neuron]]'s address is not a field of a signal: it is four [[cyberlink|cyberlinks]] the neuron publishes on its own book. the wire obeys them — discovery, subscription and routing are graph content, replicated by the mechanism they route. proven on three nodes on one machine in 2026-09 ([[soft3]] status): a cell that knew one peer followed a stranger, received the stranger's chain through the friend, read the address out of it and dialed directly.

| link | from | to | means |
|---|---|---|---|
| FOLLOW | the neuron's name | the followed neuron's name | subscription: sync that book; and the routing edge the [[locus]] is computed from |
| ANTENNA | the neuron's name | endpoint particle | the [[radio]] endpoint id the neuron answers on |
| SOCKET | the neuron's name | `ip:port` particle | a plain socket for peers without radio discovery |
| LOCUS | the neuron's name | coordinate particle | the neuron's hyperbolic coordinate $(r, \theta)$ at epoch $E$, per [[locus]] |

all four are ordinary cyberlinks: signed, sealed, in a signal, on the neuron's own book, public by construction. a neuron that wants to be reachable publishes them; a neuron that does not, is not. nothing here touches the knowledge links, whose author stays private (P1).

## LOCUS

the coordinate particle is the hemera hash of the canonical bytes

```
"locus:" ‖ E (u64 le) ‖ r (u64 le, fixed point) ‖ θ (u64 le, fixed point)
```

with the scale [[tru]]'s [[arithmetic]] declares. the bytes are the particle's content, so any node that sees the link can read the coordinate without a lookup.

**a LOCUS is a claim.** tru computes the locus of every neuron from the follow graph at the epoch boundary; the neuron publishes what it computed so that partial nodes, which do not hold the whole follow graph, can route to it. a full node recomputes the claim and ignores a mismatch for routing; two LOCUS links from the same neuron for the same epoch are a conflict and resolve like any other conflict in [[foculus]]. a stale LOCUS (epoch older than the current one by more than $\delta_E$) is still used, with the age as a tie-breaker, so a quiet neuron does not fall off the disk.

**refresh.** a neuron republishes LOCUS when its computed coordinate moved by more than $\delta_d$ in hyperbolic distance since the published one, or when the epoch gap exceeds $\delta_E$. both are network parameters ([[network]]), not constants of this page.

## what the address is for

- **sync.** FOLLOW is the subscription: the wire syncs exactly the followed books.
- **dial.** ANTENNA and SOCKET are how to reach a neuron whose address you already hold.
- **route.** LOCUS is how to reach a neuron whose address you do not hold: forward toward it over FOLLOW links, nearest locus first ([[soft3]] [[routing]]). the rule lives in the node, not here; this page only defines the record it reads.
