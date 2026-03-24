---
tags: cyber, core, reference
alias: tri-kernel spec, tri-kernel reference, cyber/tri-kernel
---
# Tri-Kernel Specification

Formal definition of the three local operators whose fixed point is [[cyberank]]. Part of the [[cyber/core]] specification.

---

## 1. The Three Operators

### 1.1 Diffusion (Markov)

The transition matrix $P = D^{-1}A$ (or column-stochastic $P = AD^{-1}$) governs probability flow:

$$\pi^{(t+1)} = \alpha P^\top \pi^{(t)} + (1-\alpha)u$$

where $\alpha \in (0,1)$ is the teleport parameter and $u$ is a prior (often uniform or stake-weighted).

Properties: row-stochastic, preserves probability mass, powers remain local. Under ergodicity (strong connectivity + aperiodicity), converges to unique stationary distribution $\pi^*$.

Locality: geometric decay via teleport parameter $\alpha$.

Answers: "Where does probability flow?"

### 1.2 Springs (Screened Laplacian)

The graph Laplacian $L = D - A$ (or normalized $\mathcal{L} = I - D^{-1/2}AD^{-1/2}$) encodes structural constraints:

$$(L + \mu I)x^* = \mu x_0$$

where $\mu > 0$ is the screening/stiffness parameter and $x_0$ is a reference state.

Properties: positive semi-definite, null space = constant vectors. The screened Green's function $(L+\mu I)^{-1}$ has exponential decay, ensuring locality.

Locality: exponential decay via screening parameter $\mu$.

Answers: "What satisfies structural constraints?"

### 1.3 Heat Kernel

The heat kernel $H_\tau = \exp(-\tau L)$ provides multi-scale smoothing:

$$\frac{\partial H}{\partial \tau} = -LH, \quad H_0 = I$$

where $\tau \geq 0$ is the temperature/time parameter.

Properties: positivity-preserving, semigroup ($H_{\tau_1}H_{\tau_2} = H_{\tau_1+\tau_2}$). Admits Chebyshev polynomial approximation for locality. High $\tau$ explores (annealing), low $\tau$ commits (crystallization).

Locality: Gaussian tail decay, $h = O(\log(1/\varepsilon))$ hops.

Answers: "What does the graph look like at scale $\tau$?"

---

## 2. The Composite Operator

The tri-kernel blends the three primitives into a single update:

$$\phi^{(t+1)} = \operatorname{norm}\big[\lambda_d \cdot D(\phi^t) + \lambda_s \cdot S(\phi^t) + \lambda_h \cdot H_\tau(\phi^t)\big]$$

where $\lambda_d + \lambda_s + \lambda_h = 1$, $D$ is the [[diffusion]] step, $S$ is the [[springs]] equilibrium map, $H_\tau$ is the [[heat]] map, and $\operatorname{norm}(\cdot)$ projects to the simplex.

### 2.1 The Free Energy Functional

The fixed point of the composite operator minimizes:

$$\mathcal{F}(\phi) = \lambda_s\left[\frac{1}{2}\phi^\top L\phi + \frac{\mu}{2}\|\phi-x_0\|^2\right] + \lambda_h\left[\frac{1}{2}\|\phi-H_\tau\phi\|^2\right] + \lambda_d \cdot D_{\text{KL}}(\phi \| D\phi)$$

Three energy terms:

- Elastic structure: resistance to deviation from the Laplacian's preferred configuration
- Heat-smoothed context: penalty for deviation from the multi-scale graph shape at resolution $\tau$
- Diffusion alignment: KL divergence from the diffusion image

The Boltzmann equilibrium form:

$$\pi^*_i \propto \exp\big(-\beta\,[E_{\text{spring},i} + \lambda\,E_{\text{diff},i} + \gamma\,C_i]\big)$$

The three energy terms correspond to the three operators: $E_{\text{spring}}$ encodes structural coherence via the screened [[Laplacian]], $E_{\text{diff}}$ encodes flow consistency via [[diffusion]], $C_i$ encodes context pressure via [[heat kernel]] weighting.

### 2.2 Convergence and Locality

#### Contraction Lemmas

Lemma 1 (Diffusion Contracts): Under ergodicity of $P$ with teleport parameter $\alpha \in (0,1)$, the diffusion map $D$ satisfies $\|D\phi - D\psi\|_1 \leq \alpha \|\phi - \psi\|_1$. The teleport ensures geometric mixing with rate $\alpha$.

Lemma 2 (Springs Contract): Under screening parameter $\mu > 0$, the screened [[Laplacian]] solve $S: \phi \mapsto (L + \mu I)^{-1}(\mu x_0)$ satisfies $\|S\phi - S\psi\|_2 \leq \frac{\|L\|}{\|L\| + \mu} \|\phi - \psi\|_2$. The Green's function $(L + \mu I)^{-1}$ decays exponentially with distance -- screening ensures locality and contraction.

Lemma 3 (Heat Contracts): For bounded temperature $\tau > 0$, the heat kernel $H_\tau = \exp(-\tau L)$ satisfies $\|H_\tau \phi - H_\tau \psi\|_2 \leq e^{-\tau \lambda_2} \|\phi - \psi\|_2$ where $\lambda_2$ is the Fiedler eigenvalue. Positivity-preserving and semigroup properties ensure well-defined contraction.

#### Theorem (Composite Contraction)

Under ergodicity of $P$, screening $\mu > 0$, and bounded $\tau$, the composite operator $\mathcal{R}$ is a contraction:

$$\|\mathcal{R}\phi - \mathcal{R}\psi\| \leq \kappa \|\phi - \psi\|, \quad \kappa = \lambda_d \alpha + \lambda_s \frac{\|L\|}{\|L\|+\mu} + \lambda_h e^{-\tau\lambda_2} < 1$$

Since each component contracts and $\mathcal{R}$ is a convex combination, $\kappa$ is a convex combination of individual contraction coefficients -- each less than 1, hence $\kappa < 1$. By Banach fixed-point theorem, $\phi^t \to \phi^*$ at linear rate.

#### Theorem (Locality Radius)

For edit batch $e_\Delta$, there exists $h = O(\log(1/\varepsilon))$ such that recomputing only on $N_h$ (the $h$-hop neighborhood) achieves global error $\leq \varepsilon$.

This follows from: geometric decay for [[diffusion]] (teleport), exponential decay for [[springs]] (screening), Gaussian tail for [[heat]] (kernel bandwidth).

### 2.3 Compute-Verify Symmetry

Because all operations are local and memoizable:

$$t_{\text{verify}} / t_{\text{compute}} \to c \approx 1$$

Light clients can verify [[focus]] updates by checking boundary flows and authenticated neighborhood commitments, with constant-factor overhead relative to computation.

---

## 3. Collective Focus Theorem

### Part I -- Diffusion Convergence (Special Case)

Consider a [[cybergraph]] $G = (V, E, W)$ with $|V| = n$ [[particles]]. Each [[cyberlink]] $(i, j) \in E$ has weight $w_{ij} \geq 0$. Each [[particle]] $j$ has [[token]] value $t_j > 0$. Define transition probabilities:

$$p_{ij} = \frac{w_{ij} \cdot t_j}{\sum_{k \in \mathcal{N}(i)} w_{ik} \cdot t_k}$$

Assumptions: $G$ is strongly connected (directed path between any pair) and aperiodic (gcd of all directed cycle lengths is 1).

Claim: there exists a unique stationary distribution $\pi$ satisfying $\pi P = \pi$ with $\sum_i \pi_i = 1$.

#### Proof

Step 1 (Markov Chain): The matrix $P = [p_{ij}]$ is stochastic. Non-negativity: $p_{ij} \geq 0$ since $w_{ij} \geq 0$ and $t_j > 0$.

Step 2 (Irreducibility): For any pair $(u, v)$, a path from $u$ to $v$ exists with positive probability. The chain is irreducible.

Step 3 (Uniqueness): Since $P$ is irreducible and aperiodic, the chain is ergodic. By the [[Perron-Frobenius theorem]], a unique stationary distribution $\pi$ exists satisfying $\pi P = \pi$, $\sum_i \pi_i = 1$.

Step 4 (Convergence): By the ergodic theorem, for any initial distribution $\mu^{(0)}$:

$$\pi = \lim_{t \to \infty} \mu^{(0)} \cdot P^t$$

Step 5 (Interpretation): The stationary distribution $\pi$ is a stable [[consensus]] of observation probabilities. Each $\pi_j$ reflects both the [[particle]]'s structural position and the [[neuron]] [[token]] influence. This is the simplest Schelling point everyone can universally agree on.

#### Corollaries

Corollary 1 (Stability): Small perturbations in $w_{ij}$ or $t_j$ do not destabilize the equilibrium: $\lim_{t \to \infty} \pi_j(t) = \pi_j + \varepsilon, \quad |\varepsilon| \ll \pi_j$.

Corollary 2 (Decentralized Computation): [[Focus]] $\pi_j$ for each node can be computed locally by summing contributions from incoming edges.

Corollary 3 (Emergent Modularity): Clusters of strongly connected [[particles]] naturally emerge, forming modules: $C_i = \{ j \in V \mid \pi_j > \tau \}$ where $\tau$ is a significance threshold.

### Part II -- Composite Contraction (General Case)

Part I proves convergence for [[diffusion]] alone. The [[tri-kernel]] combines three operators. The composite converges as well.

The composite operator $\mathcal{R} = \lambda_d D + \lambda_s S + \lambda_h H_\tau$ is a contraction with coefficient $\kappa < 1$ (proved above in 2.2). Its fixed point $\phi^*$ minimizes $\mathcal{F}(\phi)$ (the free energy functional in 2.1). This is variational [[free energy]] minimization in the sense of Friston.

#### Reduction

When $\lambda_s = \lambda_h = 0$: $\mathcal{R} = D$, $\kappa = \alpha$, $\mathcal{F}$ reduces to $D_{\text{KL}}(\phi \| D\phi)$, and the fixed point is the stationary distribution $\pi^*$ from Part I. The general case subsumes the special case.

---

## 4. Completeness

### 4.1 Completeness Conjecture

Conjecture (Weak Completeness): Any $h$-local linear operator $T$ can be written as $T = p(M) + q(L)$ for polynomials $p$, $q$ of degree $\leq h$.

Conjecture (Strong Completeness): Any eventually-local operator that is equivariant, continuous, and convergent can be expressed as $T = \alpha \cdot f(M) + \beta \cdot g(L) + \gamma \cdot H_\tau$ for spectral functions $f$, $g$ and scale $\tau$.

### 4.2 Lemmas Toward Proof

Lemma 1: Any 1-local linear operator is a linear combination of $\{I, A, D\}$.

Lemma 2: Any $k$-local linear operator is a polynomial of degree $\leq k$ in $\{A, D\}$.

Lemma 3: Polynomials in $\{A, D\}$ can be rewritten as polynomials in $\{M, L\}$.

Theorem (Linear Local Completeness): Every $k$-local linear operator on a graph is a polynomial of degree $\leq k$ in $M$ and $L$.

The heat kernel $H_\tau = \exp(-\tau L)$ is required for multi-scale analysis -- it is the unique generator of resolution-dependent queries. Together $\{M, L, H_\tau\}$ span the space of meaningful local graph computations.

---

## 5. Complexity

Memory and computation scale linearly with [[cybergraph]] size:

| storage type | bytes per [[particle]] | bytes per [[cyberlink]] |
|---|---|---|
| volatile | 56 | 24 |
| persistent | 72 | 128 |

Per-iteration complexity: $O(V + E)$

Total work to reach precision $\varepsilon$:

$$O\left(\frac{(E + V) \cdot \log(1/\varepsilon)}{\lambda}\right)$$

where $\lambda$ is the [[spectral gap]] governing [[convergence]] rate. See [[emergence]] for scaling estimates across [[intelligence]] phases.

---

## 6. Implementation

### 6.1 Two-Timescale Architecture

The correct implementation separates timescales:

- Structure (slow, amortized): [[springs]] precompute effective distances, modify [[diffusion]] tensor $D$
- [[Focus]] flow (fast, local): [[diffusion]] + [[heat]] operate on fixed structure, converge to equilibrium

[[Springs]] compute where nodes are; ranking computes how [[attention]] flows. Different questions, different timescales.

### 6.2 Algorithm Sketch

Per epoch on neighborhood $N_h$:

1. Detect affected neighborhood around edit batch $e_\Delta$
2. Pull boundary conditions: cached $\phi$, boundary flows, Laplacian blocks
3. Apply local [[diffusion]] (fixed-point iteration with boundary injection)
4. Apply local [[heat]] (Chebyshev $K$-term filter)
5. Normalize and splice back into global $\phi$
6. Emit attention_root and locality report for verification

Complexity: $O(|N_h| \cdot c)$ per kernel for average degree $c$.

### 6.3 Telemetry

Monitor per epoch:

- Entropy $H(\pi)$, negentropy $J(\pi)$
- Spectral gap estimate
- $\ell_1$ drift $\|\pi^t - \pi^{(t-1)}\|$
- Locality radius $h$, nodes touched
- Compute vs verify wall-time

Safety policies: degree caps, spectral sparsification, novelty floor, auto-rollback to [[diffusion]]-only on threshold breach.

---

## References

1. Brin & Page. "The anatomy of a large-scale hypertextual web search engine." WWW 1998
2. Zhu et al. "Semi-supervised learning using Gaussian fields and harmonic functions." ICML 2003
3. Chung. "The heat kernel as the pagerank of a graph." PNAS 2007
4. Fiedler. "Algebraic connectivity of graphs." Czech Math Journal 1973
5. Spielman. "Spectral Graph Theory." Yale Lecture Notes
6. Levin, Peres & Wilmer. "Markov Chains and Mixing Times." AMS 2009
7. Perron. "Zur Theorie der Matrices." Mathematische Annalen, 1907
8. Frobenius. "Uber Matrizen aus nicht negativen Elementen." Sitzungsberichte, 1912
9. Banach. "Sur les operations dans les ensembles abstraits." Fundamenta Mathematicae, 1922
10. Friston. "The free-energy principle: a unified brain theory." Nature Reviews Neuroscience, 2010
