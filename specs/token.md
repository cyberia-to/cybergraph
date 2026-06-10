# token

the third field of a [[cyberlink]]: $\tau \in \mathcal{T}$. the denomination in which conviction is expressed — paired with [[amount]] `a` it forms the [[box]] a cyberlink moves.

a [[cyberlink]] without a denomination is a structural assertion with no economic weight. the token field selects which scarce resource the [[neuron]] stakes against the claim.

## definition

$\mathcal{T}$ is the set of valid token denominations. each has a rate function $r: \mathcal{T} \to \mathbb{R}_+$ that converts to a common scale for the adjacency operator:

$$A_{pq} = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

$r(\tau)$ normalizes across denominations so conviction in different tokens is commensurable in the [[tri-kernel]].

## multi-denomination staking

the same structural link $(p, q)$ can carry conviction in multiple denominations — two signals staking CYB and H on the same edge create two [[box|boxes]], two [[card|cards]], two yield streams. the adjacency weight $A_{pq}$ sums both via $r(\tau)$.

## token model lives in the value layer

token *types* (coin, card, score, badge), the TSP standards, protocol tokens (CYB, H), bonding curves, and skills are the value layer — specified in [[tok]] (TSP-1 coins, TSP-2 cards) and [[plumb]] (the five operations). cybergraph uses tokens as cyberlink denominations; it does not define the token programming model.

see [[box]] for the conviction unit · [[amount]] for the magnitude · [[plumb]] for the value operations · [[cybernomics]] for the economic theory.
