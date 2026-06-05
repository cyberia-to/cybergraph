# intent

an intent is an unsealed [[signal]] — a neuron's public declaration of a directed action before it is finalized and proven

## structure

$$i \;=\; (\nu,\; h_0,\; \Sigma,\; \pi_\text{id})$$

| field | name | type | semantics |
|-------|------|------|-----------|
| $\nu$ | [[subject]] | $N$ | declaring [[neuron]] |
| $h_0$ | [[inception]] | $\mathbb{Z}_{\geq 0}$ | block height at which the intent was declared |
| $\Sigma$ | [[scope]] | $S$ | structured description of the intended action: target particle, predicate, deadline, constraints |
| $\pi_\text{id}$ | identity proof | $\text{Sig}$ | [[neuron]]'s signature over $(\nu \;\|\; h_0 \;\|\; \Sigma)$ |

an intent carries an identity proof (the neuron's signature over the declared scope) but no content STARK. the STARK is produced at sealing, when the full computation is complete

## lifecycle

```
declared
   │
   ├──→ sealed     — neuron finalizes: adds sealing height + zheng proof → [[signal]]
   │
   ├──→ abandoned  — intent never sealed; record persists in the graph
   │
   └──→ cascaded   — intent triggers coordinated sub-signals → [[cascade]]
```

### sealed

a sealed intent is a [[signal]]. the neuron assigns a sealing height and produces a [[zheng]] proof covering all [[cyberlinks]] and the [[cyber/impulse]]. the intent's scope becomes the signal's link set

### abandoned

if the neuron never seals, the intent record stays in the [[cybergraph]] at inception height. other neurons can observe that the action was declared but not followed through. abandonment is on record — there is no silent cancel

### cascaded

a neuron may declare an intent with scope that invites participation: subscribers observe it, self-organize into coordinated sub-signals, and the lead neuron seals a parent signal with a recursive [[zheng]] proof over the entire cascade. see [[cascade]] for the full multiparty protocol

## relationship to signal

signal and intent share the same substrate. signal is what intent becomes:

```
intent   = (neuron, inception, scope, identity_proof)
signal   = (neuron, links, impulse, proof, inception, sealing)
```

the difference is completeness. an intent asserts what will happen; a signal proves it happened

## identity proof

$\pi_\text{id} = \text{Sign}_\nu(\nu \;\|\; h_0 \;\|\; \text{hash}(\Sigma))$

this binds the neuron to the declared scope at the stated height. it is not a content proof — it does not cover computation correctness — but it is unforgeable and non-repudiable. any observer can verify that $\nu$ declared $\Sigma$ at $h_0$

## scope

a scope describes the intended action in structured terms. minimum fields:

| field | semantics |
|-------|-----------|
| target | one or more [[particles]] this intent addresses |
| predicate | the declared relationship or transformation |
| deadline | latest acceptable sealing height (optional) |
| constraints | any additional conditions the neuron self-imposes |

scope encoding is dialect-specific; the intent mechanism is dialect-agnostic

## discovery

intents at a given height are readable from the [[cybergraph]] by any [[neuron]]. this enables coordination without a central scheduler: subscribers discover pending intents, self-assign, and begin producing their sub-signals. the lead neuron observes incoming sub-signals and seals when ready
