---
tags: cyber, core, reference
alias: render spec, rendering reference, cybergraph render, 3d rendering spec
---
# Cybergraph Render Specification (R-1.0)

Formal specification of the deterministic rendering pipeline that projects a [[cybergraph]] snapshot onto a viewer's display. Derives 3D coordinates from the graph Laplacian, visual properties from derived fields ([[focus]], cluster ID, [[cyberank]]), and pixels from a multi-tier raster-plus-neural stack. Every participant running R-1.0 on the same authenticated graph state sees a topologically-identical world; pixel-level fidelity is not required.

This spec depends on and cites:

- [[cybergraph]] — `(P, N, L)`, axioms, adjacency operators
- [[tri-kernel]] — diffusion $\mathcal{D}$, springs $\mathcal{S}$, heat $\mathcal{H}_\tau$
- [[focus-flow]] — $\pi^*$ computation and local update rule
- [[clifford]] — multivector primitive extensions, shifted geometric product
- [[compiled transformers spec]] — CT-1.1 backbone for the T∞ tier

It does **not** depend on the [[crystal]] metadata schema. The engine renders pure topology by default; any metadata (domains, types, sizes) is consumed as a togglable overlay (§5.2).

## physics of the rendered world

Rendering does not impose structure from outside the graph. It is an observer — a camera pointed at the equilibrium state of a physical system. The world exists in the physics of [[cybergraph]]; R-1.0 reveals it.

The normalized Laplacian $\mathcal{L} = I - D^{-1/2} A^{\mathrm{eff}} D^{-1/2}$ is the discrete analog of the Laplace-Beltrami operator $\nabla^2$ on Riemannian manifolds. The springs equation from [[tri-kernel]],

$$(L + \mu I)x^* = \mu x_0$$

is the discrete analog of the Yukawa field equation $(\nabla^2 - \mu^2)\Phi = -\rho$, which governs the potential of a massive force mediator. The continuous Yukawa solution is $\Phi(r) \propto e^{-\mu r}/r$ — exponential decay with characteristic screening length $1/\mu$. The discrete Green's function $(L + \mu I)^{-1}$ is its graph analog: entry $(i, j)$ decays as $e^{-\sqrt{\mu}\,d(i,j)}$ with graph distance $d(i,j)$.

This Yukawa decay is not an algorithmic approximation — it is a law of the manifold. Particles beyond $O(1/\sqrt{\mu})$ hops exert exponentially negligible influence on each other's positions. The h-local update rule (§3.3) follows from this physics: recomputing only the $O(\log(1/\varepsilon))$-hop neighborhood of any edit is exact, not an approximation.

The [[tri-kernel]] fixed point minimizes a free-energy functional:

$$\mathcal{F}(\phi) = \lambda_s \!\left[\tfrac{1}{2}\phi^\top L\phi + \tfrac{\mu}{2}\|\phi - x_0\|^2\right] + \lambda_h \!\left[\tfrac{1}{2}\|\phi - H_\tau \phi\|^2\right] + \lambda_d \cdot D_{\mathrm{KL}}(\phi \,\|\, D\phi)$$

The elastic term $\frac{1}{2}\phi^\top L\phi$ is the discrete Dirichlet energy — the potential energy of the graph spring network. The screening term anchors focus against drift. The heat-alignment term enforces scale-consistent rendering. The KL term aligns focus with its own diffusion image, ensuring flow consistency. The world sits at the minimum of $\mathcal{F}$. Perturbations shift the minimum; the tri-kernel iteration rolls downhill.

Focus $\pi^*$ is a conserved scalar field on the manifold: $\sum_p \pi^*(p) = 1$ at all times. Total luminosity is invariant; brightness flows between particles as the graph evolves but is never created or destroyed.

The cybergraph is therefore a world in the same mathematical sense that a Riemannian manifold with matter fields is a world — defined by its geometry ($\mathcal{L}$), its dynamics ([[tri-kernel]]), and its conserved quantities ([[focus]]). R-1.0 is a specification for an observer of that world.

---

## 1. Scope

R-1.0 specifies a deterministic function

$$\mathrm{render}: (\mathbb{G}, \theta, v) \to \mathrm{Image}$$

where $\mathbb{G}$ is a cybergraph state, $\theta$ is a viewing configuration (camera pose + tier thresholds + τ), and $v$ is the backend identity. Two implementations conforming to R-1.0 on the same $(\mathbb{G}, \theta)$ must produce outputs that agree under the determinism contract of §12.

The spec covers:
- epoch-rate layout (§3) and coordinate frame (§4)
- visual encoding from derived fields (§5)
- five rendering tiers T0–T∞ (§6–7)
- spatial hierarchy (§10), navigation (§9), edges (§8)
- determinism contract (§12) and backend interface (§13)

It does **not** cover:
- UI chrome (menus, search bar, settings) — handled by the shell ([[bevy]] or host app)
- content sandboxing at T0 (WASM runtime caps) — deferred to a sandbox spec
- LLM-assisted narration at T3/T0 — deferred to a separate overlay spec

---

## 2. Pipeline

Rendering runs at two timescales:

| timescale | cadence | work |
|---|---|---|
| epoch | ≤ 1 Hz, background | layout (§3), coordinate frame alignment (§4), BVH build (§10), NRF retrain (§7) |
| frame | display refresh, foreground | cull, draw, animate flow (§6), composite |

Between epochs, positions and cluster IDs are frozen. Frames interpolate animated quantities (focus luminosity, diffusion flow) against fixed geometry. The epoch's outputs are published atomically to the frame thread via double-buffered GPU resources.

$$\begin{array}{l}
\text{epoch } k: \\
\quad (\pi^*, \phi^*) \leftarrow \text{tri-kernel}(\mathbb{G}_k) \quad \text{(see [[tri-kernel]])} \\
\quad X \leftarrow \text{spectral embed}(\mathbb{G}_k) \quad \text{(§3)} \\
\quad X' \leftarrow \text{Procrustes align}(X, X_{\text{anchor}}) \quad \text{(§4)} \\
\quad \mathrm{BVH} \leftarrow \text{build heat-kernel hierarchy}(X', \mathcal{H}_\tau) \quad \text{(§10)} \\
\quad \mathrm{NRF} \leftarrow \text{train}(\mathbb{G}_k, X', \pi^*, \mathrm{CT\text{-}1.1\text{ model}}) \quad \text{(§7)} \\[0.5em]
\text{frame } t \in [k, k+1): \\
\quad \mathrm{visible} \leftarrow \mathrm{cull}(X', \mathrm{BVH}, \mathrm{camera}) \\
\quad \text{dispatch tiers T0..T∞ on } \mathrm{visible} \\
\quad \text{animate } \pi^*(t), \text{diffusion flow} \\
\quad \mathrm{compose} \to \text{display surface}
\end{array}$$

---

## 3. Layout (Epoch)

Each epoch produces a 3D position $X: P \to \mathbb{R}^3$ for every particle via spectral embedding of the effective adjacency $A^{\mathrm{eff}}$ (see [[cybergraph]] §Derived Structures).

### 3.1 Spectral embedding

Let $\mathcal{L} = I - D^{-1/2} A^{\mathrm{eff}} D^{-1/2}$ be the normalized Laplacian of $\mathbb{G}$. Let $0 = \lambda_1 < \lambda_2 \leq \lambda_3 \leq \lambda_4 \leq \ldots$ be its eigenvalues with corresponding eigenvectors $u_1, u_2, u_3, u_4$. The position of particle $p$ is

$$X(p) = \big(u_2(p),\; u_3(p),\; u_4(p)\big) \in \mathbb{R}^3$$

(The trivial eigenvector $u_1 = \text{const}$ is discarded.) Uniqueness conditions and their resolutions:

- **Sign**: each eigenvector is unique up to a global sign. Fixed by "first nonzero component is positive in the canonical anchor basis" (see §4).
- **Eigenspace degeneracy**: when eigenvalues are repeated, their eigenvectors span a subspace and are unique only up to orthogonal rotation in that subspace. Resolved by Procrustes alignment to the anchor (§4).

### 3.2 Solver

Reference solver: **Lanczos iteration** on sparse $\mathcal{L}$ with $k = 32$ Krylov iterations, shift-invert around $\sigma = 0^+$. Deterministic with a canonical reduction order (pairwise tree-reduce, fixed dispatch block size). Numeric domain: $\mathbb{F}_p$ fixed-point with $2^{-32}$ resolution, or fp64 with enforced reproducible BLAS.

Complexity: $O(|P| \cdot \bar{d} \cdot k)$ where $\bar{d}$ is the average degree of $\mathbb{G}$.

### 3.3 Incremental update

For an edit batch $e_\Delta$, the affected positions lie within the h-hop neighborhood $N_h(e_\Delta)$ where $h = O(\log(1/\varepsilon))$ by [[cybergraph]] Theorem T4. The solver updates only $X|_{N_h}$ via perturbation theory; positions outside $N_h$ are frozen for the current epoch. Full eigendecomposition runs on a longer cadence (default: every 64 epochs, or when $\|\mathbb{G}_k - \mathbb{G}_{k-64}\|_L > 0.05$).

### 3.4 Scaling

Positions are scaled per epoch to fit a fixed scene radius $R_{\mathrm{scene}} = 1000$ (arbitrary units). The rescaling factor $s = R_{\mathrm{scene}} / \max_p \|X(p)\|$ is emitted in the epoch manifest for downstream tools.

---

## 4. Coordinate Frame

Spectral coordinates are determined up to an orthogonal transformation $Q \in O(3)$ and sign of each eigenvector. R-1.0 fixes both by Procrustes alignment to a canonical anchor subgraph.

### 4.1 Anchor subgraph

The **anchor set** $A_{\mathrm{anchor}} \subset P$ is the first 1024 particles by insertion order in $\mathbb{G}$'s canonical chain ordering. It is rebuilt only on a hard fork of the chain — in normal operation it is immutable across epochs.

Rationale: insertion order is deterministic and available at every neuron. The first 1024 particles are also the earliest — they have the most accumulated structural influence and the most stable spectral position.

### 4.2 Reference frame

The anchor's coordinates $X_{\mathrm{anchor}}: A_{\mathrm{anchor}} \to \mathbb{R}^3$ are computed at a **genesis epoch** (anchor epoch 0) using the full-graph Lanczos solver on $\mathbb{G}_0$. The result is hemera-hashed and stored as a chain-wide constant.

### 4.3 Procrustes alignment

Each epoch $k$ computes new positions $X_k$. The alignment transform $(Q_k, s_k, t_k) \in O(3) \times \mathbb{R}_+ \times \mathbb{R}^3$ is the Procrustes solution that minimizes

$$(Q_k, s_k, t_k) = \arg\min_{Q, s, t} \sum_{p \in A_{\mathrm{anchor}}} \| Q \cdot s \cdot X_k(p) + t - X_{\mathrm{anchor}}(p) \|_2^2$$

subject to $Q \in O(3)$. Aligned positions: $X'_k = Q_k \cdot s_k \cdot X_k + t_k$. Applied uniformly to all of $P$, not only the anchor.

### 4.4 Sign convention (SC-R)

If after alignment $Q_k$ has determinant $-1$ (reflection), invert the third coordinate axis and absorb the flip. The canonical frame is right-handed.

---

## 5. Visual Encoding

Every visual property derives from the graph or from derived fields. Metadata overlays are optional.

### 5.1 Schemaless defaults

| property | derivation | notes |
|---|---|---|
| position | §3 (spectral embedding, aligned via §4) | $X'(p) \in \mathbb{R}^3$ |
| radius | $r(p) = r_0 \cdot \sqrt{\pi^*(p)}$ | $r_0$ scene-scale constant |
| luminosity | $\ell(p) = L_0 \cdot \pi^*(p)$ | conserved: $\sum_p \ell(p) = L_0$ |
| hue | $h(p) = \theta(u_5(p), u_6(p))$ — angle of the 5th-6th eigenvector projection | schemaless cluster color |
| saturation | proportional to $|w_2|$ per-axon (from [[clifford]] extended adjacency) | disagreement desaturates |
| shape | sphere by default; upgraded from degree-role inference (hub / bridge / leaf) if requested | see §5.3 |
| edge weight | $A^{\mathrm{eff}}_0(p, q)$ (scalar part) | line thickness |
| edge flow direction | $P_{pq}$ (diffusion transition) + $\mathrm{sign}(A^{\mathrm{eff}}_2(p, q))$ | flow animation |

### 5.2 Metadata overlays

When the consumer supplies a metadata assignment $m: P \to \mathrm{Schema}$ (e.g., crystal-type / crystal-domain / crystal-size), it is applied as an overlay on top of the schemaless base. Presets:

- `preset = none` (default) — pure topology, as §5.1
- `preset = crystal-v5` — 6 types → shape primitives, 21 domains → triad-grouped palette (FORM / MASS / SPACE / LIFE / WORD / WORK / PLAY), 5 sizes → radius multiplier
- `preset = custom` — consumer-supplied lookup tables

Overlays are pure functions: $(\text{scheme}, m) \to (\text{shape}, \text{color}, \text{radius})$. They do not alter positions.

### 5.3 Role inference

When no shape overlay is active and `role_shapes = true`, particles are classified into three geometric primitives from graph-theoretic role signatures:

- **hub** (sphere): $\mathrm{deg}(p) > \mu_{\mathrm{deg}} + 2\sigma_{\mathrm{deg}}$
- **bridge** (torus): high between-ness centrality $> \theta_{\mathrm{bridge}}$
- **leaf** (tetrahedron): degree 1

Remaining particles render as spheres. The three thresholds are epoch-computed.

---

## 6. Rendering Tiers T0–T3

Five tiers of representation fidelity driven by projected screen size $s_p$ of particle $p$ (the screen-space radius after perspective projection).

| tier | condition | what renders |
|---|---|---|
| T0 content | $s_p > s_{\mathrm{T0}}$ (e.g., 200 px) | particle opens — content enters environment (§6.1) |
| T1 surface | $s_{\mathrm{T1}} < s_p \leq s_{\mathrm{T0}}$ | impostor + text label + edge preview |
| T2 shape | $s_{\mathrm{T2}} < s_p \leq s_{\mathrm{T1}}$ | instanced analytic impostor (sphere / torus / …) |
| T3 splat | $s_{\mathrm{T3}} < s_p \leq s_{\mathrm{T2}}$ | 3D Gaussian splat per particle, colored by §5 |
| T∞ field | $s_p \leq s_{\mathrm{T3}}$ | neural radiance field, no per-particle cost (§7) |

Default thresholds (screen-pixel diameter): $s_{\mathrm{T0}} = 200$, $s_{\mathrm{T1}} = 40$, $s_{\mathrm{T2}} = 8$, $s_{\mathrm{T3}} = 1$. Below 1 pixel, individual identity is not distinguishable — the handoff to T∞ is pixel-metric, not node-metric.

### 6.1 T0 — content

When a particle is close enough that its projected size exceeds $s_{\mathrm{T0}}$, it "opens": the camera flies through the surface and the particle's content becomes the environment until the camera flies back out. Content rendering dispatches by the particle's native language (see [[cyb/languages]]):

- `pixels` → image projects onto a panel
- `text` → typography renders on a curved surface
- `video` → playing video
- `component` → WASM sandboxed component with its own render pass (see [[cyb/component]])
- `formula` → KaTeX
- `vector` → SVG / 3D scene
- `struct`, `table`, `sound` — language-native renderers

Content fetches are sandboxed: WASM content runs with CPU / memory / GPU quotas, cannot access graph state beyond what the host passes explicitly.

### 6.2 T1 — surface

Impostor + text label (particle title or CID-prefix) + thin edge preview with flow texture. Edges render as tubes with per-edge flow direction texture driven by §8.2.

### 6.3 T2 — shape

Instanced analytic impostor. One quad per particle; fragment shader computes an analytic sphere / torus / tetrahedron (by §5.3 role inference or §5.2 crystal overlay) via ray-cast-in-fragment. No triangle mesh, no vertex buffer.

Driven by a single indirect draw dispatched from a GPU compute cull pass. Tile-shaded deferred on backends that support it (aruminium on Apple Silicon).

### 6.4 T3 — splat

Each particle becomes a 3D anisotropic Gaussian splat per Kerbl et al. (2023). Covariance is isotropic by default; optionally stretched along the local gradient of $u_5$ for directional-looking splats. Rasterized front-to-back with alpha blending.

### 6.5 Handoff

Particles in the transition band $[s_{\mathrm{T3}} - \epsilon, s_{\mathrm{T3}} + \epsilon]$ are rendered at **both** T3 and T∞ with alpha cross-fade to avoid pop. The NRF query at $X'(p)$ must match the splat's color to within an acceptance threshold (§12.2) for the handoff to be considered valid.

---

## 7. T∞ — Graph-as-Transformer Neural Rendering

This is the tier that makes R-1.0 scale to unbounded graph size: rendering cost is **per pixel**, not per particle. Every frame's cost is determined by screen resolution × samples, independent of $|P|$.

### 7.1 Architecture

The neural radiance field is a rendering head attached to the [[compiled transformers spec]] CT-1.1 model of $\mathbb{G}$.

$$\mathrm{NRF}: (x, y, z, \tau) \to (\rho, c, f)$$

with $\rho \in \mathbb{R}_+$ density, $c \in \mathbb{R}^3$ RGB color, $f \in \mathbb{R}^3$ flow vector.

The head consists of:

1. **Spatial positional encoding** — hash-grid encoding (Müller et al. 2022) with $L = 16$ levels, table size $2^{19}$ per level, feature dim 2. Output: 32-dim feature vector for any $(x, y, z, \tau)$.
2. **Graph-context conditioning** — a small cross-attention layer between the spatial feature and the CT-1.1 model's last hidden state at the nearest $k = 8$ particles (found via the BVH in §10). This is where the trained graph-knowledge enters the render.
3. **Clifford render block** — the shifted geometric product block from [[clifford]] §5 and [[compiled transformers spec]] §8.5. Default shift set $S = \{1, 2, 4, 8, 16\}$.
4. **Output head** — three linear projections to $\rho$ (via ReLU / softplus), $c$ (via sigmoid), $f$ (no activation, directional).

Typical head parameter count: $< 4$ MB fp16. Fits in the ANE SRAM. The CT-1.1 model itself remains graph-resident and is queried by the head.

### 7.2 Ray-march

Each pixel ray $r(t) = o + t d$ samples $N = 128$ points (stratified, jittered) between $t_{\mathrm{near}}$ and $t_{\mathrm{far}}$. At each sample:

$$(\rho_i, c_i, f_i) = \mathrm{NRF}(r(t_i), \tau(t_i))$$

with $\tau(t_i) = \tau_0 \cdot t_i / t_{\mathrm{far}}$ (farther samples query at larger heat-kernel scale, implementing LOD as a smooth function of depth).

Volume rendering equation:

$$C(r) = \sum_{i=1}^{N} T_i \cdot \big(1 - \exp(-\rho_i \delta_i)\big) \cdot c_i, \quad T_i = \exp\!\Big(-\sum_{j<i} \rho_j \delta_j\Big)$$

Flow: $F(r) = \sum_i T_i f_i$ is composited in a separate channel for edge flow animation overlay.

### 7.3 Training

At each epoch:

1. Sample $N_{\mathrm{train}} = 2^{16}$ points uniformly in $[{-}R_{\mathrm{scene}}, R_{\mathrm{scene}}]^3 \times [\tau_{\min}, \tau_{\max}]$.
2. Compute ground truth $(\rho^*, c^*, f^*)$ from the cybergraph directly:
   - $\rho^*(x, y, z, \tau) = \sum_p K_\tau(X'(p), (x, y, z)) \cdot \pi^*(p)$ where $K_\tau$ is a Gaussian kernel of width $\sqrt{\tau}$
   - $c^*$ from §5.1 luminosity-weighted average of nearby particles
   - $f^*$ from diffusion gradient
3. MSE loss on $(\rho, c, f)$, one Adam epoch ($\sim 8$ steps at 0.001).
4. Incremental update: only hash-grid cells inside the bounding box of $N_h(e_\Delta)$ receive gradient. Unchanged regions freeze.

Training cost per epoch: $\sim 50$ ms on ANE + GPU cooperation for a 1 M-particle graph. Lives in the epoch budget alongside Lanczos.

### 7.4 Determinism

The NRF is **not** bit-deterministic across backends (floating-point training order). The determinism contract (§12) makes it only **perceptually stable**: two trainings on the same $\mathbb{G}_k$ with the same seed must produce pixel outputs that match within $\Delta E^*_{\mathrm{CIE}} \leq 2.0$ on a reference viewpoint set.

Cross-neuron consistency: every neuron trains the NRF independently on its own snapshot. The training seed is $\mathrm{hemera}(\mathbb{G}_k \,\|\, \text{"nrf"})$ for convergence reproducibility given identical hardware; cross-hardware consistency falls back to the perceptual bound.

### 7.5 Fall-back

If the CT-1.1 model is unavailable (e.g., not yet compiled for $\mathbb{G}_k$), the NRF degrades to a hash-grid-only MLP without graph-context conditioning. This produces valid but less-informed T∞ rendering until the next CT-1.1 compile finishes.

---

## 8. Edges

### 8.1 Representation

Scalar edge weight $A^{\mathrm{eff}}_0(p, q)$ determines tube radius. Bivector $A^{\mathrm{eff}}_2(p, q)$ (from [[clifford]] §4) determines directional flow polarity.

### 8.2 Flow animation

Each frame $t$, flow offset $\phi(t) = \phi(t-1) + v \cdot \Delta t$ where $v = \mathrm{sign}(A^{\mathrm{eff}}_2) \cdot |P_{pq}|$ and $P$ is the diffusion transition matrix. Applied as a UV offset on a flow-strip texture.

### 8.3 Bundling

Edges are bundled per the heat-kernel hierarchy (§10) at tier T2 and above. Bundling clusters edges that share intermediate heat-kernel cluster membership and merges their geometry into a single tube with combined weight. Reduces edge-instance count by 10–100× at typical camera distances.

### 8.4 Labeling

Edge labels (predicate particle titles when present, see [[cyberlink]]) render at T1 only, attached to the tube midpoint with billboard orientation.

---

## 9. Navigation

### 9.1 Camera

6DOF camera: position $\mathbf{p} \in \mathbb{R}^3$, orientation $\mathbf{q} \in \mathrm{SO}(3)$, FOV ∈ [30°, 110°]. Input: keyboard translate (WASD + QE), mouse look, scroll zoom.

### 9.2 Warp

User inputs a CID or text query. Engine resolves to $p \in P$ (via particle index + text search on titles). Camera animates from current pose to $(X'(p) + 3 r(p) \hat{n}, \text{look at } X'(p))$ over 500 ms with a smooth-step ease.

### 9.3 Portal-step (T0 entry)

User clicks a particle at T1 or T2. Camera flies into the particle along the surface normal. When the projected screen size crosses $s_{\mathrm{T0}}$, the T0 content renderer takes over. Exit: camera flies back along the incoming path until projected size drops below $s_{\mathrm{T0}}$ again.

### 9.4 Follow-flow

User holds a modifier key. Each frame, camera velocity is biased toward the highest-weight outgoing cyberlink of the nearest particle. Produces a "ride the attention current" mode.

### 9.5 Camera-to-τ mapping

$\tau(\mathbf{p}) = \tau_0 \cdot (1 + \|\mathbf{p} - \mathrm{centroid}(\mathbb{G})\| / R_{\mathrm{scene}})^\alpha$ with $\tau_0 = 0.1$, $\alpha = 2$. Drives T∞ LOD smoothly with distance.

---

## 10. Spatial Hierarchy

### 10.1 Heat-kernel BVH

The bounding volume hierarchy is built by hierarchical heat-kernel clustering, reusing [[tri-kernel]] computation instead of introducing a separate octree build.

For $\tau$ at four scales $\{\tau_0, 10\tau_0, 100\tau_0, 1000\tau_0\}$, compute spectral clusters by thresholding the heat-kernel diffusion distance $\sqrt{K_\tau(p, q)}$ on $X'$. Each scale produces a partition; clusters at scale $k+1$ contain clusters at scale $k$. The resulting tree is the BVH.

Each BVH node stores:
- AABB of contained particles
- aggregated $\pi^*$ (luminosity sum, for T3 cluster billboards)
- dominant cluster color (for T∞ far-field tinting)
- child indices (up to 16 children per node)

### 10.2 Canonical label assignment

Cluster IDs at each scale are canonicalized:
1. Sort clusters by $\sum \pi^*$ descending.
2. Break ties by lowest CID of any member particle.
3. Assign IDs $0, 1, 2, \ldots$ in that order.

Deterministic across all neurons with the same $\mathbb{G}$.

### 10.3 Culling

GPU compute pass traverses the BVH per frame. Emits an indirect-draw argument buffer for each tier (T1, T2, T3). T∞ is always full-screen and does not need culling.

---

## 11. Complexity and Scale

### 11.1 Cost model

| operation | cost | cadence |
|---|---|---|
| Lanczos eigendecomp (full) | $O(\|P\| \cdot \bar{d} \cdot k)$ | every 64 epochs or on ΔG > 5% |
| Incremental eigendecomp | $O(\|N_h\| \cdot \bar{d})$ per edit batch | per epoch when applicable |
| Procrustes alignment | $O(\|A_{\mathrm{anchor}}\|)$ | per epoch |
| Heat-kernel BVH build | $O(\|P\| \cdot \log \|P\|)$ | per epoch |
| NRF training | $O(N_{\mathrm{train}})$, graph-size-independent | per epoch |
| Frame cull | $O(\log \|P\|)$ | per frame |
| Frame draw, tiers T0–T3 | $O(\text{visible instances})$ | per frame |
| Frame draw, T∞ | $O(\text{pixels} \cdot N_{\mathrm{samples}})$, graph-size-independent | per frame |

### 11.2 Scale targets

| regime | particles | edges | RAM (metadata) | target FPS @ 4K |
|---|---|---|---|---|
| crystal genesis | 5.0 × 10³ | 5 × 10⁴ | 7 MB | 120 |
| cyberia launch | 1 × 10⁶ | 1 × 10⁷ | 2.4 GB | 120 |
| planetary early | 1 × 10⁹ | 1 × 10¹⁰ | 2.4 TB | 120 |
| planetary mature | 3 × 10⁹ | 3 × 10¹⁰ | 7.2 TB | 120 |
| thermodynamic | 10¹⁵+ | 10¹⁶+ | tiered; content on IPFS | 60 (T∞-only views) |

8 TB-RAM neuron targets planetary-mature regime. 10¹⁵-particle rendering depends entirely on T∞ (graph-size-independent).

### 11.3 Streaming

Content (particle bodies at T0) streams from local SSD or IPFS at render time. Working set of T0 content: bounded by the camera shell — typically $< 1$ GB, easily cached.

Graph metadata beyond the current hot region (anchor + h-hop around the camera) is mmap'd and paged on demand. Hot region stays in RAM.

---

## 12. Determinism Contract

R-1.0 specifies three levels of cross-neuron invariance.

### 12.1 Topology stability (strict)

At epoch boundaries, every R-1.0 conformant implementation on the same $\mathbb{G}_k$ produces:

- bit-identical cluster membership at every BVH level
- bit-identical BVH tree structure
- bit-identical cluster ID assignments

This is what makes cyberspace knowable. "The math island" is at the same BVH path for everyone.

### 12.2 Position ε-stability

Positions $X'(p)$ differ across backends by at most $\varepsilon_X = 10^{-3} \cdot R_{\mathrm{scene}}$ (i.e., 0.1% of scene extent) in L2 norm per particle. This bound absorbs numerical differences across Lanczos implementations but not rotational ambiguity (the Procrustes anchor eliminates that).

### 12.3 Pixel freedom

Pixel values are not guaranteed bit-identical across backends. The perceptual constraint is $\Delta E^*_{\mathrm{CIE}} \leq 2.0$ against the cpu-reference backend on a fixed test-viewpoint set.

### 12.4 Verification

A **cpu-reference backend** (§13.2) is authoritative for determinism checks. Implementations are certified by rendering a standard test suite of $(\mathbb{G}, \theta)$ pairs and comparing:

- topology: bit-match required
- positions: within $\varepsilon_X$
- pixels: within $\Delta E^*$

---

## 13. Backend Interface

### 13.1 Trait

```rust
trait RenderBackend {
    // GPU resources
    fn buffer_create(&self, size: usize, usage: BufferUsage) -> BufferId;
    fn buffer_write(&self, id: BufferId, data: &[u8], offset: usize);
    fn buffer_read(&self, id: BufferId, offset: usize, size: usize) -> Vec<u8>;

    // compute
    fn shader_compile(&self, source: &str, stage: ShaderStage) -> ShaderId;
    fn compute_dispatch(&self, shader: ShaderId, bindings: &[Binding], workgroups: [u32; 3]);

    // raster
    fn pipeline_create(&self, shaders: &PipelineShaders, layout: &PipelineLayout) -> PipelineId;
    fn draw_indirect(&self, pipeline: PipelineId, bindings: &[Binding], indirect_buffer: BufferId, count: u32);

    // presentation
    fn surface_acquire(&self) -> TextureId;
    fn surface_present(&self, texture: TextureId);

    // Clifford primitives (required for T∞ Clifford-block variants)
    fn roll(&self, tensor: BufferId, shift: i32, axis: u32) -> BufferId;
    fn shifted_inner(&self, h: BufferId, c: BufferId, shifts: &[i32]) -> BufferId;
    fn shifted_wedge(&self, h: BufferId, c: BufferId, shifts: &[i32]) -> BufferId;
    fn clifford_block(&self, h: BufferId, c: BufferId, shifts: &[i32], gate: BufferId) -> BufferId;

    // determinism contract
    fn deterministic_mode(&self) -> bool;  // promise: fixed reduce order, fixed FMA, fixed dispatch
    fn numeric_domain(&self) -> NumericDomain;  // FP32 | FP64 | FixedQ32 | Goldilocks
}
```

### 13.2 Required backends

- **honeycrisp** — `aruminium` (Metal GPU) + `rane` (ANE) + `acpu` (AMX CPU), zero-copy via `unimem` IOSurface. Primary backend. §14.
- **cpu-reference** — single-threaded, fp64, deterministic reduce order. Used for golden-image generation and cross-backend verification. Slow, correct.

### 13.3 Optional backends

- `wgpu` — cross-platform fallback (Vulkan / Metal-via-wgpu / DX12 / WebGPU)
- `webgpu` — browser target with aggressive LOD degradation
- `vulkan-native` — Linux with vendor extensions (mesh shaders, ray-tracing)

Each documents its degradation profile (which features downgrade) and its determinism profile against cpu-reference.

---

## 14. Honeycrisp Backend

Primary reference implementation for R-1.0.

| component | role |
|---|---|
| [[aruminium]] | Metal GPU raster pipelines, compute shaders for cull / impostor / splat / edge passes |
| [[rane]] | ANE inference of NRF head (T∞) and Clifford block ops |
| [[acpu]] | AMX-accelerated Lanczos eigendecomp, Procrustes alignment, BVH build |
| [[unimem]] | IOSurface-backed shared buffers across CPU / GPU / ANE, zero copies |

Missing primitives to add (see [[honeycrisp]] roadmap):

- `roll(tensor, shift, axis)` — cyclic channel shift, needed for §7 Clifford block and §13.1 trait
- `shifted_inner`, `shifted_wedge`, `clifford_block` — fused kernels per [[clifford]] §5
- MetalFX upscaling integration — render at 0.5× resolution, reconstruct via temporal ML
- EDR / P3 wide-color hooks for focus luminosity and cluster hue

Frame budget on M3 Pro at 4K 120 Hz (8.3 ms), 1 M-particle graph:

```
cull + LOD compute:      0.3 ms   (aruminium)
NRF inference (ANE):     0.8 ms   (rane)
T2 impostor pass:        1.5 ms   (aruminium, tile-shaded)
T3 splat pass:           1.0 ms   (aruminium)
edge pass, bundled:      1.0 ms   (aruminium)
T∞ composite:            0.7 ms   (aruminium)
MetalFX upscale:         0.6 ms   (aruminium)
UI + present:            0.4 ms   (aruminium)
slack:                   2.0 ms
```

---

## 15. Conformance

An implementation is R-1.0 conforming on test suite $\mathcal{T}$ iff the following hold for every test pair $(\mathbb{G}, \theta) \in \mathcal{T}$.

### 15.1 Topology (P-RENDER-TOPO)

BVH and cluster IDs at every level match the cpu-reference backend bit-for-bit.

### 15.2 Position (P-RENDER-POS)

$\max_p \|X'_{\mathrm{impl}}(p) - X'_{\mathrm{ref}}(p)\|_2 \leq \varepsilon_X$ where $\varepsilon_X = 10^{-3} \cdot R_{\mathrm{scene}}$.

### 15.3 Pixel perceptual (P-RENDER-PIX)

Mean CIE ΔE* against the cpu-reference backend on a 16-viewpoint fixed test set is $\leq 2.0$.

### 15.4 Frame budget (P-RENDER-FPS)

On a defined reference hardware (M3 Pro or equivalent), the frame budget of §14 holds at 120 FPS on the 1 M-particle test snapshot.

### 15.5 T∞ graph-size independence (P-RENDER-T∞)

Frame time of a pure-T∞ view (all particles sub-pixel) is within 10% across snapshots of $10^6$ vs $10^9$ particles with the same CT-1.1 head size.

Results stored in a sidecar `r1_conformance.toml`:

```toml
[r1_conformance]
P_RENDER_TOPO = 1
P_RENDER_POS  = 1   # max deviation 2.8e-4 * R_scene, below 1e-3 bound
P_RENDER_PIX  = 1   # mean ΔE* = 1.4, below 2.0 bound
P_RENDER_FPS  = 1   # 120 FPS sustained
P_RENDER_TINF = 1   # 7% frame time delta across 10^6 and 10^9
```

---

## 16. Open Items

Reserved for R-1.1 or later:

- decoupled shift sets $S_{\mathrm{inner}} \neq S_{\mathrm{wedge}}$ in the T∞ Clifford block
- LLM-narrator overlay at T3 (cluster labels) and T0 (contextual narration)
- bivector-aware edge rendering (§8.1 extended to grade-2 visualization)
- adaptive NRF training cadence based on $\|\mathbb{G}_k - \mathbb{G}_{k-1}\|$
- volumetric content at T0 (3D-native particles, not just 2D-projected media)
- rotor-native camera controls (gesture → versor composition)
- collaborative rendering: two neurons seeing the same viewpoint simultaneously
- occlusion culling via hierarchical depth buffer
- variable-rate shading for peripheral vision (VR mode)

---

## References

- Ji, Z. *CliffordNet: All You Need is Geometric Algebra.* arXiv 2601.06793, 2026.
- Kerbl, B. et al. *3D Gaussian Splatting for Real-Time Radiance Field Rendering.* SIGGRAPH, 2023.
- Müller, T. et al. *Instant Neural Graphics Primitives with a Multiresolution Hash Encoding.* SIGGRAPH, 2022.
- Hammond, D. K., Vandergheynst, P., Gribonval, R. *Wavelets on graphs via spectral graph theory.* ACHA, 2011.
- Fiedler, M. *Algebraic connectivity of graphs.* Czech Math Journal, 1973.
- Spielman, D. *Spectral Graph Theory.* Yale Lecture Notes.
- Koren, Y. *Drawing graphs by eigenvectors: theory and practice.* Computers & Mathematics with Applications, 2005.
- Holten, D. *Hierarchical edge bundles.* IEEE TVCG, 2006.

See [[cybergraph]] for primitives. See [[tri-kernel]] for the layout operators. See [[clifford]] for multivector extensions and the shifted geometric product. See [[compiled transformers spec]] for the CT-1.1 backbone. See [[honeycrisp]] for the primary backend.

discover all [[concepts]]
