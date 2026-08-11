# signal

a bundle of [[cyberlinks]] a [[neuron]] commits in a single [[step]] — the atomic broadcast unit in [[cyber]]. each link in the signal consumes [[focus]], making every statement a [[costly signal]]

## structure

$$s \;=\; (\nu,\; \mathit{net},\; \vec\ell,\; \Delta\phi^*,\; \sigma,\; h_0,\; h_1)$$

| field | name | type | semantics |
|-------|------|------|-----------|
| $\nu$ | [[subject]] | $N$ | signing [[neuron]] |
| $\mathit{net}$ | network | $\text{CardId}$ | destination [[network]] this signal is delivered to — a [[card]] id. defaults to the sender's private network |
| $\vec\ell$ | links | $L^+$ | one or more [[cyberlinks]] — each a 5-tuple $(p, q, \tau, a, v)$ |
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

- correctness of each [[cyberlink]] in $\vec\ell$ (valid signatures, valid particle references)
- validity of all conviction box movements (each link's $(\tau, a)$ spend is backed by an unspent output)
- correctness of the [[cyber/impulse]] $\Delta\phi^*$ (the [[tri-kernel]] computation against $\text{bbg\_root}$ from the current header)

one proof for everything. proving $n$ links together costs less than $n$ separate proofs because shared neighborhood state and box set are proved once. any verifier runs `decide(σ)` in $O(\log n)$ without recomputing anything

## two effects

validation of a signal produces two outcomes:

1. each link in $\vec\ell$ enters $L$ — conviction boxes are created for each [[cyberlink]]
2. if $\|\Delta\phi^*\| > 0$ and $\sigma$ is valid, the [[neuron]] self-mints [[$CYB]] proportional to the proven shift — a reward box is created for $\nu$

the conviction boxes ([[tokens]] spent into links) and the reward box ([[tokens]] minted for contribution) are separate token movements within one atomic signal. see [[cyber/rewards]] for the full reward specification

## minting conservation

total minting per epoch is bounded by the actual global $\Delta\phi^*$, verifiable from consecutive headers. if the sum of individual claims exceeds the actual shift (overlapping neighborhoods), all claims are scaled proportionally
