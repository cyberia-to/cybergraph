---
tags: cyber, docs
alias: cybergraph overview, what is cybergraph
---
# What Is the Cybergraph

The cybergraph is a directed, authenticated multigraph over content-addressed nodes, carrying an emergent [[probability]] measure -- the shared memory of the planet.

In plain language: it is a graph where every node is a piece of content (identified by its hash), every edge is a signed, staked claim by someone, and the whole structure continuously computes a single number for every node -- how much collective attention it deserves. That number is [[focus]], and the process that computes it is the [[tri-kernel]].

---

## Five Primitives

The entire protocol runs through five primitives. Everything else is derived.

### Particles

A particle is a content-addressed node. You take any piece of content -- a sentence, a file, an image, a proof -- and hash it with [[Hemera]]. The 64-byte output is the particle's identity. Same content always produces the same particle, no matter who hashes it or when. Two researchers on opposite sides of the planet thinking the same thought produce the same address. There is no registration authority, no namespace, no collision.

A particle comes into existence the moment someone links to it. A naked hash floating with no connections is not a particle. The graph creates its nodes through the act of linking.

### Cyberlinks

A cyberlink is the atomic unit of knowledge. It is a signed, staked, timestamped edge from one particle to another. Seven fields: who made it, where it comes from, where it goes, what token backs it, how much stake, what epistemic prediction (positive, neutral, or negative), and when. Three layers packed into one record -- the structural fact (the connection exists), the economic commitment (how much the author is willing to bet), and the epistemic signal (what the author predicts the market will believe).

Every cyberlink is also a card -- an immutable, unique, transferable, yield-bearing epistemic asset. The assertion is permanent (append-only). The economic position can be transferred or withdrawn. Early, correct links earn the most because they discover important particles before the crowd arrives.

### Neurons

A neuron is an authenticated agent -- an entity with a public key that signs cyberlinks. Neurons hold tokens, which determine how much weight their links carry. A neuron's accumulated track record is [[karma]] -- the history of how well their predictions matched collective outcomes.

### Tokens

Tokens are conviction denominations. When a neuron creates a cyberlink, they move tokens into the link as stake. The token amount determines the link's weight in the graph. Multiple token denominations can coexist, each pricing a different dimension of conviction.

### Focus

Focus is the collective attention distribution. It is a probability measure over all particles: every particle gets a number between 0 and 1, and all numbers sum to 1. Focus is computed by the [[tri-kernel]] -- three local operators that blend exploration, structure, and adaptation into a single fixed point. Focus is conserved: it flows between particles but is never created or destroyed.

---

## Why Content Addressing Matters

Traditional systems assign identifiers through registries -- URLs, DOIs, ISBNs. These identifiers are opaque: the identifier says nothing about the content, and changing the content does not change the identifier. This creates a gap between identity and truth.

Content addressing closes the gap. The address IS the content (or rather, a collision-resistant hash of it). If the content changes, the address changes. If two copies have the same address, they have the same content. Identity equals content, permanently.

At planetary scale ($10^{15}$ particles), this matters economically. Every byte of framing overhead -- version prefixes, multicodec headers, hash function tags -- multiplies by $10^{15}$. Five bytes of overhead becomes five petabytes of pure waste. Content addressing with a single, fixed hash function eliminates all framing. 64 bytes in, 64 bytes out, forever.

---

## Why Append-Only

The cybergraph grows monotonically. A cyberlink, once created, cannot be deleted. This is axiom A3, and it is a feature.

Append-only means the historical record is tamper-proof. Every assertion ever made is preserved with its author, its stake, its timestamp. This is what makes the card (the cyberlink as epistemic asset) credible: you cannot go back and erase a wrong prediction.

Deletion is expressed through economics and epistemics, not erasure. A neuron can withdraw their stake (closing the economic position) and submit a negative valence (epistemic retraction). The original record remains, but its effective weight approaches zero. The graph remembers everything; the tri-kernel computes what matters now.

---

## The Cybergraph as Shared Memory

The cybergraph is the shared memory of all participants. Every neuron contributes links. The tri-kernel integrates all contributions into a single, convergent focus distribution. This distribution is the collective's answer to the question: "What is important?"

This is a different relationship between data and computation than traditional systems. In a database, data sits passively until queried. In a knowledge graph, relations encode structure but require external inference engines. In the cybergraph, the data structure itself computes: the graph continuously converges toward its own fixed point, and that fixed point is the model.

The focus distribution $\pi^*$ IS the model. It is the probability measure over particles that minimizes a free energy functional -- the balance between exploration (diffusion), structural coherence (springs), and adaptive context (heat). Adding a correct, well-placed cyberlink is equivalent to taking a gradient step on this functional. Every link teaches the system.

---

## How It Differs from Databases and Knowledge Graphs

A database stores records and answers queries. It does not compute importance, does not weight assertions by conviction, does not converge toward collective agreement. The database is passive.

A knowledge graph (RDF, Wikidata, property graphs) stores typed relations between entities. It encodes structure but requires external algorithms to compute rankings, infer missing links, or resolve contradictions. The knowledge graph is structural but inert.

The cybergraph is a knowledge graph that computes. Every edge carries economic weight. Every node receives a probability score from the tri-kernel. The structure is authenticated (every edge is signed). The computation is continuous (every new link shifts the fixed point). The result is verifiable (every state transition has a [[zheng]] proof).

Fifteen protocol functions -- identity, key exchange, authentication, consensus, fork choice, finality, privacy, incentives, relay payment, version control, file system, type system, computation, data availability, sybil resistance -- all run through the same five primitives. One data structure, one computation, one shared memory.

---

See [[cybergraph]] for the formal definition. See [[tri-kernel]] for the three operators. See [[collective focus theorem]] for convergence proofs. See [[focus flow computation]] for the inference process.
