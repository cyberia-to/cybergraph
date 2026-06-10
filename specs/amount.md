# amount

the fourth field of a [[cyberlink]]: $a \in \mathbb{R}_+$. the magnitude of the conviction [[box]] — how many units of the staked [[token]] the assertion carries.

$$a(\ell) \in \mathbb{R}_+$$

amount is what separates a structural claim from an economic commitment. a link with $a = 0$ is a bare assertion — it exists in the graph but holds no box. a link with $a > 0$ holds a box of `a` units: the [[neuron]] has moved real value into a conviction position bound to the claim.

amount alone is a context-free number. paired with a denomination it becomes a [[box]] — `(token, a)` — and the box is where all conviction mechanics live: lifecycle (create / transfer / withdraw / spend), adjacency weight, yield, costly signaling. see [[box]].

| amount | meaning |
|--------|---------|
| $a = 0$ | bare assertion. no box |
| $a$ small | low-conviction box |
| $a$ large | high-conviction box |
| $a$ → burn | permanent conviction ([[eternal cyberlinks]]) |

see [[box]] for conviction mechanics · [[token]] for the denomination · [[cyberlink]] for the full record · [[will]] for the budget that constrains allocation.
