# token

the third field of a [[cyberlink]]: $\tau \in \mathcal{T}$. the denomination in which conviction is expressed

a [[cyberlink]] without a [[token]] denomination is a structural assertion with no economic weight. the token field is what makes a link a bet — it selects which scarce resource the [[neuron]] stakes against the claim

## definition

$\mathcal{T}$ is the set of valid token denominations in the [[cybergraph]]. each denomination has a rate function $r: \mathcal{T} \to \mathbb{R}_+$ that converts to a common scale for the adjacency operator:

$$A_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

the rate function $r(\tau)$ normalizes across denominations so that conviction in different [[tokens]] is commensurable in the [[tri-kernel]]

## protocol tokens

| token | ticker | role |
|-------|--------|------|
| CYB | [[$CYB]] | root consensus token. staked for security, locked for [[will]], burned for permanent φ*-weight, spent as fees |
| HYDROGEN | [[$H]] | liquidity engine. paired with [[$CYB]] via [[bonding curves]]. provides the external price signal |

## the four token types

from [[token theory]] — two axes (fungible/unique × movable/immovable):

| type | properties | role |
|------|-----------|------|
| [[coin]] | fungible, movable | [[$CYB]], [[$H]] — stake, fees, economic commitment |
| [[card]] | unique, movable | provenance binding to a [[particle]]. every [[cyberlink]] is a card |
| [[score]] | fungible, immovable | [[karma]], [[will]] — reputation and capacity |
| [[badge]] | unique, immovable | achievements, proofs |

## multi-denomination staking

the same structural link $(p, q)$ can carry conviction in multiple denominations. a [[neuron]] who creates:

- $(\nu, p, q, \text{CYB}, 100, +1, t_1)$
- $(\nu, p, q, \text{H}, 50, +1, t_2)$

expresses conviction in two currencies. the adjacency weight $A_{pq}$ sums both via $r(\tau)$. each creates a separate UTXO, a separate card, a separate yield stream

## token as costly signal

the choice of denomination carries information. staking [[$CYB]] (scarce, governance-weighted) signals higher conviction than staking a liquid pair token. the market reads denomination choice as a signal of commitment depth

## permanent weight tokens

burning [[$CYB]] creates [[eternal particles]] (permanent φ*-weight on a [[particle]]) or [[eternal cyberlinks]] (permanent edge weight). burned tokens exit circulation permanently — the strongest possible conviction signal

see [[cyberlink]] for the full 5-tuple. see [[amount]] for the stake quantity. see [[cybernomics]] for the economic theory
