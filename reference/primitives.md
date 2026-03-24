---
tags: cyber, core, reference
alias: primitives spec, particle spec, cyberlink spec, signal spec, axon spec
---
# Primitives

Formal specification of the four primitive structures of the [[cybergraph]]: particle, cyberlink, signal, and axon.

---

## Particle

A [[particle]] is a content-addressed node. Identity = [[Hemera]] hash of content. 64 raw bytes, no headers, no version prefix. One hash function, one address space, permanent.

The address is the identity. `Hemera(content)` -- that is the particle. No registration, no authority, no namespace collision. Two agents on opposite sides of the planet hashing the same content produce the same address. The first [[cyberlink]] to that address brings the particle into the [[cybergraph]]. A naked hash with no links never enters the graph.

### Hemera

```
Hemera = Poseidon2(
  p  = 2^64 - 2^32 + 1     Goldilocks field
  d  = 7                   S-box: x -> x^7
  t  = 16                  state width (elements)
  Rf = 8                   full rounds (4 + 4)
  Rp = 64                  partial rounds
  r  = 8                   rate (64 bytes in)
  c  = 8                   capacity (64 bytes)
  out = 8 elements          64 bytes out
)
```

Every parameter is a power of 2. The [[Goldilocks field]] gives native 64-bit CPU arithmetic -- a field multiplication is a single instruction. The S-box exponent $d = 7$ is the minimum invertible exponent for this field ($\gcd(7, p-1) = 1$; both 3 and 5 divide $p-1$).

Capacity 8 (256-bit) provides 256-bit classical collision resistance, 170-bit quantum collision resistance (BHT), and algebraic degree $7^{64} \approx 2^{180}$. Production systems use capacity 4 (128-bit) because their hashes are ephemeral -- trace commitments that live seconds. Particle addresses live decades. The parameter choice matches the lifetime.

One mode only: sponge. No compression mode. Two modes producing the same 64-byte output from different inputs would break the address space as a function.

```
initialize:  state <- [0; 16]
absorb:      for each 8-element chunk of padded input:
               state[0..8] ^= chunk
               state <- permute(state)
squeeze:     output <- state[0..8]
```

Round constants are self-bootstrapping: Hemera generates its own constants from the seed `"cyber"` (5 bytes) through the zero-constant permutation. No foreign primitives in the dependency chain.

See [[hemera/spec]] for the full decision record.

### Tree Structure

Large content splits into 4 KB chunks -- OS page aligned, L1 cache fit, 512 field elements per chunk, 64 absorb blocks per leaf.

```
leaf:          Hemera(chunk_bytes)
internal node: Hemera(left_id || right_id)    128 bytes in, 64 bytes out
tree shape:    binary, left-balanced
particle:      root hash of the tree
```

Left-balanced means the same content prefix always produces the same left subtree. Streaming: buffer at most 4 KB + proof per step. Deduplication: 4 KB blocks show meaningful repetition in real data. Overhead: 1.6% tree metadata.

A single chunk (<=4 KB) hashes directly -- no tree, just `Hemera(content)`. The particle address is the same whether content is 10 bytes or 10 gigabytes: always 64 bytes, always a Hemera output.

### Domain Separation

Different uses of Hemera are separated at the input:

| prefix | domain |
|--------|--------|
| `0x01` | edge hashing |
| `0x02` | record commitments |
| `0x03` | nullifier derivation |
| `0x04` | Merkle internal nodes (NMT, MMR) |
| `0x05` | Fiat-Shamir challenges (WHIR) |
| `0x06` | proof transcript binding |

`H_edge(x) = Hemera(0x01 || x)`. Particle content addressing uses no prefix -- bare content in, address out. The particle address space is the default.

### Output Format

```
IPFS CIDv1:  <version><multicodec><multihash><length><digest>   36-69 bytes
nox CID:     <digest>                                           64 bytes
```

Inside the protocol, the 64-byte digest is the complete identifier. IPFS compatibility is a thin translation layer at the gateway -- inside [[nox]], the wrapper never exists.

All identities live in one flat 64-byte namespace: [[particles]], edges, [[neurons]], commitments, nullifiers. No type tags in the address. The type is determined by where the address appears in the [[BBG]] structure.

### Endofunction

`Hemera(Hemera(x) || Hemera(y))` type-checks: 64 bytes in one side, 64 bytes the other, 64 bytes out. Hash of hashes is a hash. This closure under composition is why Merkle trees, polynomial commitments, and recursive proofs all use the same function without conversion.

### Permanence

| property | zkVM (SP1, RISC Zero) | cyber |
|----------|----------------------|-------|
| hash lifetime | seconds to hours | decades to permanent |
| parameter update | software release | impossible without rehash |
| rehash cost | zero (ephemeral) | $O(10^{15})$ operations |
| cost of parameter error | reissue proofs | lose the graph |

If Hemera is ever broken: full graph rehash under a new primitive. No version byte, no algorithm agility, no graceful coexistence. One graph, one hash, one identity. Storage proofs make this possible -- they guarantee content availability for rehashing and must be operational before genesis.

### Performance

| metric | Hemera | SHA-256 in STARK |
|--------|--------|-----------------|
| hash rate (single core) | ~62 MB/s | ~200 MB/s |
| STARK constraints per hash | ~1,200 | ~25,000 |
| particles per second (200 B avg) | ~310K | -- |

20x cheaper in proofs than SHA-256. 0.6x the raw throughput. The tradeoff: particle addresses are verified far more often than they are created. Optimizing for proof cost is optimizing for the common case.

---

## Cyberlink

The atomic unit of [[knowledge]]. A [[neuron]] binds two [[particles]] with a signed, staked, timestamped assertion -- every cyberlink is simultaneously a [[learning]] act and an economic commitment.

### The Seven Fields

$$\ell \;=\; (\nu,\; p,\; q,\; \tau,\; a,\; v,\; t) \;\in\; N \times P \times P \times \mathcal{T} \times \mathbb{R}_{+} \times \{-1,\,0,\,+1\} \times \mathbb{Z}_{\geq 0}$$

| field | name | type | layer | semantics | question |
|-------|------|------|-------|-----------|----------|
| $\nu$ | [[subject]] | $N$ | structural | signing [[neuron]] | [[who]] asserts this? |
| $p$ | from | $P$ | structural | source [[particle]] | [[what]] is the source? |
| $q$ | to | $P$ | structural | target [[particle]] | [[what]] is the target? |
| $\tau$ | token | $\mathcal{T}$ | economic | token denomination | in what denomination? |
| $a$ | amount | $\mathbb{R}_+$ | economic | stake amount | how much conviction? |
| $v$ | [[valence]] | $\{-1,0,+1\}$ | epistemic | [[Bayesian Truth Serum\|BTS]] meta-prediction | what is the epistemic prediction? |
| $t$ | at | $\mathbb{Z}_{\geq 0}$ | temporal | block height | [[when]]? |

Three layers in one atomic record. Structural $(\nu, p, q)$ is binary -- the connection either exists or it does not. Epistemic $v$ is ternary -- the neuron's prediction of how the [[inversely coupled bonding surface|ICBS]] market on this edge will converge. Economic $(\tau, a)$ is continuous over $\mathbb{R}_+$. See [[two three paradox]] for why this layering is not arbitrary.

Conviction = ($\tau$, $a$): the pair that turns an assertion into a bet. Denomination selects the [[token]], amount declares the stake. A link with zero conviction is structurally identical to a link with maximum conviction -- the structural layer is binary. The conviction layer prices it.

Cyberlinks are bundled into [[cyber/signals]] for broadcast. The [[cyber/signal]] adds the computational layer: an [[cyber/impulse]] ($\pi_\Delta$ -- the proven [[focus]] shift) and a recursive [[stark]] proof covering the entire batch. See [[cyber/signal]] for the full specification.

The [[cybergraph]] is append-only. $t$ (block height) distinguishes every record: the same author linking from->to at block $t_1$ and again at block $t_2 > t_1$ produces two separate entries in $L$. This enables reinforcement (higher $a$ on a new record), valence updates (new $v$ at a new block), and multi-denomination staking (same structural link in different [[tokens]]).

### UTXO Semantics

Conviction is a [[UTXO]]. Creating a cyberlink is a transaction: the author moves $a$ tokens of denomination $\tau$ from a wallet UTXO to a new output bound to the cyberlink record. Funds always move from one object to another.

The conviction output can itself be spent:

- Transfer: spend the conviction UTXO to a new owner. The structural record stays in $L$; beneficial ownership moves. This is how the card's transferability operates at the protocol level.
- Withdraw: spend the conviction UTXO back to the author's wallet. The economic position closes. The structural record remains.

The non-fungibility of the card (unique 7-tuple) and the fungibility of the token (transferable UTXO) coexist: the assertion is non-fungible, the economic position is a standard UTXO output.

### CRUD Operations

The append-only graph expresses all four operations through cyberlinks:

| operation | cyberlink action | what changes |
|-----------|-----------------|--------------|
| create | first record for structural triple $(\nu, p, q)$ | relation enters $L$ |
| read | query $\pi^*$ at any block -- no link required | nothing |
| update | new record with new $(\tau, a, v, t)$ for the same triple | any mutable dimension |
| delete | withdraw conviction UTXO + new record with $v = -1$ | economic position closed, epistemic signal negated |

The three mutable dimensions -- epistemic ($v$), economic ($a$), and temporal ($t$) -- vary independently. Every combination is meaningful:

| $v$ | $a$ | reading |
|-----|-----|---------|
| $+1$ | high | funded affirmation -- bet the market confirms |
| $+1$ | zero | unfunded affirmation -- structural + epistemic signal, no economic exposure |
| $0$ | high | funded agnostic -- stake without prediction |
| $0$ | zero | bare assertion -- structural fact only |
| $-1$ | high | funded short -- bet the market rejects |
| $-1$ | zero | logical retraction -- epistemic negation, no economic exposure |

$v = -1$ does not mean the structural link is absent. The connection $p \to q$ is permanent (A3). $v = -1$ is the [[subject]]'s prediction that the [[inversely coupled bonding surface|ICBS]] market on this edge will converge to FALSE -- a funded short when $a > 0$, a pure retraction when $a = 0$.

Delete in the graph is never erasure. The record $(\nu, p, q, t_{\text{first}})$ stays in $L$ permanently. Economic close and epistemic retraction are separable operations -- a subject can withdraw conviction while keeping $v = +1$, or submit $v = -1$ while maintaining stake. The full semantic delete is both together.

### The Card

Every cyberlink is also a card -- an epistemic asset with four properties:

Immutable. Axiom A3 (append-only) guarantees the record $\ell = (\nu, p, q, \tau, a, v, t)$ is permanent once published. The assertion cannot be altered or retracted. The author's conviction, valence, and timestamp are locked into the graph's history forever. Immutability is what makes the card a credible commitment rather than a revisable claim.

Unique. The 7-tuple is the card's identity -- no two cyberlinks are identical (block height $t$ ensures this even when the same author re-links the same particles). Each card is non-fungible: it is a specific assertion, by a specific author, at a specific block, with a specific conviction.

Transferable. Ownership of a cyberlink -- and thus the rights to its yield and governance weight -- can be transferred between [[neurons]]. The structural record stays in $L$ forever; beneficial ownership moves. This separates the assertion (immutable, authorial) from the economic position (transferable, tradeable).

Yield-bearing. A cyberlink earns in proportion to how much the target particle gains [[focus]]:

$$R_\ell(T) = \int_0^T w(t) \cdot \Delta\pi^*(q, t)\, dt$$

where $w(t)$ is the conviction weight at time $t$ and $\Delta\pi^*(q, t)$ is the increment in the target particle's focus. A link that correctly anticipated an important particle -- created early, with genuine conviction -- earns the most. Early discovery is maximally rewarded; late consensus-following earns little.

The card unifies what financial instruments split: the assertion (content), the commitment (conviction), the epistemic signal (valence), and the yield right -- all in one atomic, immutable, tradeable record.

### Edge Labeling

A cyberlink has no built-in type field. Labeling works through the graph itself: every directed edge induces an [[axon]]-[[particle]] via axiom A6 ($H(p, q) \in P$). To label an edge, create a cyberlink from a type-[[particle]] to the [[axon]]-[[particle]]:

```
A --cyberlink--> B                  the assertion
"is-a" --cyberlink--> axon(A, B)    the label
```

Any [[particle]] can serve as a label: `is-a`, `contradicts`, `extends`, `cites`, `created-by`. The label itself has [[cyberank]], [[karma]], market price -- the graph weights the importance of relation types the same way it weights everything else.

This means no new primitive is needed. The seven fields of the cyberlink tuple remain unchanged. Metadata, annotations, and type labels are all cyberlinks to [[axon]]-[[particles]] -- the graph describes its own structure.

---

## Signal

A bundle of [[cyberlinks]] a [[neuron]] commits in a single [[step]] -- the atomic broadcast unit in [[cyber]]. Each link in the signal consumes [[focus]], making every statement a [[costly signal]].

### Structure

$$s \;=\; (\nu,\; \vec\ell,\; \pi_\Delta,\; \sigma,\; t)$$

| field | name | type | semantics |
|-------|------|------|-----------|
| $\nu$ | [[subject]] | $N$ | signing [[neuron]] |
| $\vec\ell$ | links | $L^+$ | one or more [[cyberlinks]] -- each a 7-tuple $(\nu, p, q, \tau, a, v, t)$ |
| $\pi_\Delta$ | [[cyber/impulse]] | $(P \times \mathbb{F}_p)^*$ | sparse [[focus]] update: how the batch of links shifts $\pi^*$ |
| $\sigma$ | proof | $\Pi$ | recursive [[stark]] proof covering the [[cyber/impulse]], all conviction UTXO movements, and [[cyberlink]] validity against the current [[BBG]] root |
| $t$ | at | $\mathbb{Z}_{\geq 0}$ | block height |

The signal separates what a [[neuron]] asserts (the [[cyberlinks]]) from what the assertion computes (the [[cyber/impulse]]). See [[cyber/impulse]] for how $\pi_\Delta$ is computed.

### STARK Proof Coverage

$\sigma$ is a single recursive [[stark]] proof that covers the entire signal atomically:

- Correctness of each [[cyberlink]] in $\vec\ell$ (valid signatures, valid particle references)
- Validity of all conviction UTXO movements (each link's $(\tau, a)$ spend is backed by an unspent output)
- Correctness of the [[cyber/impulse]] $\pi_\Delta$ (the [[tri-kernel]] computation against $\text{bbg\_root}$ from the current header)

One proof for everything. Proving $n$ links together costs less than $n$ separate proofs because shared neighborhood state and UTXO set are proved once. Any verifier checks $\sigma$ in $O(\log n)$ without recomputing anything.

### Two Effects

Validation of a signal produces two outcomes:

1. Each link in $\vec\ell$ enters $L$ -- conviction UTXOs are created for each [[cyberlink]]
2. If $\|\pi_\Delta\| > 0$ and $\sigma$ is valid, the [[neuron]] self-mints [[$CYB]] proportional to the proven shift -- a reward UTXO is created for $\nu$

The conviction UTXOs (tokens spent into links) and the reward UTXO (tokens minted for contribution) are separate token movements within one atomic signal. See [[cyber/rewards]] for the full reward specification.

### Minting Conservation

Total minting per epoch is bounded by the actual global $\Delta\pi$, verifiable from consecutive headers. If the sum of individual claims exceeds the actual shift (overlapping neighborhoods), all claims are scaled proportionally.

---

## Axon

An axon is the bundle of all [[cyberlinks]] between two [[particles]] across all [[neurons]] and time. If a [[cyberlink]] is a synapse, an axon is the nerve fiber. Weight sums contributions from many [[neurons]], reflecting collective judgment.

### Definition

Axons emerge from the [[cybergraph]]; they are never created directly. The natural unit for the [[tri-kernel]]: [[diffusion]] flows along them, [[springs]] constrain them, [[heat]] smooths across them.

### Weight Computation

The axon weight for the directed pair $(p, q)$ is the aggregate of all cyberlinks from $p$ to $q$:

$$w_{\text{axon}}(p, q) = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

This feeds directly into the adjacency operator $A_{pq}$.

### Homoiconicity -- Axon as Particle

Every axon is a [[particle]]: $H(\text{from}, \text{to}) \in P$. The hash of the directed edge induces a content-addressed node in the [[cybergraph]]. This means axons have [[cyberank]], receive [[focus]], carry [[value]], and can themselves be targets of [[cyberlinks]]. The graph ranks its own structure.

### Meta-Annotation

You can [[cyberlink]] TO an axon -- meta-annotating a relationship. You can stake on axon-particles -- betting on the importance of a connection. [[Focus]] flows through axon-particles alongside content-particles.

See [[cyber/axon]] for the full formal specification.
