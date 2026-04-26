---
tags: cyber, core, reference
alias: clifford extensions, geometric algebra extensions, wedge extensions
---
# Clifford Extensions to Cybergraph Primitives

Formal extension of selected [[cybergraph]] primitives from scalar to multivector form (scalar + bivector). Restores directional and structural information currently reduced to scalar weights. Strictly additive: the scalar part of every extended quantity reproduces the existing [[cybergraph]] specification byte-for-byte.

This spec defines:
- the multivector representation over the [[Goldilocks field]]
- the extended axon weight (§3)
- the extended effective adjacency (§4)
- the shifted geometric product as the native local operator (§5)
- backward-compatibility contract for consumers that only read the scalar part (§6)

Downstream consumers using these extensions: [[compiled transformers spec]] (CT-1.1 wedge-augmented attention and Clifford-block MLP), [[render]] (T∞ neural rendering via Clifford block).

See [[Ren]] for the Clifford language that emits these operations as [[nox]] jets. See [[tri-kernel]] for the scalar operators that remain unchanged. Wedge enhancement of [[springs]], syntropy, and [[cyberank]] is out of scope — deferred to a later pass.

---

## 1. Multivector Representation

Each extended quantity is a graded multivector over the scalar Goldilocks field $\mathbb{F}_p$, $p = 2^{64} - 2^{32} + 1$:

$$M = M_0 + M_2 \in \mathbb{F}_p \oplus \bigwedge\nolimits^{\!2}\!\mathbb{F}_p^D$$

where $M_0 \in \mathbb{F}_p$ is the scalar grade, $M_2 \in \bigwedge^2 \mathbb{F}_p^D$ is the bivector grade, and $D$ is the ambient dimension of the enclosing context (e.g., vocabulary size, embedding dimension).

Grades 1 (vectors) and higher than 2 are not used in this spec. A full Clifford algebra $G(p, q, r)$ with mixed grade signature is reserved for [[Ren]].

The bivector is stored as a sparse list of coefficients indexed by ordered basis pairs:

$$M_2 = \sum_{1 \leq i < j \leq D} c_{ij}\, e_i \wedge e_j, \quad c_{ij} \in \mathbb{F}_p$$

Canonical encoding: ordered pairs $(i, j)$ with $i < j$. Wrap-around from shift operators (see §5) is resolved by sign flip rather than reordering.

---

## 2. Legacy Scalar Form

For any quantity $X$ defined in [[cybergraph]] as a scalar, its extended form is:

$$X^{\star} = X^{\star}_0 + X^{\star}_2$$

with the conformance invariant

$$X^{\star}_0 \equiv X$$

i.e., the scalar part equals the legacy scalar value exactly. All theorems in [[cybergraph]] (T1–T4), axiom A5 (conservation), and existing determinism predicates continue to hold on the scalar grade.

---

## 3. Extended Axon Weight

### 3.1 Legacy form

From [[cybergraph]] §Raw Adjacency and [[axon]]:

$$w(p, q) = \sum_{\ell : \mathrm{src}(\ell) = p,\, \mathrm{tgt}(\ell) = q} r(\tau(\ell)) \cdot a(\ell)$$

### 3.2 Extended form

$$w^{\star}(p, q) = w_0(p, q) + w_2(p, q)$$

Scalar grade (unchanged):

$$w_0(p, q) = \sum_{\ell} r(\tau(\ell)) \cdot a(\ell)$$

Bivector grade (new):

$$w_2(p, q) = \sum_{\ell} r(\tau(\ell)) \cdot a(\ell) \cdot v(\ell) \cdot (e_p \wedge e_q)$$

where $v(\ell) \in \{-1, 0, +1\}$ is the [[valence]] of the cyberlink and $e_p, e_q$ are the basis vectors indexed by the particle indices of $p$ and $q$ in the particle set $P$.

### 3.3 Interpretation

The bivector coefficient $c_{pq} = \sum_\ell r(\tau) a v$ encodes **signed valence consensus** on the directed edge $(p, q)$. Positive $c_{pq}$ means the neurons that cyberlinked $p \to q$ predominantly asserted $v = +1$ (affirmative consensus). Negative $c_{pq}$ means predominantly $v = -1$ (contested). Zero means balanced or $v = 0$ throughout.

The bivector $e_p \wedge e_q$ is oriented: swapping the endpoints flips the sign. This captures directional asymmetry natively — $w_2(p, q) = -w_2(q, p)$ as bivectors, where the legacy scalar form conflated them.

### 3.4 Recovery of legacy form

Any consumer that wants the scalar weight computes $w^{\star}_0(p, q)$ and ignores $w^{\star}_2(p, q)$. This preserves CT-1.0 ([[compiled transformers spec]] §§2–9) behavior exactly.

---

## 4. Extended Effective Adjacency

### 4.1 Legacy form

From [[cybergraph]] §Effective Adjacency:

$$A^{\mathrm{eff}}_{pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f(m(\ell))$$

where $\kappa$ is [[karma]] and $m(\ell) \in [0, 1]$ is the [[inversely coupled bonding surface\|ICBS]] market belief.

### 4.2 Extended form

$$A^{\mathrm{eff}\star}_{pq} = A^{\mathrm{eff}}_{0,\,pq} + A^{\mathrm{eff}}_{2,\,pq}$$

Scalar grade (unchanged):

$$A^{\mathrm{eff}}_{0,\,pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f_0(m(\ell))$$

with $f_0(m) = f(m)$ unchanged from the legacy mapping.

Bivector grade (new):

$$A^{\mathrm{eff}}_{2,\,pq} = \sum_{\ell} a(\ell) \cdot \kappa(\nu(\ell)) \cdot f_2(m(\ell)) \cdot \mathrm{sign}(v(\ell)) \cdot (e_p \wedge e_q)$$

where $f_2: [0, 1] \to \mathbb{F}_p$ is a confidence mapping:

$$f_2(m) = |2m - 1|$$

(i.e., $f_2 = 0$ when the market is maximally uncertain at $m = 0.5$; $f_2 = 1$ when the market is fully confident either direction).

### 4.3 Interpretation

The bivector part captures **oriented confidence**. When the market converges to high or low $m$, $f_2$ is large; when neurons agree on $\mathrm{sign}(v)$, the bivector coefficients accumulate; when they disagree, they cancel. This exposes a grade-2 algebraic signal for disagreement that [[foculus]] fork-choice can threshold on without consulting market internals.

### 4.4 Downstream use

- [[compiled transformers spec]] CT-1.0 reads $A^{\mathrm{eff}}_0$, discards $A^{\mathrm{eff}}_2$.
- CT-1.1 reads both grades; bivector participates in wedge-augmented attention (§7).
- [[render]] T∞ reads both; bivector drives directional flow in the neural field.

---

## 5. Shifted Geometric Product

The primitive local operator for all downstream consumers is the **shifted geometric product** over channel-indexed features, following CliffordNet (Ji, 2026). For feature tensors $H, C \in \mathbb{F}_p^{N \times D}$ and a shift offset $s \in \{1, 2, \ldots, D-1\}$:

### 5.1 Shifted inner product

$$\mathrm{Inner}_s(H, C)_{i, c} = \sigma\!\left( H_{i, c} \cdot C_{i,\, (c+s) \bmod D} \right)$$

where $\sigma$ is the SiLU activation lifted to $\mathbb{F}_p$ via lookup (see [[Goldilocks field processor]] LUT primitive).

### 5.2 Shifted wedge product

$$\mathrm{Wedge}_s(H, C)_{i, c} = H_{i, c} \cdot C_{i,\, (c+s) \bmod D} - H_{i,\, (c+s) \bmod D} \cdot C_{i, c}$$

This is the strictly anti-symmetric bivector coefficient for the basis pair $(e_c, e_{(c+s) \bmod D})$. Wrap-around ($c + s \geq D$) yields a sign flip that is absorbed by the subsequent learnable projection.

### 5.3 Full shifted geometric product

$$[H C]_s = \mathrm{Inner}_s(H, C) \oplus \mathrm{Wedge}_s(H, C)$$

where $\oplus$ is channel-wise concatenation followed by a learnable linear projection back to $\mathbb{F}_p^D$. For a shift set $S = \{s_1, \ldots, s_k\}$, the full interaction concatenates across all shifts:

$$\mathrm{Clifford}(H, C; S) = \mathrm{Linear}\!\left( \bigoplus_{s \in S} [HC]_s \right) \in \mathbb{F}_p^{N \times D}$$

### 5.4 Complexity

$O(N \cdot D \cdot |S|)$ time, $O(N \cdot D)$ space. Linear in every dimension.

### 5.5 Jet mapping

The shifted geometric product compiles to two [[nox]] jets:

- `shifted_inner_product` — extends the existing `geometric_product` jet used by `hull_attention` in nox
- `shifted_wedge_product` — new jet; same arithmetic structure as `shifted_inner_product` with subtraction

Reference shift set (default in CT-1.1 and render T∞):

$$S = \{1, 2, 4, 8, 16\}$$

(logarithmic — gives global channel mixing in $O(\log D)$ hops.)

---

## 6. Backward Compatibility

### 6.1 Scalar-only consumer contract

A consumer is **CT-1.0-compatible** iff it reads $X^{\star}_0$ and discards $X^{\star}_2$ for every extended quantity. Such a consumer produces output byte-identical to the unextended [[cybergraph]] specification.

Specifically:
- [[compiled transformers spec]] CT-1.0 is scalar-only: ignores $w_2$, $A^{\mathrm{eff}}_2$, and emits no Clifford jets.
- CT-1.1 is multivector-aware: reads both grades and emits Clifford jets for attention and MLP replacement.

### 6.2 Storage and wire format

Extended quantities serialize as two adjacent sections:
- scalar section: identical to legacy binary layout
- bivector section: sparse CSR of `(i, j, coefficient)` triples, ordered by `(i, j)` lexicographically, omitted entirely when empty

A `.graph` snapshot (see [[cyb-graph]]) with an empty bivector section is bit-identical to a legacy snapshot. CT-1.0 readers that do not know about the extension consume only the scalar section; the bivector section is skipped as an unknown extension per [[cyb-graph]] §extensions.

### 6.3 Hash invariance under scalar restriction

Let $H(G)$ denote the Hemera hash of graph state $G$. Then:

$$H(G^{\star}) \bigg|_{G_2 = 0} = H(G)$$

i.e., the hash of an extended graph with zero bivector part equals the legacy hash. This preserves content-addressing (A1) for the legacy subset.

---

## 7. Conformance

An implementation is Clifford-conforming on snapshot $G^{\star}$ iff:

### C1 — Scalar identity

$w^{\star}_0(p, q) \equiv w(p, q)$ and $A^{\mathrm{eff}\star}_0(p, q) \equiv A^{\mathrm{eff}}(p, q)$ for every $(p, q)$.

### C2 — Bivector anti-symmetry

$w^{\star}_2(p, q) = -w^{\star}_2(q, p)$ when interpreted as bivectors (the coefficient is stored canonically with $p < q$ or $q < p$ and the sign is flipped accordingly on read).

### C3 — Shifted product anti-symmetry

$\mathrm{Wedge}_s(H, H) \equiv 0$ for every $H, s$. A zero self-wedge is the defining property of the exterior product.

### C4 — Determinism

Two independent runs on the same $G^{\star}$ produce byte-identical extended quantities (same bivector coefficients in the same order).

### C5 — Jet equivalence

Running the shifted geometric product via the [[nox]] `shifted_inner_product` and `shifted_wedge_product` jets yields the same coefficients as a reference scalar-field implementation.

---

## 8. Open Items

Reserved for a later extension pass, not specified here:

- wedge-extended [[springs]] operator (rotational imbalance on directed Laplacians)
- bivector syntropy (directional circulation component)
- bivector [[cyberank]] (orientation of incoming focus support)
- mixed-grade Clifford $G(p, q, r)$ beyond scalar+bivector — reserved for [[Ren]]
- hemera bivector S-box hardening — reserved for a crypto-layer proposal

---

## References

- Ji, Z. *CliffordNet: All You Need is Geometric Algebra.* arXiv 2601.06793, 2026. — shifted geometric product as O(N) primitive.
- Dorst, Fontijne, Mann. *Geometric Algebra for Computer Science.* Morgan Kaufmann, 2007.
- Hestenes, D. *Space-Time Algebra.* Gordon and Breach, 1966.
- [[tri-kernel]] — scalar operators that remain unchanged.
- [[cybergraph]] — axioms and legacy primitive definitions.
- [[compiled transformers spec]] — CT-1.0 (scalar) and CT-1.1 (multivector) consumers.
- [[render]] — T∞ neural rendering backbone.
- [[nox]] — existing `geometric_product` jet in `hull_attention`.

discover all [[concepts]]
