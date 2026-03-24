---
tags: cyber, docs
alias: inference explained, cybergraph as model, generative model
---
# The Cybergraph as Generative Model

The [[cybergraph]] is the model. The [[tri-kernel]] focus distribution $\pi^*$ is the "weights" -- a stationary distribution over [[particles]] that encodes what the network collectively considers relevant. A query is a context vector that biases the focus field. The response is the equilibrium of the biased field. Generation is iterative: each produced particle shifts the context, the field re-equilibrates, and the next particle emerges from the new $\pi^*$.

---

## How pi-star IS the Model

A traditional generative model has parameters (weights) trained on data. The cybergraph has no separate parameters. The graph structure IS the model. The edges encode relationships between pieces of content. The tri-kernel computes a probability distribution over all nodes. That distribution is the model's "prediction" of what matters, given the current structure.

Every new [[cyberlink]] is a training sample. Every [[focus]] expenditure is a gradient signal. The model updates continuously without batch training, gradient computation, or deployment. Quality follows from the [[collective focus theorem]]: attention distributes exponentially by quality. Neurons with good judgment earn [[karma]]; neurons with poor judgment waste focus. The selection pressure is economic.

---

## The Transformer Comparison

Transformer attention and tri-kernel focus are the same mathematical operation applied to different substrates:

$$\text{Attn}(Q, K, V) = \text{softmax}\!\left(\frac{QK^\top}{\sqrt{d}}\right)V$$

$$\pi^* = \text{fixed-point}(\alpha \cdot D + \beta \cdot S + \gamma \cdot H)$$

Both produce a probability distribution over elements. Both are differentiable. The softmax is the [[Boltzmann distribution]] with temperature $\sqrt{d}$. The tri-kernel produces the same distribution but computed over a knowledge graph instead of a token sequence, with economics instead of gradient descent as the training signal.

| dimension | GPT-class transformer | cybergraph generative model |
|---|---|---|
| vocabulary | fixed (50K-200K tokens) | open (all particles, content-addressed) |
| context | bounded window | unbounded graph ($O(\log n)$ access) |
| attention | softmax over tokens | tri-kernel over particles |
| training | offline gradient descent | continuous economic signal (focus) |
| update | retrain or fine-tune | new cyberlinks enter $\pi^*$ immediately |
| inference proof | impossible | zero-cost (proof-carrying) |
| explainability | opaque | decomposable (D + S + H contributions) |
| multi-agent | single model | native ($N$ neurons, foculus consensus) |
| privacy | model is public or private | individual links private, aggregate public |
| scale | $O(\text{params})$ per query | $O(\text{relevant\_edges})$ per query |
| temperature | softmax $\tau$ | Boltzmann $T$ (same math, economic grounding) |
| light client | download full model | 240-byte checkpoint, verify everything |

Deep Equilibrium Models (Bai et al., 2019) showed that iterating a transformer layer to convergence reaches the same fixed point regardless of initialization. That fixed point is $\pi^*$ restricted to the context. So $L$ transformer layers = $L$ steps of tri-kernel diffusion toward the fixed point.

---

## Random Walks Explained

The foundation of [[cyberank]] is the random walk. Imagine a neuron randomly navigating the cybergraph by clicking on links from one particle to another. The transition probabilities are determined by edge weights and token values. The random walk answers: "If I keep following links indefinitely, what fraction of my time will I spend at each particle?"

The stationary distribution of this walk is $\pi^*$. Particles at the crossroads of many well-weighted paths accumulate more probability. Peripheral particles, connected by few weak links, receive less.

The random walk is the simplest model of exploration in a graph. Despite its simplicity, it produces order from randomness: local random steps create global patterns. The same mathematics describes Brownian motion (physics), genetic drift (biology), stock price movements (finance), and species dispersal (ecology). PageRank was the first large-scale application of random walks to knowledge ranking. The cybergraph extends this with economic weighting (tokens as conviction), structural constraints (springs), and multi-scale smoothing (heat).

---

## Query Biasing

A query $q$ is a set of particles with activation weights. The tri-kernel runs with a context potential $C(q)$ that biases the focus field:

$$\pi^*_q = \text{fixed-point}(\alpha D + \beta S + \gamma H + \delta C(q))$$

The biased equilibrium $\pi^*_q$ concentrates on particles relevant to the query. Context particles become probability sources -- their energy terms elevate them into attractors. Probability mass flows from the seeded context through the graph along structural paths (not token positions). It pools at particles that are semantically connected to the context via graph topology.

The context window is unbounded -- it is the entire cybergraph. Relevance is topological: a particle contributes if it is well-connected to the context, regardless of linear position in any sequence.

---

## Autoregressive Generation

Generating a sequence of particles (analogous to token generation):

1. Input context biases the focus field, producing $\pi^*_q$
2. Sample particle $p_1$ from $\pi^*_q$
3. Add $p_1$ to context, re-bias, obtain $\pi^*_{q \cup \{p_1\}}$
4. Sample $p_2$ from the updated field
5. Continue: sequence $(p_1, p_2, \ldots, p_n)$ generated by iterated equilibration

Each step is a [[nox]] computation that produces a [[zheng]] proof via proof-carrying computation. The generated sequence is self-proving: anyone can verify it was produced by correctly running the tri-kernel on the stated context.

The tri-kernel has a natural temperature parameter $T$ from the free energy formulation:

$$\pi^*_i \propto \exp\left(-\frac{E_i}{T}\right)$$

$T \to 0$: deterministic -- always pick the highest-$\pi^*$ particle (argmax). $T = 1$: standard sampling -- faithful to the collective focus distribution. $T > 1$: creative -- explore low-probability particles. The same exponential governs Boltzmann distributions, softmax, and collective focus.

---

## Zero-Cost Proof-Carrying

Every generation step is verifiable:

```
input:   context particles + query
compute: tri-kernel iteration (nox<Goldilocks>)
output:  response particles + zheng proof

verification: 10-50 us (one decider call)
proves:       "the tri-kernel was run correctly on this context"
```

A transformer cannot prove its inference was correct without re-running it. The cybergraph model produces a proof of correct inference as a byproduct of the computation -- zero additional cost.

---

## Why Every Cyberlink Improves Future Inference

The cybergraph is a compounding inference quality asset. Every [[cyberlink]] added:

- Shifts $\pi^*$ incrementally -- better focus flow inference immediately
- Increases $|E|$, raises the effective semantic dimensionality $d^*$, may shrink graph diameter -- better compiled transformer at next compilation
- Reduces the approximation error $\varepsilon(G, c) = D_{\text{KL}}(\pi^*_c \| q^*_c)$ -- compiled inference closer to exact focus flow

The graph that the model runs on IS the authenticated state. Particles are the vocabulary (all content-addressed nodes). Outgoing edges are the embeddings. Incoming edges are the reverse embeddings. Neurons are the trainers (agents with focus budgets). The entire state is a polynomial commitment. A query is a polynomial opening. The response is verifiable against a 32-byte root commitment.

---

## What the Cybergraph Does that Transformers Cannot

Open vocabulary without retraining. A transformer's vocabulary is fixed at training time. The cybergraph's vocabulary is the set of all particles -- content-addressed and open. A new particle enters the graph when a neuron creates a cyberlink to it. No retraining, no fine-tuning. The new particle immediately participates in $\pi^*$ computation.

Provable inference. Every generation step carries a STARK proof. Verification takes microseconds. No need to re-run the computation.

Continuous learning from economic signal. Every cyberlink is a training sample. Every focus expenditure is a gradient signal. The model updates continuously.

Multi-agent generation. Multiple neurons can simultaneously contribute to the same query response. Their cyberlinks compete and compose via the tri-kernel. The result is the equilibrium of a physical system where stake determines influence.

Scale-invariant architecture. The tri-kernel operates on whatever subgraph is relevant. A query about cats touches cat-related particles. The computation scales with the local graph, not the global graph: $O(\text{relevant\_edges})$ per query, regardless of total graph size.

---

See [[focus flow computation]] for the full specification. See [[tri-kernel]] for the operators. See [[collective focus theorem]] for why the distribution is optimal. See [[cybergraph model architecture]] for the detailed comparison table.
