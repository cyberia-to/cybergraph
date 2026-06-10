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

an intent carries an identity proof (the neuron's signature over the declared scope) but no content STARK. the STARK is produced at sealing — it proves the neuron actually *ran* the scope. the intent commits to what will be computed; the signal proves it was.

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

a sealed intent is a [[signal]]: the neuron ran the scope, produced a [[zheng]] proof $\sigma$ over the execution (its [[cyberlinks]] and [[cyber/impulse]]), and committed it. seal is accepted only if $\sigma$ attests the declared scope — see completion below.

### abandoned

if the neuron never seals, the intent record stays in the [[cybergraph]] at inception height. other neurons can observe that the action was declared but not followed through. abandonment is on record — there is no silent cancel

### cascaded

a neuron may declare an intent with scope that invites participation: subscribers observe it, self-organize into coordinated sub-signals, and the lead neuron seals a parent signal with a recursive [[zheng]] proof over the entire cascade. see [[cascade]] for the full multiparty protocol

## completion — how an intent becomes a complete signal

an intent is a deferred computation. its [[scope]] is not a passive description — it is an executable specification of what to compute. the intent commits to it; the signal proves it was run. completion fills the three things an intent lacks — the [[cyberlinks]], the [[impulse]] $\Delta\phi^*$, and the proof $\sigma$ — by executing the scope and proving the execution:

```
declare    intend(scope)         scope_hash committed + signed — nothing run yet
execute    run the scope on nox  reading committed state via cybergraph query;
                                  iterate until it converges → cyberlinks + Δφ* (+ a trace)
prove      zheng proves the trace → σ
seal       seal(intent, signal)  commit the proven run
```

the orchestration — decide, collect, run, judge what is left, iterate — is the [[soma]] control loop, not [[cybergraph]]'s. cybergraph stores the intent, serves the reads ([[query]]), and gates the seal. [[nox]] executes; [[zheng]] proves; soma drives.

### the seal binding

seal is valid only if $\sigma$ proves an execution of the intent's declared scope:

$$\text{seal}(i, s)\ \text{accepted} \iff \sigma(s) \vdash \text{scope\_hash}(i)$$

this binding is what makes intent and signal **one transaction** rather than two unrelated records. the intent is a promise (committed scope); the signal is its proven fulfillment. a sealed signal is cryptographic proof the neuron did exactly what it declared — the alignment property, enforced at the boundary. without the binding, any signal could seal any intent.

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
