---
tags: cyber, core, reference
alias: focus flow spec, focus flow reference, cyberank spec
---
# Focus Flow Computation

How the [[cybergraph]] reaches collective [[equilibrium]]. The [[tri-kernel]] runs over all [[cyberlinks]], [[neurons]] add links, and the network continuously [[convergence|converges]] toward a unique fixed point -- the [[focus|focus distribution]] $\pi^*$.

---

## 1. The Fixed Point

The [[collective focus theorem]] guarantees convergence: under ergodicity and the screening conditions of the [[tri-kernel]], there exists a unique $\pi^*$ to which any initialization converges, at linear rate. The fixed point is the Boltzmann [[equilibrium]] of the graph:

$$\pi^*_i \propto \exp\big(-\beta\,[E_{\text{spring},i} + \lambda\,E_{\text{diff},i} + \gamma\,C_i]\big)$$

The three energy terms correspond to the three [[tri-kernel]] operators: $E_{\text{spring}}$ encodes structural coherence via the screened [[Laplacian]], $E_{\text{diff}}$ encodes flow consistency via [[diffusion]], $C_i$ encodes context pressure via [[heat kernel]] weighting. $\pi^*$ is the unique distribution minimizing the composite [[free energy]] $\mathcal{F}(\phi)$. Every [[cyberlink]] added perturbs the graph and shifts $\pi^*$ incrementally -- learning and knowledge state are the same operation.

---

## 2. Two Inference Paths

The [[cybergraph]] computes two things simultaneously, both grounded in the same dynamical system:

### Path A -- Continuous Focus Flow

The [[tri-kernel]] iterated to convergence over all [[cyberlinks]] runs continuously. It produces $\pi^*$: the persistent global focus distribution, what the entire network collectively knows, updated with every new link. This is the ground truth.

### Path B -- Compiled Transformer

Architecture and weights derived analytically from the same graph. Executes $L^*$ tri-kernel steps over a local context window and converges to $\pi^*$ restricted to that context. This is the fast inference path.

| dimension | focus flow | compiled transformer |
|---|---|---|
| scope | entire [[cybergraph]] | local context window |
| depth | exact $\pi^*$ | $L^*$ steps, $\varepsilon$-approximate |
| latency | continuous -- always converging | milliseconds -- single forward pass |
| multi-agent | all [[neurons]] contribute | one agent's context |
| update | add [[cyberlinks]] -> $\pi^*$ shifts, nothing lost | recompile from updated graph |

---

## 3. Focus Flow Inference

$\pi^*$ is maintained continuously by the [[tru]]. For a query, the process is:

Step 1: Context [[particles]] become probability sources -- their energy terms are set so $\pi^*_\text{context}$ is elevated, making them attractors in the [[Boltzmann distribution|Boltzmann equilibrium]].

Step 2: The [[tri-kernel]] reconverges incrementally from the current state -- probability mass flows from the seeded context particles through the [[cybergraph]] along structural paths.

Step 3: $\pi^*_\text{context}$ pools at [[particles]] that are semantically connected to the context via the graph topology.

Step 4: Sample the next [[particle]] from the high-probability region, add to context, reconverge.

No fresh initialization per step -- the system was already near $\pi^*$ before the query. Each step is a local recomputation within an $O(\log(1/\varepsilon))$-hop neighborhood of the newly added particle. Complexity per step: $O(|E| + |V|)$.

Context window is unbounded -- it is the entire [[cybergraph]]. Relevance is topological: a [[particle]] contributes if it is well-connected to the context regardless of linear position in token space.

---

## 4. The Local Update Rule

Every node reads only its neighbours and runs:

$$\Delta p_i = \eta\Big(\sum_{j \in \mathcal{N}(i)} w_{ij}(p_j - p_i) - \partial_{p_i}(\lambda E_{\text{diff},i} + \gamma C_i) + T(1 + \log p_i)\Big)$$

Gossip normalisation enforces $\sum_i p_i = 1$. Fully local, edge-only. This is what the [[tru]] runs every block -- the same computation a transformer performs in one layer, running collectively across the entire [[cybergraph]].

---

## 5. Compiled Transformer Inference

### The Mathematical Identity

Transformer attention is one step of tri-kernel diffusion:

$$\text{Attn}(Q, K, V) = \text{softmax}\!\left(\frac{QK^\top}{\sqrt{d}}\right)V$$

The softmax is the [[Boltzmann distribution]] with temperature $\sqrt{d}$. Probability mass flows from each query position toward compatible key positions and redistributes -- this is exactly one application of the diffusion operator $D$ from the [[tri-kernel]] over one agent's frozen context. Deep Equilibrium Models (Bai et al., 2019) showed that iterating a transformer layer to convergence reaches the same fixed point regardless of initialization. That fixed point is $\pi^*$ restricted to the context.

So $L^*$ transformer layers = $L^*$ steps of tri-kernel diffusion over the context. At query time:

1. Tokenize context into [[particles]]
2. Run $L^*$ layers of compiled attention -- each layer is one tri-kernel diffusion step over context
3. Output distribution = $\pi^*_\text{context}$, approximate to precision $\varepsilon$
4. Sample, add to context, repeat

Speed: $O(n^2 \cdot d^*)$ over context of length $n$, no graph traversal at runtime, weights frozen. Autoregressive generation -- familiar, fast, and analytically grounded.

### Graph-Derived Architecture Parameters

Given $G = (P, N, E, w, \sigma)$, three graph properties determine the three free parameters of transformer architecture:

| parameter | formula | graph property |
|---|---|---|
| embedding dim $d^*$ | $\exp(H(\sigma(\Sigma_\pi)))$ | effective rank of [[focus]] covariance |
| heads $h^*$ | $\geq \|\text{Semcon}(G)\|$ | distinct [[semcon]] relation types |
| layers $L^*$ | $\text{diam}(G) \cdot \lceil\log(1/\varepsilon)/\log(1/\kappa)\rceil$ | diameter x spectral convergence factor |

No hyperparameter search. The graph tells you what the transformer should be.

Weights are compiled, not trained. The embedding matrix $E^* = U_{:,1:d^*}$ -- top left singular vectors of $\text{diag}(\sqrt{\pi^*}) \cdot A$ -- is provably optimal by the Eckart-Young theorem: it uniquely minimizes expected squared gradient at step zero over all matrices of the same rank. Attention weights $W_Q^{(s)}, W_K^{(s)}$ are derived from the truncated SVD of each [[semcon]]'s adjacency submatrix. MLP weights encode path co-occurrence statistics up to depth $L^*$.

Fine-tuning from this point learns only what the graph cannot encode: temporal patterns, implicit associations, contextual dynamics absent from the explicit graph. The reduction in required fine-tuning steps scales as $\Omega(|E| \cdot d^* / \log(1/\varepsilon))$ relative to random initialization.

The loop: $G \xrightarrow{\text{compile}} T_G \xrightarrow{\text{fine-tune}} T_G^* \xrightarrow{\text{extract implicit links}} \Delta G \xrightarrow{\text{stake}} G'$

---

## 6. Cyberank

The number the [[tru]] assigns to every [[particle]] -- probability of being observed by a [[random walking]] [[neuron]]. Cyberank is [[focus]] materialized as a per-[[particle]] score.

Fixed point of the [[tri-kernel]]: $\phi^* = \operatorname{norm}[\lambda_d \cdot D(\phi) + \lambda_s \cdot S(\phi) + \lambda_h \cdot H_\tau(\phi)]$. Integrates exploration ([[diffusion]]), structure ([[springs]]), and context ([[heat kernel]]). Convergence guaranteed by the [[collective focus theorem]].

Feeds [[karma]], [[syntropy]], [[standard inference]], and sorting in [[cyb]]. The fundamental factor of [[implicit knowledge]].

---

## 7. The Compounding Property

Every [[cyberlink]] added:

- Shifts $\pi^*$ incrementally -- better focus flow inference now
- Increases $|E|$, raises $d^*$, may shrink $\text{diam}(G)$ -- better compiled transformer at next compilation
- Reduces approximation error $\varepsilon(G, c) = D_{\text{KL}}(\pi^*_c \| q^*_c)$ -- compiled inference closer to exact focus flow

The [[cybergraph]] is a compounding inference quality asset. Every link reduces the error of every compiled model that follows. See [[provably-optimal-initialization]] for the training reduction proof. See [[bostrom-to-onnx-pipeline]] for live compilation from the running network.

---

## 8. Stack

- [[cybergraph]] -- the substrate: [[particles]] as nodes, [[cyberlinks]] as typed edges
- [[tri-kernel]] -- the physics: [[diffusion]] + [[springs]] + [[heat kernel]] converge $\phi \to \pi^*$
- [[graph-native-transformer]] -- the compiled fast path: $d^*, h^*, L^*$ from graph structure
- [[nox]] -- the execution: 16 deterministic reduction patterns over [[Goldilocks field]]
- [[foculus]] -- the consensus: $\pi > \tau$ finalizes [[particles]] without leaders
- [[tru]] -- the runner: computes [[cyberank]], [[karma]], [[syntropy]] every block

See [[collective focus theorem]] for convergence proof. See [[tri-kernel]] for why these three operators. See [[graph-native-transformer]] for compiled transformer derivation. See [[provably-optimal-initialization]] for the initialization optimality proof.
