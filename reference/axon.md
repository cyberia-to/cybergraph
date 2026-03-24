# axon

the bundle of all [[cyberlinks]] between two [[particles]] across all [[neurons]] and [[time]]. if a [[cyberlink]] is a synapse, an axon is the nerve fiber. weight sums contributions from many [[neurons]], reflecting collective judgment

## definition

axons emerge from the [[cybergraph]]; they are never created directly. the natural unit for the [[tri-kernel]]: [[diffusion]] flows along them, [[springs]] constrain them, [[heat]] smooths across them

## weight computation

the axon weight for the directed pair $(p, q)$ is the aggregate of all cyberlinks from $p$ to $q$:

$$w_{\text{axon}}(p, q) = \sum_{\substack{\ell \in L \\ \operatorname{src}(\ell)=p,\; \operatorname{tgt}(\ell)=q}} r(\tau(\ell)) \cdot a(\ell)$$

this feeds directly into the adjacency operator $A_{pq}$

## homoiconicity — axon as particle

every axon is a [[particle]]: $H(\text{from}, \text{to}) \in P$. the hash of the directed edge induces a content-addressed node in the [[cybergraph]]. this means axons have [[cyberank]], receive [[focus]], carry [[value]], and can themselves be targets of [[cyberlinks]]. the graph ranks its own structure. this is axiom A6

## meta-annotation

you can [[cyberlink]] TO an axon — meta-annotating a relationship. you can stake on axon-particles — betting on the importance of a connection. [[focus]] flows through axon-particles alongside content-particles

see [[cybergraph]] axiom A6 for the formal specification
