---
tags: cyber, soft3, cybergraph, cli
crystal-type: spec
crystal-domain: cyber
status: draft
date: 2026-06-11
---
# cybergraph cli

a command-line console for the cyberlink processor: declare intents, seal and link signals, query the graph with [[inf]], and inspect the per-neuron chains. `cybergraph`'s verb is **link** — the CLI is the processor's console, one layer above the [[bbg]] state store (`bbg` CLI).

## scope — the processor, not the store

two consoles, two layers:

- **`bbg`** ([[bbg/specs/cli|bbg cli]]) is the raw state store — `apply` is `insert`, `prove` opens a key. it sees dimensions and roots.
- **`cybergraph`** is the processor on top — it runs the full lifecycle (validate → order → apply), keeps the per-neuron [[signal]] chains, and answers [[inf]] queries. `cybergraph link` is `bbg apply` plus chain ordering; `cybergraph query` is datalog, where `bbg prove` is a single opening.

use `bbg` to debug state; use `cybergraph` to drive and query the graph. it does not decide *what* to link — that is the [[soma]] runtime above it (see the [whitepaper](../docs/README.md)).

## store — signal-first, the canonical envelope log

cybergraph state is a fold of the signal log (see [[bbg/specs/storage|storage]]), and the log is **[[tape]] frames** — here the *envelope* [[signal]] (neuron, links, prev, step, network), the canonical wire form [[foculus]] mints and [[radio]] carries:

```
cybergraph --store <dir> <command>
  <dir>/log        append-only tape stream — envelope signal / intent / finalize frames
  <dir>/checkpoint latest root + height cache
```

`open` replays the log through the processor — each signal frame goes through `link` (chain_append → `bbg.insert`), rebuilding both the chains and the state; intent frames replay through `intend`; finalize markers advance the block. this reuses [[foculus]]'s frame codec (`decode_signals` / `encode_signal_frame`) — the envelope shape, distinct from the `bbg` CLI's lower state-application log. JSON appears only as a `--json` output view, never the log.

## commands

### lifecycle — the write verbs

```
cybergraph intend --neuron H --scope H [--h0 N]     declare an unsealed intent → prints its key
cybergraph link   --neuron H --from H --to H        atomic one-shot submit
                  [--token H --amount N --valence V]
cybergraph seal   --intent KEY --neuron H --from H --to H [...]   seal a declared intent
```

`link` and `seal` build a signal from the cyberlink flags; the CLI **auto-fills** the chain fields — `step` = the neuron's chain length, `prev` = its head hash, `network` = the neuron's private network — so the operator supplies only content. the signal is ordered (equivocation / step / prev) then applied. Release 0: no [[zheng]] proof, `box_moves` empty; the seal binding (`σ ⊢ scope`) is not yet enforced (see the whitepaper).

### read

```
cybergraph query '<inf script>'    run an inf datalog query over the graph; print rows
cybergraph chain <neuron>          show a neuron's signal chain — step, prev, link count
cybergraph intents                 list declared (unsealed) intents
```

`query` is the headline: it runs the [[inf]] engine (`parse → plan → eval`) over the relations cybergraph exposes ([[query|the relation schema]]) — `particles`, `neurons`, `axons`, `focus`, `karma`, `signals`. examples:

```datalog
?[p, s] := focus{particle: p, score: s} :sort -s :limit 10       # top particles by focus
?[to]   := axons{from: 0x02…, to}                                # what 0x02 links to
?[n, k] := karma{neuron: n, k}, gt(k, 1000)                      # high-karma neurons
```

### inspect

```
cybergraph root      BBG_root (hex)
cybergraph stats     graph statistics (node_count, relation_sizes, max_degree, diameter_bound)
```

## output & exit codes

`--json` on every command for scripting; exit `0` ok, `1` not-found, `2` rejected (equivocation / double-spend), `3` bad input.

## implementable now vs deferred

buildable against today's API: `intend`, `link`, `seal`, `query`, `chain`, `intents`, `root`, `stats`. it reuses [[foculus]]'s tape codec (envelope frames) and the inf evaluator cybergraph already wires (`inf-eval` over `BbgSource`). deps: cybergraph + foculus (frames, `default-features = false` — no network stack) + clap.

deferred:
- **`watch <filter>`** — subscribe is in-process; a one-shot CLI cannot stream future events. this needs a `serve`/daemon mode, or a log-tail that replays and follows. left for a later release.
- **seal binding enforcement** — `seal` accepts an unproven signal today; `σ ⊢ scope_hash` lands with the proof path.
- **provable query** — `query` returns plain rows; Lens-opening-backed proofs over the root arrive with lens serde (soft3 blocker #1).

## crate + conventions

- new binary crate `cybergraph/cli` (sibling of the lib), bin `cybergraph`; the library stays untouched.
- args via `clap`; installed by `scripts/install-bins.sh` alongside `bbg` / `cy` / `mudra`; built with `RUSTC_BOOTSTRAP=1`.
- the chain-field auto-fill (`step`/`prev`/`network`) is the CLI's ergonomic layer; the library keeps requiring an explicit, correct signal.

## open questions

1. binary name — `cybergraph` (explicit, matches `bbg`) vs a short `cg`. proposal: `cybergraph`, add a `cg` symlink in install-bins.
2. `watch`/`serve` — is a long-running cybergraph daemon in scope for the CLI, or does that belong to `cy` / soma? proposal: keep the CLI one-shot; a daemon is soma's concern.
3. authoring beyond one cyberlink — `link` builds a single-link signal; multi-link signals and box moves need either repeated flags or a frame-file input (`link --frame <file>`). proposal: one-link flags now, `--frame` import later, sharing the `bbg` CLI's frame reader.
