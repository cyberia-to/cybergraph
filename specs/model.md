---
tags: cybergraph, specs
crystal-type: entity
crystal-domain: cyber
---
# programming model

two objects. one primitive.

**token** — the only value-carrying object. two standards, two conservation laws:

| standard | type | invariant |
|---|---|---|
| TSP-1 coin | fungible | `Σ balances = supply` |
| TSP-2 card | unique | `owner_count(id) = 1` |

**cyberlink** — the only action. moves one token between two tokens:

```
cyberlink(from: TokenId, to: TokenId, token: TokenId, amount: u64, valence: {-1, 0, +1})
```

every cyberlink is simultaneously an economic act, a semantic assertion, and an epistemic prediction.

---

## everything is a token

neurons, particles, cards — all are tokens. the protocol has no special-cased entity types.

| entity | standard | subtype skill | id derivation |
|---|---|---|---|
| coin | TSP-1 | — | denomination |
| card | TSP-2 | asset | hash(content) |
| particle | TSP-2 | knowledge | hemera(content) |
| neuron | TSP-2 | identity | hemera(pubkey) |

particles and neurons are cards distinguished by their skills — composable behavior hooks — not by separate primitive types. a particle is a card with the knowledge skill. a neuron is a card with the identity skill.

**neuron = identity card**: runs nox programs via the prog skill. creates cyberlinks autonomously. no protocol privileges — its cyberlinks pass through identical plumb validation as any other actor.

**particle = knowledge card**: id derived from content (hemera hash). accumulates fungible tokens from incoming cyberlinks. conviction weight = accumulated CYB. non-fungible — each content hash is unique. transferable — ownership and yield rights are separable from the structural record.

the identity of the actor (neuron) IS the wallet. a neuron card holds tokens, authorizes spends, and bears karma and focus. there is no separate wallet type.

---

## the cyberlink

```
cyberlink(from: TokenId, to: TokenId, token: TokenId, amount: u64, valence: {-1, 0, +1})

from     source token — authorizes the spend (plumb auth on this token's leaf)
to       destination token — receives the moved token
token    what moves — any coin denomination or card id
amount   how much (1 for non-fungible cards)
valence  epistemic prediction: +1 affirm / 0 agnostic / -1 challenge
```

examples:

```
(neuron_alice, particle_A, CYB, 500, +1)    alice stakes conviction on a particle
(particle_A, particle_B, CYB, 100, +1)      particle routes value to related particle
(neuron_alice, neuron_bob, card_X, 1, 0)    card transfer between neurons
(neuron_alice, pool_card, HYDROGEN, 1000, 0) stake into a liquidity pool
(particle_A, particle_B, particle_A, 1, +1) particle asserts relation to another
```

the from/to pair forms the structural edge in the knowledge graph. the token/amount pair is the economic weight. valence is the epistemic layer — the actor's prediction of how the ICBS market on this edge will converge.

a cyberlink with `a = 0, v = 0` is a bare structural assertion. a cyberlink with high amount and `v = +1` is a funded affirmation. both are valid. `a > 0` is required for the link to contribute to cyberank.

---

## plumb: the complete validation system

every cyberlink is a plumb Pay operation. there is no separate lock script layer. plumb provides authorization and logic in one unified structure:

```
auth_hash      WHO     hemera(secret) == leaf.auth_hash    ownership of from token
conservation   WHAT    Σ inputs(d) ≥ Σ outputs(d)          token conservation
hooks          HOW     semcon rules for this link type      semantic validity
```

proving ownership of the `from` token is the only authorization needed. there is no other "who can act" question — token ownership IS the right to link.

plumb operations: Pay, Lock, Update, Mint, Burn. each has a dedicated authority field and hook slot in the config. hooks are composable ZK programs — they extend validation without modifying the core circuit.

see [[plumb]] for the full framework: 10-field leaf model, config schema, hook architecture, nullifier scheme.

---

## skills

a skill is a composable hook that adds behavior to a token. skills install into plumb hook slots. multiple skills compose — their proofs combine independently.

| skill | standard | adds |
|---|---|---|
| knowledge | TSP-2 | content-addressing, cyberank contribution, yield-bearing edges |
| identity | TSP-2 | prog execution, karma tracking, focus accumulation, delegation |
| conviction | TSP-2 | BTS valence, epistemic market participation |
| staking | TSP-1/2 | lock + reward computation, unbonding period |
| liquidity | TSP-1 | AMM swap invariant, TIDE protocol |
| governance | TSP-1/2 | voting power, proposal lifecycle |
| compliance | TSP-1/2 | transfer restrictions, KYC gate |
| royalties | TSP-2 | creator fees on secondary transfers |

particles = cards + knowledge skill. neurons = cards + identity skill. everything else is a card or coin + selected skills. a new standard would require a new conservation law — none exists beyond divisible supply and unique ownership. two standards cover the complete space.

see [[gold standard]] for the full skill library and proof composition architecture.

---

## proof composition

proofs fold. programs do not call.

```
ethereum:   A.method() → calls B → calls C      ordering matters, reentrancy possible
cyber:      proof(A valid) ⊗ proof(B valid) ⊗ proof(C valid) = proof(signal valid)
```

a signal batches multiple cyberlinks governed by different semcons. zheng validates all of them in one folded proof via HyperNova. 1000 cyberlink proofs fold into one constant-size proof. no ordering constraints, no reentrancy surface, no call stack.

a DEX is not a contract. it is a semcon that validates swap cyberlinks. a lending protocol is not a contract. it is a semcon that validates borrow/repay cyberlinks. every application is semcon rules + optional progs. the graph is the state.

---

## state

a cyberlink destroys input boxes and creates output boxes. a box is what persists between two cyberlinks — the token holding. every box has two visibility properties that combine independently:

```
chain visibility    local visibility
────────────────    ────────────────
encrypted           plaintext         ← default: private on chain, public to owner
plaintext           plaintext         ← opt-in: public on chain, public to everyone
```

there is no "private locally" — the owner always sees plaintext. chain privacy means encrypted for the world, not for the owner.

```
BBG (chain):
  private box:  A_live[c] = commit_jali(v, ρ)              encrypted for all
  public box:   BBG_poly(balances, H(owner || token)) = v   plaintext for all

personal BBG (local):
  owner store: (c → (v, ρ))                                 plaintext always, for owner only
```

box privacy is determined by the `to` address type — the cyberlink is unchanged:

```
to = stealth address (genies-derived)  →  private box  (A_live)
to = direct neuron_id or card_id       →  public box   (BBG_poly balances)
```

a single signal mixes both freely. one zheng proof covers the full transition.

spending:
```
private box:  nullifier + ZK proof of ownership
public box:   auth signature + conservation check (no nullifier)
```

---

## progs: autonomous neurons

a prog is installed on a neuron (identity card). it gives the neuron autonomous behavior: listens for graph events, reads public state, creates cyberlinks in response. two runtimes, one interface:

```
os.cyber.hint(filter) → Event              subscribe to graph events
os.cyber.bbg_query(dim, key) → Value       read public aggregate state, O(1) lens opening
os.cyber.cyberlink(from, to, token, a, v)  submit a cyberlink
```

| runtime | nature | when to use |
|---|---|---|
| Rune | dynamic, async, eval, hot-reload | default — most neurons, fast iteration, live scripting |
| Trident | compiled, static, STARK-provable | opt-in — governance bots, high-value automation, auditable logic |

Rune progs reload without stopping the neuron. a Rune prog can eval new code at runtime — the neuron is programmable on the fly. Trident progs are compiled warriors: slower to deploy but their execution is provable via `trident prove`.

```
prog lifecycle (both runtimes):
  event = os.cyber.hint(filter)             ← graph event arrives
  val   = os.cyber.bbg_query(dim, key)      ← read public state
  os.cyber.cyberlink(from, to, token, a, v) ← create response
  plumb validates                           ← identical path as manual cyberlinks
```

progs run outside the proof circuit. the cyberlinks they produce go through normal plumb validation. progs have no elevated permissions — the semcon does not care whether the author is a human, a Rune prog, or a Trident warrior.

`os.cyber.*` is the syscall ABI — a Trident namespace task. Rune implements bindings to the same interface.

---

## state queries

semcon hooks and progs read BBG_poly state without spending anything:

```
bbg_query(neurons,   ν,       karma)     is this neuron reputable?
bbg_query(particles, p,       energy)    how much attention does this particle have?
bbg_query(pools,     pool_id, reserves)  what are the current pool reserves?
```

O(1) polynomial evaluation. one lens opening. ~200 bytes proof. replaces Ethereum view functions, Solana account reads, Cardano reference inputs.

---

## what replaces what

| ethereum | cyber |
|---|---|
| contract deployment | register semcon (cyberlink in the graph) |
| contract storage | BBG_poly dimensions |
| function call | cyberlink with matching semcon |
| msg.sender | `from` token's plumb auth |
| wallet | neuron card (identity IS the wallet) |
| ERC-20 | TSP-1 coin |
| ERC-721 | TSP-2 card |
| view function | bbg_query (polynomial evaluation) |
| approve / transferFrom | not needed — plumb auth is direct |
| reentrancy guard | structurally impossible — proofs, not calls |
| event emission | output cyberlinks (public aggregates in BBG) |
| gas estimation | exact focus cost (per-pattern, deterministic) |
| flashloan | impossible — conservation enforced per signal |
