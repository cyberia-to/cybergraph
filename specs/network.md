# network

a network is a shard of the [[cybergraph]] — a self-contained graph with its own
state tree, its own local [[focus]], and its own validators. a [[signal]] is
delivered to exactly one network, named by the signal's $\mathit{net}$ field.

this is the explicit, discrete realization of the economic dimension of
[[cyber/hierarchy]]'s 4D fold. the semantic, social, and geographic dimensions
are computed by the [[tri-kernel]]; the network dimension is a label the
[[neuron]] chooses.

## a network is a card

a network is identified by a `TokenId` whose token type is a [[card]] — unique,
movable. the [[card]] model is exact for a network:

- unique — network A is never network B; networks are not interchangeable
- owned — the `cards` [[dimension]] records `CardRecord { owner, particle }`; the
  owner is the network's founder / governance
- transferable — a network can be handed over or sold, as any card transfers

the existing `cards` dimension *is* the network registry. "is this a real
network?" reduces to "does this card exist?" — no new dimension, no new registry.

a [[coin]] would be wrong: coins are fungible denominations of conviction, and a
network is a singular identity, not a denomination. the network's identity (one
card) is distinct from the denominations circulating inside it (any coins).

## the default network is the neuron's private network

there is no privileged global default. each [[neuron]] has its own private
network, and that is where its signals land unless it says otherwise.

$$\text{private\_network}(\nu) = H(\texttt{"network:"} \,\|\, \nu)$$

the private network card is derived deterministically from the neuron — it needs
no minting step and is owned by $\nu$. on the wire, the zero card
(`SELF_NETWORK`) is the sentinel for "my private network"; cybergraph resolves it
to $\text{private\_network}(\nu)$ when the signal is committed.

joining a shared network is opt-in: the neuron sets $\mathit{net}$ to that
network's card id. this matches cyber's local-first, private-by-default stance —
sharing is a deliberate act, not the ambient condition.

## routing

each cybergraph node serves a network. on commit it resolves the destination
(sentinel → private network) and, when serving a specific network, accepts the
signal only if the resolved network matches; otherwise it rejects with
`WrongNetwork`. a node with no fixed network (local-first dev) accepts any and
records whatever network each signal names.

the resolved network is recorded in the `signals` [[dimension]]
(`SignalRecord.network`) and committed into `BBG_root`, so the routing decision
is authenticated, provable history.

## one network, one state tree

[[bbg]] state is network-local: one bbg instance holds one network's state — one
root, one local [[focus]], per [[cyber/hierarchy]]'s "each shard computes local
focus independently". a [[cyberlink]] whose endpoints live in two networks is a
cross-network link, relayed by [[zheng]] [[proof]] — see [[3c]]. intra-network
delivery is assumed by the signal path.

see [[signal]] for the $\mathit{net}$ field. see [[card]] for the token type. see
[[cyber/hierarchy]] for the full 4D scaling model. see [[3c]] for cross-network
relay.
