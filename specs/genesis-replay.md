---
tags: cyber, cybergraph, bostrom
crystal-type: process
crystal-domain: cyber
alias: genesis replay, genesis bijection
---
# genesis replay

property 14 of [[cyber/launch|the launch]]: genesis is bijective with the burial snapshots of [[bostrom]] and space-pussy. this page is the bijection contract. the derivation rules are normative in soft3 `specs/chain.md` §4, the pipeline that implements them is soft3 `specs/genesis.md` (`soft3 genesis build | verify`), and observed results live in soft3 `audit/genesis-*.md`.

## input

each burial's `manifest.json` (`bostrom/snapshot/`, `space-pussy/snapshot/`) pins every file by CID, sha256 and size and declares its row counts. every count in this contract is the manifest's declared count, recounted from the decompressed files; replay verifies against it and never substitutes a re-derived figure.

## bijective, zero loss

every source row produces exactly the genesis entries `chain.md` §4.2 assigns it, and every genesis entry traces to a source row:

| genesis entry | equals |
|---|---|
| `l` frames | passports + cyberlink rows, in source order, duplicates kept |
| accounts | auth accounts |
| owner ids | every address the snapshot names, once each, unique |
| particles | distinct particle addresses the cyberlink rows name; a particle whose bytes are not in the burial is a black hole, addressed but empty, and still counts |
| Σ `b` per carried token | `supply.json` for that token, after the decomposition `chain.md` §4.2 defines |

no row is deduplicated, coalesced or silently dropped. what the phase-1 rules do not carry is named, never lost: a token whose balance or total exceeds 2^64 − 1 is excluded whole and listed in the build report; delegated and undelegating amounts return to their accounts as liquid balances (the pure-PoW rule set has no staking; the shadow staking indexer reads `delegations.csv`); LP positions dissolve into their pool reserves; rounding dust stays on the pool accounts `chain.md` §4.2 names.

## verification

before a genesis candidate is accepted, on both chains:

1. recount the source rows and diff against the manifest. a mismatch means the snapshot itself changed and replay has not started.
2. recount the genesis entries for every row of the table above and diff against step 1. any gap is loss, any excess is a duplicate; either blocks genesis.
3. build twice: the files are byte-identical, and `soft3 genesis verify --root` re-derives the file particle, the state root and the block-0 id from the file alone.
4. record the counts, the state root and the block-0 id, dated, against the exact manifest pins, in soft3 `audit/genesis-*.md`. property 14 on [[cyber/launch|launch]] carries the date and the root.

see [[particle]] for the identity in play · [[bostrom]] for the snapshot · [[cyber/launch|launch]] for property 14.
