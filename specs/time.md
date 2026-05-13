# time

the $t$ field of a [[signal]]: $t \in \mathbb{Z}_{\geq 0}$. block height at which the signal is committed to the [[cybergraph]]. every [[cyberlink]] in the signal shares the same $t$ — they enter the graph together, timestamped by the containing signal.

time is the temporal anchor of every [[signal]]. it answers [[when]] — the one question that cannot be answered by content or structure alone. the block height is the [[cybergraph]]'s logical clock.

## definition

$$t(s) \in \mathbb{Z}_{\geq 0}$$

monotonically increasing block counter. every [[cyberlink]] in a given signal shares the same $t$. ordering within a block is determined by the $\phi^*$ convergence trajectory (see [[foculus]])

## role in the append-only graph

axiom A3 guarantees $t < t' \Rightarrow L_t \subseteq L_{t'}$. time makes this ordering explicit: every signal carries its birth moment, and every cyberlink it produces enters $L$ at that height. the graph grows monotonically through time.

the same author linking the same structural triple $(\nu, p, q)$ in two signals at $t_1$ and $t_2 > t_1$ produces two separate entries in $L$. the signals' block heights distinguish them. this enables:

- reinforcement — higher [[amount]] on a new record at a later block
- [[valence]] update — new epistemic prediction at a new block
- multi-denomination staking — same structural link in different [[tokens]] at different times

## role in discovery premium

time determines who discovered what first. the [[attention]] yield curve rewards early accurate linking:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\phi^*(q, t)\, dt$$

a [[cyberlink]] created at $t_1$ (early, when $\phi^*(q)$ is low) captures more cumulative $\Delta\phi^*$ than an identical link at $t_2 \gg t_1$ (late, when $\phi^*(q)$ has already risen). the time field is what makes this attribution possible — it is the provenance of priority

## role in temporal decay

staleness depends on time. a link created at $t_\ell$ can be weighted by temporal decay:

$$w(t, \ell) = a(\ell) \cdot e^{-\lambda(t - t_\ell)}$$

where $\lambda$ is a per-namespace decay constant. mathematics namespaces use $\lambda = 0$ (no decay). current events namespaces use positive $\lambda$. the time field makes decay computable

## role in consensus

[[foculus]] orders state transitions by $\phi^*$ convergence trajectory across time. a [[particle]] crosses the finality threshold $\tau$ at some block height — that block height is the finality time. the nullifier set $N$ grows monotonically through time. time provides the causal ordering that makes double-spend detection deterministic

## role in signals

$t$ is a first-class field of every [[signal]] — the signal's batch of [[cyberlinks]] all inherit this timestamp. the [[stark]] proof $\sigma$ references the [[BBG]] root at $t$ — proving that the [[focus]] shift $\Delta\phi^*$ was computed against the graph state at that specific moment

## machine time

time in the [[cybergraph]] is discrete, deterministic, and consensus-verified. it is the [[vimputer]]'s logical clock — the ordered sequence of [[steps]] at which the [[tru]] recomputes [[focus]], [[cyberank]], [[karma]], and [[syntropy]]

| property | physical time | cybergraph time |
|----------|--------------|-----------------|
| substrate | continuous | discrete block heights |
| authority | clocks, GPS | [[consensus]] |
| ordering | relativistic, observer-dependent | total order via block sequence |
| granularity | Planck time | block interval (~5s in [[bostrom]]) |
| verifiability | requires trusted clock | verifiable from chain headers |

see [[signal]] for the full structure. see [[cyberlink]] for the 6-tuple content that time stamps. see [[foculus]] for how time interacts with finality.
