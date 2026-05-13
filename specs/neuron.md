# neuron

the one who links. agent with stake, identity, and [[will]] to shape the [[cybergraph]]

$$\nu \in N, \quad \text{id}(\nu) = \text{Hemera}(\text{pk}_\nu)$$

human, AI, sensor, or program — anything that can prove a [[signature]] or act within [[consensus]]. identity is the [[Hemera]] hash of a [[public key]]. no registration, no authority.

## definition

a neuron is a tuple $(\text{id}, \text{pk}, S, F, K)$:

| field | name | type | semantics |
|-------|------|------|-----------|
| $\text{id}$ | identity | $P$ | Hemera hash of public key — a [[particle]] |
| $\text{pk}$ | public key | $\mathbb{F}_p^*$ | verifies [[signal]] signatures |
| $S$ | signals | $\text{Signal}^*$ | ordered history of committed [[signal]]s |
| $F$ | focus | $\mathbb{R}_+$ | current balance of [[focus]] tokens |
| $K$ | karma | $\mathbb{R}$ | accumulated epistemic reward from correct [[valence]] predictions |

## axioms

**N1 (identity uniqueness):** no two neurons share an identity. $\text{id}(\nu_1) = \text{id}(\nu_2) \Rightarrow \nu_1 = \nu_2$.

**N2 (costly participation):** every [[cyberlink]] in a [[signal]] consumes focus. $\Delta F \geq |\vec\ell| \cdot c_{\min}$ where $c_{\min}$ is the minimum focus cost per link.

**N3 (stake-bounded links):** a neuron cannot commit [[cyberlinks]] with total [[amount]] exceeding available stake. conviction UTXOs are created and locked.

**N4 (karma accountability):** karma accrues from [[valence]] predictions that converge with the collective [[focus distribution]]. no external karma injection.

## role in the graph

neurons are the source of all [[cyberlinks]]. without neurons there is no [[cybergraph]] — the graph is the aggregate of all neuron signals over all [[time]].

the [[tri-kernel]] focus distribution $\phi^*$ reflects collective neuron judgment: high-focus [[particles]] are those many neurons have linked to with high [[stake]].

## see also

- [[signal]] — the atomic act a neuron performs
- [[cyberlink]] — what a signal decomposes into
- [[focus]] — the resource neurons spend to link
- [[karma]] — the reward neurons earn for accurate predictions
