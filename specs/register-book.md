---
tags: cyber, cybergraph, docs
alias: register book, book registration
---
# register book

`Cybergraph::register_book(neuron, book_token)` is the naming half of
[[cyber/launch|launch]] property 30: any [[neuron]] roots a home
[[cyber/research/oikos|oikos]] book, and registration links the book's
name into the graph, not only into the ledger.

`tok::BookRegistry::root` (plumb) derives a book's token id
deterministically from the neuron and claims the one-home-book-per-neuron
invariant ledger-side. `register_book` publishes that claim as a
[[cyberlink]]: `from` the neuron's own particle identity, `to` the book
token, staked with `amount = 1` and `valence = 1` — an affirmed naming
edge, the same shape `link` already gives any signal, so a reader of the
graph sees a neuron's book the same way it sees any other link, without a
side channel into the ledger.

the caller does not track signal-chain step or prev hash; `register_book`
reads the neuron's chain position itself, the same computation
`SignalChain::append` validates against.

out of scope here, both open elsewhere:

- re-pointing the link at the book's current state root as it settles —
  needs [[foculus]] domain finality (property 31)
- rejecting a second registration for the same neuron at the graph layer
  — the one-home-book invariant is enforced ledger-side by
  `tok::BookRegistry`; cybergraph does not read tok state, so two
  `register_book` calls for the same neuron both land as links today
