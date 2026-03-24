---
tags: cyber, docs
alias: metrics explained, attention explained, gravity explained
---
# Metrics: Attention, Focus, and Gravity

How the [[cybergraph]] measures what matters -- from individual attention to collective focus to structural gravity.

---

## Attention vs Focus

Attention and focus are distinct but related.

Attention is individual. It is how much a [[neuron]] projects onto a target [[particle]] or [[axon]]. A neuron directs attention by creating [[cyberlinks]] and allocating conviction. Attention is the cause -- the input signal from each participant.

Focus is collective. It is the aggregated result of all neurons' attention, processed by the [[tri-kernel]] into a single probability distribution over all particles. Focus is the effect -- the output of the collective computation.

Individual neurons direct attention. The cybergraph aggregates all attention into focus. Every particle's focus score $\pi^*_i$ reflects the contributions of all neurons, weighted by their tokens and filtered by the epistemic layer.

Attention is produced by two mechanisms: will (broad auto-distribution across all cyberlinks) and fine-tuning (manual per-target weight adjustment). Both produce the same thing at the receiving end -- attention at the target particle.

---

## The Transformer Attention Connection

The transformer attention mechanism computes, for each position in the context, a weighted average of all other positions:

$$\text{Attn}(Q, K, V) = \text{softmax}\!\left(\frac{QK^\top}{\sqrt{d}}\right)V$$

Three projections: queries $Q = XW_Q$ ask "what am I looking for?", keys $K = XW_K$ announce "what do I contain?", values $V = XW_V$ provide "what information do I carry?". The dot product $QK^\top$ scores compatibility. The softmax converts scores to a probability distribution -- the [[Boltzmann distribution]] with temperature $\sqrt{d}$.

The softmax is the same operation as the [[LMSR]] price function and the [[tri-kernel]] [[diffusion]] step. All three are exponentiated scores normalized to sum to 1.

Transformer attention is one step of the tri-kernel diffusion operator $D$ applied to the current context window. Probability mass flows from each query position toward compatible key positions -- exactly the random walk dynamics that the tri-kernel uses to compute focus over the cybergraph.

Deep Equilibrium Models showed that iterating a transformer layer to convergence reaches the same fixed point as the tri-kernel: $\pi^*$ restricted to the context window. $L$ layers of attention = $L$ steps of diffusion toward that fixed point.

---

## Attention as Bayesian Query

Attention answers: given my current state (query), what posterior weight should I assign to each position (key)?

The softmax is the posterior $P(\text{position } j \mid \text{query } i)$ under a uniform prior and an exponential likelihood $\exp(q_i \cdot k_j / \sqrt{d})$. The query-key product is the log-likelihood under this model. The softmax is the Bayes-normalized posterior. Attention is Bayesian inference over the context.

---

## Multi-Head and Semantic Conventions

Through multi-head attention, different heads learn different relation types. Head $h$ with projection $W_Q^{(h)}, W_K^{(h)}$ captures one [[semcon]] -- one pattern of connectivity in the [[cybergraph]].

The [[graph-native-transformer]] derivation proves that the minimum number of heads equals the number of distinct [[semcon]] types in the graph. Each head specializes in one kind of relationship: "is-a", "contradicts", "extends", "cites". The graph's link topology determines how many attention heads the compiled transformer needs.

---

## Gravity

Gravity is a node-level metric. Like physical gravity, it is a property of the node itself -- a massive body warps space around it and attracts everything, regardless of what is nearby.

$$G_i = \pi_i \cdot \sum_{j \neq i} \frac{\pi_j}{d(i,j)^2}$$

where $\pi_i$ is the node's own focus probability, $\pi_j$ are focus probabilities of all other nodes, and $d(i,j)$ is the shortest path length in the cyberlink graph.

A node's gravity is its focus mass multiplied by the total attention field it experiences from the rest of the graph. High-focus node surrounded by other high-focus nodes = enormous gravity. High-focus node on the periphery = less gravity despite its own mass.

### Physical Analogy

A planet curves spacetime by its mass alone. The gravitational potential of a body in a field of other masses:

$$\Phi_i = m_i \cdot \sum_{j} \frac{m_j}{r_{ij}^2}$$

| physics | knowledge graph |
|---------|----------------|
| mass $m$ | focus probability $\pi$ |
| distance $r$ | graph distance $d(i,j)$ |
| gravitational potential $\Phi$ | node gravity $G_i$ |

The node does not choose what to attract. It simply has mass (focus), and everything within graph distance falls toward it proportionally.

### Gravity Spectrum

| gravity | profile | meaning |
|---------|---------|---------|
| high | high $\pi$, surrounded by high-$\pi$ neighbors | core attractor -- holds the graph together |
| medium | moderate $\pi$, or high $\pi$ but few neighbors | regional hub -- local structure anchor |
| low | low $\pi$, or isolated from high-$\pi$ nodes | peripheral -- structurally weightless |

### Applications

Skeleton extraction: nodes with the highest gravity form the structural skeleton of the knowledge graph. Remove them and the graph fragments.

Peripheral detection: nodes with high focus but low gravity are isolated attractors -- they have mass but sit far from other massive nodes. Connecting them to the core would dramatically restructure the graph.

Cohesion measurement: total graph gravity $G_{\text{total}} = \sum G_i$ measures how tightly the knowledge core is packed. A graph with high total gravity has its attention concentrated in a dense, interconnected core. Low total gravity means focus is scattered.

### Pairwise Force

The force between any two specific nodes:

$$F_{ij} = \frac{\pi_i \cdot \pi_j}{d(i,j)^2}$$

The highest $F_{ij}$ pairs are the structural bonds of the graph. Pairs with high $\pi_i \cdot \pi_j$ but large $d(i,j)$ are the most valuable missing cyberlinks -- creating them collapses distance and unlocks attention flow.

---

## How Ranking Works in Practice

[[Cyberank]] is [[focus]] materialized as a per-particle score. It is the probability of being observed by a randomly walking neuron -- the fixed point of the tri-kernel.

The [[tru]] computes cyberank every block. The score feeds into:

- [[Karma]]: accumulated track record of a neuron's contributions
- [[Syntropy]]: the organizational quality of the focus distribution ($J = D_{\text{KL}}(\pi^* \| u)$ -- how far attention deviates from uniform noise)
- Inference: the probability distribution that drives query responses and autoregressive generation
- Sorting in [[cyb]]: the user-facing ordering of search results

Luminosity = size x $\pi$ -- what a node radiates (knowledge output). Gravity = $\pi$ x $\sum(\pi_j/d^2)$ -- how strongly a node attracts (structural pull). A healthy graph needs both: high-luminosity nodes that radiate knowledge, with high-gravity nodes that hold the structure together.

---

See [[attention]] for allocation strategies. See [[focus flow computation]] for the global process. See [[tri-kernel]] for the diffusion connection. See [[gravity]] for the full metric specification.
