---
tags: cyber, cybergraph, bostrom
crystal-type: process
crystal-domain: cyber
alias: genesis replay, genesis bijection
---
# genesis replay

property 14 of [[cyber/launch|the launch]]: genesis is bijective with the [[bostrom]] burial snapshot. this is the contract a genesis pipeline must satisfy before either chain roots at 2026-11-05.

## input

the snapshot at [snapshot.bostrom.network](https://snapshot.bostrom.network), pinned by CID and sha256 in `bostrom/snapshot/manifest.json`: `balances.csv` (61,675 accounts, every denom), `cyberlinks_indexed.csv.gz` (2,949,732 links, bit-exact to the on-chain counter), `passports.jsonl` (47,837 moon passports), `delegations.csv`, `pubkeys.csv`, `pools.json`, `state_export.json.gz`. space-pussy's own extraction carries 29,112 links. every count here is the manifest's declared count, not a re-derived one; replay verifies against it, never substitutes for it.

## replay

```
replay(snapshot) → genesis state
  for each balance row: one box per (account, denom) in A
  for each cyberlink row: one link + box + valence in the cyberlink dimension
  for each passport row: one naming cyberlink
  for each delegation row: one stake entry
  commit each dimension table (bbg state.md) ; StateCertificate.root() is genesis's root
```

one replay pass, no second pass, no merge step: cybergraph's [[apply]] verb applied once per snapshot row, in the row's file order, into the dimension tables [[bbg]] already defines. the [[particle]] identity used for cyberlink endpoints is whichever definition property 19 freezes; this contract does not depend on which one wins.

## bijective, zero loss

replay is a bijection: every snapshot row produces exactly one genesis entry, and every genesis entry traces to exactly one snapshot row. concretely, for every dataset:

```
count(genesis entries from dataset D) == count(rows in D per manifest.json)
```

no row is deduplicated, coalesced or silently dropped; no genesis entry appears with no snapshot origin. the 2.37% of bostrom particles with a CID and no bytes (decision open on [[cyber/launch|launch]]) still replay to one particle identity each — a black hole, addressed but empty — so they count toward the bijection instead of vanishing from it.

## verification

before a genesis candidate is accepted:

1. recompute row counts from the decompressed source files; diff against the manifest's declared counts. a mismatch there means the snapshot itself changed and replay has not started.
2. recompute genesis entry counts per dataset from the replayed state; diff against step 1. any gap is loss, any excess is a duplicate — either blocks genesis.
3. record the two counts and the resulting `StateCertificate.root()` in `bostrom/audit/genesis-replay.md`, dated, against the exact manifest CIDs replayed.

closed only when steps 1–3 run clean on both bostrom and space-pussy's own snapshot; property 14 on [[cyber/launch|launch]] carries the date and the root.

## open until property 19 and the genesis decisions close

replay cannot run for real until: the particle definition (19) is frozen, the identity of the 2.37% missing-bytes particles is decided, and the bond formula for milliampere/millivolt is fixed — all logged as open decisions on [[cyber/launch|launch]]. this spec is the contract those decisions plug into; it does not itself resolve them.

see [[apply]] for the verb this reduces to · [[particle]] for the identity in play · [[bostrom]] for the snapshot · [[cyber/launch|launch]] for property 14.
