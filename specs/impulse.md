---
alias: impulse, focus impulse, Δφ*, pi_delta, impulses
tags: cyber, core
crystal-type: process
crystal-domain: cyber
---
# impulse

the proven change in [[focus]] that a [[neuron]] delivers to the [[cybergraph]] via a [[cyber/signal]]. mathematically $\Delta\phi^*$ — a sparse vector of (particle_id, $\Delta\phi^*$) pairs representing how the [[focus]] distribution $\phi^*$ shifts when the signal's [[cyberlinks]] are applied

in physics, impulse is force applied over time that changes momentum ($J = \Delta p$). in neuroscience, the nerve impulse is the action potential that propagates through a network and changes downstream potentials. in cyber, the impulse is the neuron's proven push on collective [[focus]] — discrete, has magnitude, delivered at a specific moment, and propagates through the [[cybergraph]]

## computation

the [[neuron]] computes the impulse by running the [[tri-kernel]] locally on their $O(\log(1/\varepsilon))$-hop neighborhood, adding their [[cyberlinks]], and measuring how $\phi^*$ shifts. the [[locality theorem]] guarantees effects beyond that radius are below $\varepsilon$ — most entries are zero, so the sparse representation is compact

the result is whatever the math says. there is no target, no threshold, no minimum. a link to a well-connected [[particle]] in a sparse region produces a larger impulse than a redundant link in a dense cluster. the neuron discovers their contribution by computing it

## proof

the impulse is accompanied by a [[stark]] proof $\sigma$ that certifies correctness against the current [[BBG]] root. the proof covers the entire [[cyber/signal]] — all [[cyberlinks]] in the batch, all conviction box movements, and the resulting $\Delta\phi^*$ — in a single recursive verification. any node checks $\sigma$ in $O(\log n)$ without recomputing the [[tri-kernel]]

## reward and conservation live in tru

the impulse proof doubles as a reward claim — a valid $\sigma$ with $\|\Delta\phi^*\| > 0$ self-mints [[$CYB]] proportional to the shift, and per-epoch minting is bounded by the actual global $\Delta\phi^*$ (overlapping claims scaled proportionally). cybergraph carries the field; the reward functions, attribution, and conservation math are the [[tru]]'s — see [[rewards]]. how $\Delta\phi^*$ is computed is in [[focus-flow]].

see [[signal]] for the field's slot · [[focus]] · [[tru]] for computation and reward
