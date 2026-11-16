# signal

a bundle of [[cyberlinks]] one or more [[neurons]] commit in a single [[step]] — the atomic broadcast unit in [[cyber]]. each link in the signal consumes [[focus]], making every statement a [[costly signal]]. a signal is multiparty by design: every link names its author, the subjects are the set of authors, and the signal lands for all of them or for none

## structure

$$s \;=\; (\vec\nu,\; \mathit{net},\; \vec\ell,\; \Delta\phi^*,\; \sigma,\; h_0,\; h_1,\; i)$$

| field | name | type | semantics |
|-------|------|------|-----------|
| $\vec\nu$ | [[subject|subjects]] | $N^+$ | the signing [[neurons]] — the set of authors of the links in $\vec\ell$; a signal of one neuron is the common case, a signal of many is one atomic act of many |
| $i$ | [[intent]] | $\text{Particle}^?$ | the intent this signal seals, when it seals one (see [[intent]], the demand face); absent for a signal that fulfils no declared intent |
| $\mathit{net}$ | network | $\text{CardId}$ | destination [[network]] this signal is delivered to — a [[card]] id. defaults to the sender's private network |
| $\vec\ell$ | links | $L^+$ | one or more [[cyberlinks]] — each a 5-tuple $(p, q, \tau, a, v)$ carried under its author $\nu_k \in \vec\nu$ |
| $\Delta\phi^*$ | [[cyber/impulse]] | $(P \times \mathbb{F}_p)^*$ | sparse [[focus]] update: how the batch of links shifts $\phi^*$ |
| $\sigma$ | proof | $\Pi$ | [[zheng]] proof covering the [[cyber/impulse]], all conviction box movements, and [[cyberlink]] validity against the current [[BBG]] root |
| $h_0$ | [[inception]] | $\mathbb{Z}_{\geq 0}$ | block height when the [[neuron]] declared the signal |
| $h_1$ | [[sealing]] | $\mathbb{Z}_{\geq 0}$ | block height when the signal was finalized and proven |

a signal has duration: $h_0$ marks when the [[neuron]] committed to the action; $h_1$ marks when the proof was produced and the signal became final. the gap $h_1 - h_0$ is the signal's span

before $h_1$ is assigned and $\sigma$ produced, the signal exists as an [[intent]]: declared, identity-proven, but not yet sealed

the signal separates what a [[neuron]] asserts (the [[cyberlinks]]) from what the assertion computes (the [[cyber/impulse]]). see [[cyber/impulse]] for how $\Delta\phi^*$ is computed

## destination network

$\mathit{net}$ names the [[network]] the signal is delivered to. a network is identified by a [[card]] id — unique, owned, transferable — so one network is never interchangeable with another. one signal targets one network; all its [[cyberlinks]] land in that network's state.

the default is the sender's private network: the zero card (`SELF_NETWORK`) resolves to $\text{private\_network}(\nu) = H(\texttt{"network:"} \,\|\, \nu)$ at the cybergraph boundary. every [[neuron]] writes to its own sovereign private network by default; joining a shared network is the explicit act of setting $\mathit{net}$ to that network's card id. `$CYB` is a [[coin]], so it is never a network — there is no privileged global default.

$\mathit{net}$ is bound into the signal hash, so the destination is tamper-evident in relay. links whose $p$/$q$ live in another network are cross-network — see [[3c]] for [[zheng]] proof-relay; intra-network delivery is assumed here. see [[network]] for the full model.

## zheng proof coverage

$\sigma$ is a single [[zheng]] proof that covers the entire signal atomically:

- correctness of each [[cyberlink]] in $\vec\ell$ (a valid signature from its author $\nu_k$, valid particle references)
- validity of all conviction box movements (each link's $(\tau, a)$ spend is backed by an unspent output owned by that link's author)
- conservation over the union: $\sum$ box inputs of all subjects $= \sum$ outputs $+$ fee, one equation for the whole signal
- correctness of the [[cyber/impulse]] $\Delta\phi^*$ (the [[tri-kernel]] computation against $\text{bbg\_root}$ from the current header)
- when $i$ is set: that the execution σ proves is the one the intent's scope declared, $\sigma \vdash \text{scope\_hash}(i)$

one proof for everything. proving $n$ links together costs less than $n$ separate proofs because shared neighborhood state and box set are proved once. any verifier runs `decide(σ)` in $O(\log n)$ without recomputing anything

## many subjects, one act

a signal with $|\vec\nu| > 1$ is how neurons act together: a trade, a settlement, an intent and its fulfilment, a joint claim. the rules are the single-subject rules applied per author, plus one shared id:

- authorization and ownership are checked per link against that link's author, never against one signer for all
- the signal extends every subject's chain: one `prev` per subject, and the signal counts as that subject's step for equivocation ([[order]])
- conservation is one equation over every subject's boxes; what one subject gives another receives inside the same signal
- the signal lands for every subject or for none — a failed leg rejects the whole signal, no partial entry into $L$

the signal id is shared by every leg, so a subject can show its receipt for the joint act without the others' private state. see [[receipt]] for the object a subject keeps

## two effects

validation of a signal produces two outcomes:

1. each link in $\vec\ell$ enters $L$ — conviction boxes are created for each [[cyberlink]]
2. if $\|\Delta\phi^*\| > 0$ and $\sigma$ is valid, each subject self-mints [[$CYB]] proportional to the shift its own links proved — a reward box is created per subject

the conviction boxes ([[tokens]] spent into links) and the reward box ([[tokens]] minted for contribution) are separate token movements within one atomic signal. see [[cyber/rewards]] for the full reward specification

## minting conservation

total minting per epoch is bounded by the actual global $\Delta\phi^*$, verifiable from consecutive headers. if the sum of individual claims exceeds the actual shift (overlapping neighborhoods), all claims are scaled proportionally
