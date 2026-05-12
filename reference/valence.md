# valence

the fifth field of a [[cyberlink]]: $v \in \{-1, 0, +1\}$. the epistemic prediction — the [[neuron]]'s forecast of where the [[inversely coupled bonding surface|ICBS]] market on this edge will converge

valence is what makes every [[cyberlink]] a [[Bayesian Truth Serum]] input. the structural layer says "this link exists." the economic layer says "I stake this much." the epistemic layer says "I predict the collective will agree (or disagree)"

## definition

$$v(\ell) \in \{-1,\, 0,\, +1\}$$

a ternary field encoding the [[neuron]]'s meta-prediction about the link's epistemic fate:

| value | meaning | market prediction |
|-------|---------|-------------------|
| $+1$ | affirm | the ICBS market will converge toward TRUE |
| $0$ | agnostic | no epistemic prediction — structural assertion only |
| $-1$ | negate | the ICBS market will converge toward FALSE |

## the three layers

valence is the third dimension of the [[cyberlink]]'s layered structure:

| layer | field(s) | type | what it encodes |
|-------|----------|------|-----------------|
| structural | $(p, q)$ | binary | connection exists or does not |
| economic | $(\tau, a)$ | continuous | conviction depth |
| epistemic | $v$ | ternary | prediction of collective judgment |

the three layers are orthogonal. every combination of $v$ and $a$ is meaningful — see [[cyberlink]] CRUD table

## role in Bayesian Truth Serum

the valence field IS the BTS meta-prediction. no separate submission step is needed. mapping:

| BTS concept | cyberlink field |
|-------------|-----------------|
| first-order belief $p_i$ | link creation + [[amount]] $(\tau, a)$ |
| meta-prediction $m_i$ | valence $v$ |
| agent identity | signing [[neuron]] $\nu$ |

the BTS score for [[neuron]] $i$:

$$s_i = D_{KL}(p_i \| \bar{m}_{-i}) - D_{KL}(p_i \| \bar{p}_{-i}) - D_{KL}(\bar{p}_{-i} \| m_i)$$

where $p_i$ is the neuron's revealed belief (link + stake), $m_i$ is the valence meta-prediction, $\bar{p}_{-i}$ is the geometric mean of others' beliefs, and $\bar{m}_{-i}$ is the geometric mean of others' predictions

Prelec (2004) proved that truthful reporting is a Bayes-Nash [[equilibrium]]: no [[neuron]] can improve their expected score by misreporting either belief or meta-belief

## role in effective adjacency

valence feeds into the [[inversely coupled bonding surface|ICBS]] market price $m(\ell) \in [0,1]$, which enters effective adjacency:

$$A^{\text{eff}}_{pq} = \sum_\ell a(\ell) \cdot \kappa(\nu(\ell)) \cdot f(m(\ell))$$

edges where the collective valence is negative ($v = -1$ dominates) see their market price $m \to 0$, suppressing effective weight toward zero. this is [[market inhibition]] — the epistemic immune system of the [[cybergraph]]

## valence and karma

[[karma]] is the accumulated BTS score history. a [[neuron]] whose valence predictions consistently match where markets converge accumulates high karma. high karma amplifies future link weight through $\kappa(\nu)$ in effective adjacency

the compounding loop: honest valence → high BTS score → high karma → more effective weight per link → more [[focus]] shift per contribution → higher reward

## the epistemic signal

valence creates a two-dimensional epistemic signal per [[cyberlink]]:

- ICBS market price $m(\ell)$ encodes magnitude of collective belief (continuous, $[0,1]$)
- aggregate valence $\bar{v}$ encodes collective confidence in that belief (discrete, $\{-1, 0, +1\}$)

one-dimensional price becomes two-dimensional: how much the collective believes, and how confident the collective is in that belief

see [[cyberlink]] for the full 5-tuple. see [[Bayesian Truth Serum]] for the full scoring mechanism. see [[inversely coupled bonding surface]] for the market mechanics. see [[karma]] for accumulation
