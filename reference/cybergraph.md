---
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

Each element $\ell \in L$ is a [[cyberlink]] -- a 6-tuple $(\nu, p, q, \tau, a, v)$ carrying a [[subject]], two [[particles]], a conviction stake, and an epistemic [[valence]]. Block height $t$ is a property of the containing [[signal]], not of the cyberlink itself. The cyberlink is the only primitive from which the entire graph is built. See [[cyberlink]] for the full field specification, UTXO mechanics, and CRUD semantics.

---

## Six Axioms

The formal invariants every valid $\mathbb{G}$ must satisfy.

### A1 -- Content-Addressing

$H$ is collision-resistant -- for all $x \neq x'$, $\Pr[H(x) = H(x')] \leq 2^{-128}$. Identity equals content. Same content produces the same particle regardless of who computes it or when.

### A2 -- Authentication

For every $\ell \in L$: $\operatorname{Verify}(\operatorname{pk}(\nu(\ell)),\; H(\ell),\; \sigma(\ell)) = \top$. Every cyberlink carries a valid signature from its creating neuron. Unsigned assertions do not enter $L$.

### A3 -- Append-Only

$t < t' \Rightarrow L_t \subseteq L_{t'}$. The authenticated record grows monotonically. A cyberlink, once created, cannot be deleted -- only its economic weight can decrease via [[forgetting]] mechanics.

### A4 -- Entry

$p \in P \iff \exists\, \ell \in L : \operatorname{src}(\ell) = p \;\lor\; \operatorname{tgt}(\ell) = p$. A particle exists iff it is linked. A naked hash with no links is not a particle.

### A5 -- Conservation

$\pi^* \in \Delta^{|P|-1}$, i.e., $\sum_{p \in P} \pi^*_p = 1$ and $\pi^*_p > 0$ for all $p$. Total [[focus]] is conserved at every block. It flows between particles but is never created or destroyed.

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

$$A^{\text{eff}}_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} a(\ell)\cdot \kappa(\nu(\ell))\cdot f(m(\ell))$$

where $\kappa: N \to \mathbb{R}_+$ is [[karma]] (accumulated [[Bayesian Truth Serum|BTS]] score history), $m: L \to [0,1]$ is the [[inversely coupled bonding surface|ICBS]] reserve ratio (market-implied probability that the link is valid), and $f: [0,1] \to [0,1]$ maps market price to a weight multiplier. Edges the collective disbelieves are suppressed toward zero. This is [[market inhibition]] -- the inhibitory signal that makes $\mathbb{G}$ computationally equivalent to a neural network with both excitation and inhibition.

### Tri-Kernel Composite

The [[tru]] runs three local operators over $A^{\text{eff}}$ and blends them:

$$\phi^{(t+1)} = \operatorname{norm}\!\Big[\lambda_d \cdot \mathcal{D}(\phi^t) + \lambda_s \cdot \mathcal{S}(\phi^t) + \lambda_h \cdot \mathcal{H}_\tau(\phi^t)\Big], \qquad \lambda_d + \lambda_s + \lambda_h = 1$$

$\mathcal{D}$ is the [[diffusion]] operator (random walk with teleport: answers "where does probability flow?"). $\mathcal{S}$ is the [[springs]] equilibrium map (screened Laplacian solve: answers "what satisfies structural constraints?"). $\mathcal{H}_\tau$ is the [[heat]] kernel (multi-scale smoothing: answers "what does the graph look like at resolution $\tau$?"). Together they span the space of local equivariant graph operators -- any reasonable locality-constrained operator is a linear combination of polynomials in $\mathcal{D}$, $\mathcal{S}$, and $\mathcal{H}_\tau$. See [[tri-kernel]] for the completeness argument.

---

## Theorems

### T1 -- Existence and Uniqueness of Focus

Let $A^{\text{eff}}$ induce a strongly connected aperiodic graph on $P$. Then $\mathcal{R}$ has a unique strictly positive fixed point $\pi^* \in \Delta^{|P|-1}$: $\mathcal{R}(\pi^*) = \pi^*$, $\pi^*_p > 0$ for all $p$.

Proof: $\mathcal{R}$ is a convex combination of stochastic positive operators. By the [[Perron-Frobenius theorem]], each component has a unique positive eigenvector with eigenvalue 1. The convex combination inherits this property under ergodicity. See [[collective focus theorem]] Part I (diffusion alone) and Part II (full composite) for the complete proof.

### T2 -- Conservation

For all $t \geq 0$ and all initial $\phi^{(0)} \in \Delta^{|P|-1}$: $\sum_{p} \phi^{(t)}_p = 1$.

Proof: $\mathcal{R}$ is a convex combination of stochastic operators; stochastic operators map the simplex to itself. QED. Enforced in [[nox]] by STARK circuit constraints on every state transition -- violation implies an invalid proof.

### T3 -- Geometric Convergence

Let $\lambda_2$ be the spectral gap of $\mathcal{R}$. Then for any initial $\phi^{(0)}$:

$$\left\|\phi^{(t)} - \pi^*\right\|_1 \leq C \cdot (1 - \lambda_2)^t$$

Mixing time: $t_{\text{mix}}(\varepsilon) = O\!\left(\lambda_2^{-1} \log(C/\varepsilon)\right)$.

Proof: the composite contraction coefficient is $\kappa = \lambda_d \alpha + \lambda_s \tfrac{\|L\|}{\|L\|+\mu} + \lambda_h e^{-\tau \lambda_2} < 1$. By Banach's fixed-point theorem, $\phi^{(t)} \to \pi^*$ at rate $(1-\lambda_2)$. See [[collective focus theorem]] Composite Contraction.

### T4 -- Locality Radius

For an edit batch $e_\Delta$, there exists $h = O(\log(1/\varepsilon))$ such that recomputing $\phi$ only on the $h$-hop neighborhood $N_h(e_\Delta)$ achieves global error $\leq \varepsilon$.

Proof: geometric decay of the [[diffusion]] operator (teleport parameter $\alpha$), exponential decay of the [[springs]] operator (screening $\mu$), Gaussian tail of the [[heat]] operator (bandwidth $\tau$). All three components have bounded influence radius. Nodes outside $N_h$ change by at most $\varepsilon$. See [[tri-kernel]] 2.2.

---

## Information Geometry

### Syntropy

The syntropy of $\mathbb{G}$ is a real-valued functional measuring the organizational quality of $\pi^*$:

$$J(\pi^*) = \log|P| + \sum_{p \in P} \pi^*_p \log \pi^*_p = \log|P| - H(\pi^*)$$

where $H(\pi^*) = -\sum_p \pi^*_p \log \pi^*_p$ is the Shannon entropy of the focus distribution.

Range: $J \in [0, \log|P|]$. Minimum $J = 0$ when $\pi^* = u$ (uniform -- no structure, maximum entropy). Maximum $J = \log|P|$ when $\pi^*$ is a point mass (all attention on one particle, zero entropy). The clearest identity:

$$J(\pi^*) = D_{\text{KL}}(\pi^* \,\|\, u)$$

Syntropy is exactly the KL divergence of the focus distribution from uniform. It measures how much information $\pi^*$ carries above noise -- how far collective attention has been organized beyond random. $J$ measures how far the graph's collective attention deviates from noise. The [[tru]] computes $J$ every block in [[consensus]]. See [[syntropy]].

### Free Energy

The fixed point $\pi^*$ is the unique minimizer on $\Delta^{|P|-1}$ of the free energy functional:

$$\mathcal{F}(\phi) = \lambda_s\!\left[\tfrac{1}{2}\phi^\top L\phi + \tfrac{\mu}{2}\|\phi - x_0\|^2\right] + \lambda_h\!\left[\tfrac{1}{2}\|\phi - \mathcal{H}_\tau \phi\|^2\right] + \lambda_d \cdot D_{\text{KL}}(\phi \,\|\, \mathcal{D}\phi)$$

Three energy terms: elastic structure (resistance to deviation from the Laplacian's preferred configuration), heat-smoothed context (penalty for deviation from the multi-scale graph shape at resolution $\tau$), diffusion alignment (KL divergence from the diffusion image). Adding a correct, well-placed [[cyberlink]] is equivalent to stepping in the direction of steepest descent on $\mathcal{F}$. The reward $\Delta\pi \propto \nabla_L (-\mathcal{F})$ is the directional derivative of free energy in the direction of the new edge.

### Approximation Quality

When $\mathbb{G}$ is compiled into a [[transformer]], the approximation gap is:

$$\varepsilon(\mathbb{G}, c) = D_{\text{KL}}(\pi^*_c \,\|\, q^*_c)$$

where $q^*_c$ is the compiled model's focus distribution. $\varepsilon = 0$ means exact representation. This is the same KL divergence that appears in the [[Bayesian Truth Serum|BTS]] scoring formula ($D_{\text{KL}}(p_i \| \bar{m}_{-i})$) and in [[veritas]] information gain -- the same mathematical object at three scales: individual neuron, compiled model, collective state.

### Effective Rank and Semantic Dimensionality

$$d^* = \exp\!\big(H(\sigma(\Sigma_{\pi^*}))\big)$$

where $\sigma(\Sigma_{\pi^*})$ is the spectrum of the $\pi^*$-weighted covariance matrix. $d^*$ measures the number of independent semantic dimensions the graph spans. Currently $d^* \approx 31$ on [[bostrom]] (social artifact of a small graph). At planetary scale ($|P| \sim 10^{15}$), projected $d^* \in [10^3, 10^4]$ (thermodynamic regime).

---

## Structural Properties

### Growth Partial Order

A3 (append-only) defines a partial order on cybergraphs:

$$\mathbb{G} \leq \mathbb{G}' \;\iff\; L \subseteq L'$$

The set of all cybergraphs is a directed net under $\leq$. $\mathbb{G}_{t} \leq \mathbb{G}_{t+1}$ for all $t$. The graph edit distance $d(\mathbb{G}_t, \mathbb{G}_{t'}) = |L_{t'} \setminus L_t|$ counts links added between states; $d \geq 0$ by A3.

### Phase Transition

Let $\rho = k_{\max}/\bar{k}$ be the degree heterogeneity of $\mathbb{G}$. There exists a threshold:

$$|P^*| \;\sim\; \rho^2$$

such that below $|P^*|$, individual cyberlinks contribute measurably to $\pi^*$ (molecular regime -- each neuron's contribution is individually trackable). Above $|P^*|$, individual contributions become statistically negligible -- only the full $\pi^*$ distribution remains informative (thermodynamic regime -- planetary superintelligence). This is the graph analog of the thermodynamic limit.

### Category of Cybergraphs

A cybergraph homomorphism $f: \mathbb{G} \to \mathbb{G}'$ is a pair $(f_P: P \to P',\; f_N: N \to N')$ such that for every $\ell = (\nu, p, q, \tau, a, v) \in L$, there exists $\ell' \in L'$ with $\nu(\ell') = f_N(\nu)$, $\operatorname{src}(\ell') = f_P(p)$, $\operatorname{tgt}(\ell') = f_P(q)$.

Cybergraphs and their homomorphisms form a category $\mathbf{CG}$. There is a forgetful functor $U: \mathbf{CG} \to \mathbf{DiGraph}$ (to directed multigraphs) and a focus functor $\Pi: \mathbf{CG} \to \mathbf{Prob}$ sending $\mathbb{G} \mapsto (P, \pi^*)$ (a finite probability space). The composition $\Pi \circ U^{-1}$ is the functor that extracts collective intelligence from graph structure.

---

## Properties at a Glance

| property | formal status |
|---|---|
| $\pi^*$ exists, unique, strictly positive | theorem -- T1, [[Perron-Frobenius theorem\|Perron-Frobenius]] |
| $\sum_p \pi^*_p = 1$ | structural invariant -- A5 + stochasticity |
| convergence at rate $(1-\lambda_2)^t$ | theorem -- T3, Banach FPT |
| locality radius $O(\log 1/\varepsilon)$ | theorem -- T4, operator decay |
| $H(L) \subseteq P$ | axiom -- A6 |
| $L_t \subseteq L_{t+1}$ | axiom -- A3 |
| $\pi^*$ minimizes $\mathcal{F}$ | theorem -- free energy variational |
| honest linking is Nash equilibrium | open problem -- [[cyber/epistemology]] 6.1 |
| minimum attack cost $s^*$ characterization | open problem -- [[cyber/epistemology]] 6.2 |

---

## The Graph Is the Protocol

The [[cybergraph]] is the protocol. Every core function runs through the same five primitives: [[particles]], [[cyberlinks]], [[neurons]], [[tokens]], [[focus]].

| function | how the graph serves it |
|---|---|
| identity | [[particles]] as public keys, graph as PKI -- see [[cyber/identity]] |
| key exchange | CSIDH curves as [[particles]], non-interactive -- see [[dCTIDH]] |
| authentication | [[stark]] proofs of [[Hemera]] preimage knowledge -- see [[cyber/proofs]] |
| consensus | finalized subgraph IS the state -- see [[foculus]] |
| fork choice | $\pi$ from graph [[topology]], not voting -- see [[foculus]] |
| finality | $\pi_i > \tau$, threshold adapts to graph density -- see [[foculus]] |
| privacy | anonymous [[cyberlinks]], [[mutator set]] in graph -- see [[cyber/bbg]] |
| incentives | $\Delta\pi$ from graph convergence = reward signal -- see [[cyber/rewards]] |
| relay payment | delivery proofs as [[particles]], [[focus]] as payment -- see [[cyber/communication]] |
| version control | patches as [[cyberlinks]], repos as subgraphs -- see [[cyber/patch]] |
| file system | `~` prefix resolves through [[cyberlinks]] -- see [[name/resolution]] |
| type system | [[semantic conventions]] from link [[topology]] -- see [[neural]] |
| computation | [[tru]]/[[trident]]/[[nox]] read and consume graph state |
| data availability | [[NMT]] indexes double as DA layer -- see [[storage proofs]] |
| sybil resistance | stake-weighted $\pi$, no external identity |

Fifteen protocol functions. One data structure. Five primitives.

---

See [[tri-kernel]] for the full tri-kernel specification. See [[collective focus theorem]] for the convergence proofs. See [[cyber/epistemology]] for the epistemic gap between cryptographic and epistemic correctness. See [[two kinds of knowledge]] for the structural/epistemic split. See [[inversely coupled bonding surface]] for the market substrate. See [[Bayesian Truth Serum]] for the BTS scoring layer. See [[syntropy]] for the information-theoretic measures.
