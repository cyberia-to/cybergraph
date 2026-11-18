---
tags: cybergraph, project, studio
alias: studio, signal studio, the cyberlink composer
crystal-type: plan
crystal-domain: cyber
status: proposed
date: 2026-11-17
owner: master
serves: cybergraph
---

# studio

the place where a neuron makes cyberlinks: composes them, sees what they move, signs the signal. today a mockup at cyberia.my/studio, built inside the land console. the owner, 2026-11-17: the studio is the key UX of creating cyberlinks, a much lower-level thing than a city console, so it moves to [[cybergraph]], and the move comes with one iteration on clarity and on working for real.

| field | value |
|---|---|
| delivery | `cybergraph/studio/`: a trunk + leptos app served on our own front; cyberia.my/studio redirects to it |
| date | — |
| stage | proposed |
| source | cyberia-to/cyberia-my `src/signal.rs`, `src/signal_pages.rs`, `src/world.rs` (StudioPage) |

## what moves, what stays

moves to cybergraph, because it is the book's own vocabulary: the neuron (mudra domain key), particles (hemera), the cyberlink, the draft signal and its commit, the signal list and view, the intent. that is `signal.rs` (no dependency on the rest of cyberia.my) and the word, link and signal pages of `signal_pages.rs`.

stays in cyberia.my as a dialect over the book: cards, coins, motifs, schedules, views, genetics (`studio.rs`, `erp.rs`). cyberia.my keeps them and calls the studio for linking.

## the iteration: the studio speaks the spec

the mockup links word → rel → word with a float weight. the spec ([[cyberlink]]) links two particles and moves a box with a valence. the studio becomes that, and only that:

1. one screen, the composer: from, to (search a particle by name or paste one, or drop a file — its particle is computed in the page), coin and amount (the box), valence (−1 · 0 · +1). add → the link joins the draft.
2. the draft signal beside it: the links, their total box, the focus each costs ([[validate]] focus sufficiency), the impulse preview when tru answers; remove a link; commit = sign with the neuron.
3. after commit: the signal page with its [[proof]] slot (the σ the neuron produced) and its [[order]] slot (what the chain answers: height, prev, no equivocation), re-verify in the page.
4. many subjects — waits on the spec, not built here: [[signal]] carries exactly one signing subject ν and one proof σ, and [[validate]] checks focus sufficiency and the signature against that same ν. a co-signed draft (whose links are whose, every signature in, one atomic commit) needs an extension to [[signal]] that no page in this tree proposes. this step is named so it is not forgotten; it starts when that extension is proposed and accepted.
5. an intent ([[intent]]): declare a scope with its sealer (me or anyone) and offer; the intents list shows which are open to fill.
6. gone: the seven-button bar, the ladder strip, the KPI tiles, the TSP labels. the neuron is one line in the header, not three cards.

works for real means: where the chain answers (pussy), commit submits; where it does not, the signal stays local and says so on its face, never pretends.

## acceptance

- a person who has not seen it makes a link and commits a signal in under a minute, walked by a page check (cdp, desktop and phone);
- the stored signal re-verifies in the page (signature over the canonical body), and on pussy the submitted signal is found by its id;
- cyberia.my/studio and its old /world/* links land on the new studio; cyberia.my builds without the moved files.

## order

1. carve: `signal.rs` and the three page families into `cybergraph/studio/` unchanged, building and serving; cyberia.my imports nothing from it yet.
2. the iteration above, one change per pull request: composer, draft, commit view, intent. step 4 is not in this order: it waits on a proposal to extend [[signal]].
3. cyberia.my drops the moved files and redirects.

related: [[cybergraph/specs/signal|signal]] · [[cybergraph/specs/cyberlink|cyberlink]] · [[cyberia/projects/cyberia-my|cyberia.my]]
