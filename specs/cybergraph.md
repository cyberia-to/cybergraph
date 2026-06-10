---
title: cybergraph spec
tags: cyber, core, reference
alias: cybergraph spec, cybergraph reference
---
# Cybergraph

Formal specification of the cybergraph data structure, its axioms, derived operators, and theorems.

---

## Definition

A cybergraph $\mathbb{G}$ is a triple:

$$\mathbb{G} = (P,\; N,\; L)$$

| symbol | set | element type |
|---|---|---|
| $P \subseteq \operatorname{Im}(H)$ | [[particles]] | content-addressed nodes |
| $N$ | [[neurons]] | authenticated agents |
| $L$ | [[cyberlinks]] | labeled directed edges (multiset) |
| $\mathcal{T}$ | [[tokens]] | conviction denominations (derived from $L$) |

$H: \text{Val} \to \mathbb{F}_p^8$ is the global [[Hemera]] hash primitive, fixed at genesis. Every particle is a hash of some value -- $P$ is a subset of $H$'s image. $\mathcal{T}$ and the karma function $\kappa$ are derived from $L$.

Each element $\ell \in L$ is a [[cyberlink]] -- a 5-tuple $(p, q, \tau, a, v)$ carrying two [[particles]], a conviction stake, and an epistemic [[valence]]. The signing [[neuron]] $\nu$ and block height $t$ are attributes of the containing [[signal]], not of the cyberlink itself. The cyberlink is the only primitive from which the entire graph is built. See [[cyberlink]] for the full field specification, box mechanics, and CRUD semantics.

---

## Six Axioms

The formal invariants every valid $\mathbb{G}$ must satisfy.

### A1 -- Content-Addressing

$H$ is collision-resistant -- for all $x \neq x'$, $\Pr[H(x) = H(x')] \leq 2^{-128}$. Identity equals content. Same content produces the same particle regardless of who computes it or when.

### A2 -- Authentication

Every [[signal]] $s$ carries a valid signature from its creating [[neuron]]: $\operatorname{Verify}(\operatorname{pk}(\nu(s)),\; H(s),\; \sigma(s)) = \top$. Authentication is at the signal level — cyberlinks within an invalid signal do not enter $L$.

### A3 -- Append-Only

$t < t' \Rightarrow L_t \subseteq L_{t'}$. The authenticated record grows monotonically. A cyberlink, once created, cannot be deleted -- only its economic weight can decrease via [[forgetting]] mechanics.

### A4 -- Entry

$p \in P \iff \exists\, \ell \in L : \operatorname{src}(\ell) = p \;\lor\; \operatorname{tgt}(\ell) = p$. A particle exists iff it is linked. A naked hash with no links is not a particle.

### A5 -- Conservation

$\phi^* \in \Delta^{|P|-1}$, i.e., $\sum_{p \in P} \phi^*_p = 1$ and $\phi^*_p > 0$ for all $p$. Total [[focus]] is conserved at every block. It flows between particles but is never created or destroyed.

### A6 -- Homoiconicity

$H(\operatorname{src}(\ell),\, \operatorname{tgt}(\ell)) \in P$. Every directed edge -- every [[axon]] -- induces a [[particle]] via content-addressing. The hash of the (from, to) pair, without metadata, produces one axon-particle per unique relationship. All [[cyberlinks]] along the same edge contribute weight to the same axon-particle. Axon-particles receive [[focus]], carry [[cyberank]], and can themselves be targets of [[cyberlinks]] -- the graph ranks its own structure.

---

## Derived Structures

### Raw Adjacency

From $L$, define the weighted adjacency operator $A: \mathbb{R}^P \to \mathbb{R}^P$:

$$A_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

where $r: \mathcal{T} \to \mathbb{R}_+$ converts token denomination to a common scale. $A_{pq}$ is the total economic weight of all cyberlinks from $p$ to $q$. The stochastic normalization $\hat{A}_{pq} = A_{pq} / \sum_{q'} A_{pq'}$ gives the transition matrix of the raw [[random walk]] on $\mathbb{G}$.

### Effective Adjacency

With the epistemic layer active (ICBS markets running and karma accumulated), the effective adjacency modifies each link's weight by market belief and neuron trust:

$$A^{\text{eff}}_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} a(\ell)\cdot \kappa(\nu(s_\ell))\cdot f(m(\ell))$$

where $s_\ell$ is the [[signal]] containing $\ell$, $\kappa: N \to \mathbb{R}_+$ is [[karma]] (accumulated [[Bayesian Truth Serum|BTS]] score history), $m: L \to [0,1]$ is the [[inversely coupled bonding surface|ICBS]] reserve ratio (market-implied probability that the link is valid), and $f: [0,1] \to [0,1]$ maps market price to a weight multiplier. Edges the collective disbelieves are suppressed toward zero. This is [[market inhibition]] -- the inhibitory signal that makes $\mathbb{G}$ computationally equivalent to a neural network with both excitation and inhibition.

### Tri-Kernel Composite

The weighted graph $A^{\text{eff}}$ is the input to the [[tru]]. Cybergraph produces the weighted graph; tru runs the three local operators ([[diffusion]], [[springs]], [[heat]]) over it to compute the [[focus]] fixed point $\phi^*$. The boundary is sharp: cybergraph defines the graph and its adjacency, tru defines what the graph computes.

---

## Dynamics live in tru

Everything about the $\phi^*$ fixed point is computed and specified by the [[tru]], not here: existence and uniqueness, conservation, geometric convergence, the locality radius, [[syntropy]], the free-energy variational form, and effective semantic rank. Cybergraph defines the graph; tru defines the dynamics on it.

- [[tri-kernel]] — the three operators, the completeness argument, the locality radius
- [[collective focus theorem]] — existence, uniqueness, conservation, geometric convergence
- [[focus-flow]] — the inference process that computes $\phi^*$
- [[syntropy]] — the information-geometric measures (syntropy, free energy, effective rank)
- [[rewards]] — how a proven $\Delta\phi^*$ ([[impulse]]) becomes a self-minted reward

---

## Structural Properties

### Growth Partial Order

A3 (append-only) defines a partial order on cybergraphs:

$$\mathbb{G} \leq \mathbb{G}' \;\iff\; L \subseteq L'$$

The set of all cybergraphs is a directed net under $\leq$. $\mathbb{G}_{t} \leq \mathbb{G}_{t+1}$ for all $t$. The graph edit distance $d(\mathbb{G}_t, \mathbb{G}_{t'}) = |L_{t'} \setminus L_t|$ counts links added between states; $d \geq 0$ by A3.

### Phase Transition

Let $\rho = k_{\max}/\bar{k}$ be the degree heterogeneity of $\mathbb{G}$. There exists a threshold:

$$|P^*| \;\sim\; \rho^2$$

such that below $|P^*|$, individual cyberlinks contribute measurably to $\phi^*$ (molecular regime -- each neuron's contribution is individually trackable). Above $|P^*|$, individual contributions become statistically negligible -- only the full $\phi^*$ distribution remains informative (thermodynamic regime -- planetary superintelligence). This is the graph analog of the thermodynamic limit.

## Properties at a Glance

| property | formal status |
|---|---|
| $H(L) \subseteq P$ | axiom -- A6 |
| $L_t \subseteq L_{t+1}$ | axiom -- A3 |
| $H$ collision-resistant | axiom -- A1 |
| $p \in P \iff p$ is linked | axiom -- A4 |
| $\phi^*$ exists, unique, conserved, converges | theorems -- in [[tru]] (see [[collective focus theorem]]) |
| honest linking is Nash equilibrium | open problem -- [[cyber/epistemology]] 6.1 |

---

## The Graph Is the Protocol

The [[cybergraph]] is the protocol. Every core function runs through the same five primitives: [[particles]], [[cyberlinks]], [[neurons]], [[tokens]], [[focus]].

| function | how the graph serves it |
|---|---|
| identity | [[particles]] as public keys, graph as PKI -- see [[cyber/identity]] |
| key exchange | CSIDH curves as [[particles]], non-interactive -- see [[dCTIDH]] |
| authentication | [[stark]] proofs of [[Hemera]] preimage knowledge -- see [[cyber/proofs]] |
| consensus | finalized subgraph IS the state -- see [[foculus]] |
| fork choice | $\phi^*$ from graph [[topology]], not voting -- see [[foculus]] |
| finality | $\phi^*_i > \tau$, threshold adapts to graph density -- see [[foculus]] |
| privacy | anonymous [[cyberlinks]], [[mutator set]] in graph -- see [[cyber/bbg]] |
| incentives | $\Delta\phi^*$ from graph convergence = reward signal -- see [[cyber/rewards]] |
| relay payment | delivery proofs as [[particles]], [[focus]] as payment -- see [[cyber/communication]] |
| version control | patches as [[cyberlinks]], repos as subgraphs -- see [[cyber/patch]] |
| file system | `~` prefix resolves through [[cyberlinks]] -- see [[name/resolution]] |
| type system | [[dialects]] from link [[topology]] -- see [[neural]] |
| computation | [[tru]]/[[trident]]/[[nox]] read and consume graph state |
| data availability | [[NMT]] indexes double as DA layer -- see [[storage proofs]] |
| sybil resistance | stake-weighted $\phi^*$, no external identity |

Fifteen protocol functions. One data structure. Five primitives.

---

See [[tri-kernel]] for the full tri-kernel specification. See [[collective focus theorem]] for the convergence proofs. See [[cyber/epistemology]] for the epistemic gap between cryptographic and epistemic correctness. See [[two kinds of knowledge]] for the structural/epistemic split. See [[inversely coupled bonding surface]] for the market substrate. See [[Bayesian Truth Serum]] for the BTS scoring layer. See [[syntropy]] for the information-theoretic measures.
