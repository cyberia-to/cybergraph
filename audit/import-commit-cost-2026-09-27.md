# every commit recomputes every commitment — the native node's cost curve

Date: 2026-09-27. Scope: the main-line node (soft3 `434244e` + cybergraph `feat/durable-applications` `0eae7ae` + bbg `feat/atomic-application-storage` `ef149e5`, the composition the four version PRs describe), exercised by `NativeNode::import_legacy` on the spacepussy-test signal log (39,060 signals, 9,977,600 bytes, source sha256 `a588264f…`). No code changed.

## the finding in one paragraph

`prepare_native_batch` calls `BbgState::refresh_root` on every accepted operation, and `refresh_root` → `root_leaves` → `commit_fields` rebuilds the commitment of every dimension from its full record set (stack sampled with gdb on deimos, always inside `cyber_hemera::permutation::permute_with_constants` under `bbg::dim::commit_fields` ← `root_leaves` ← `refresh_root` ← `prepare_native_batch` ← `accept_encoded` ← `import_legacy`). The cost of accepting a batch is therefore linear in the state, and the cost of a chain is quadratic in its length. This is the live acceptance path, not an import-only path: every `POST /v1/link` on the new node pays the same full recommit.

## measured

Import of a strict prefix of the log (M4 Max, one thread, release build, `spt-import HOME LOG N`):

| events | wall | ratio per doubling |
|---|---|---|
| 250 | 20.1 s | — |
| 500 | 67.3 s | 3.3× |
| 1,000 | 193.7 s | 2.9× |

Exponent ≈ 1.7 and rising toward 2 as the per-batch recommit dominates. Extrapolated to the full 39,060-signal log: ≈ 1.5–2 days on the M4 Max, longer on the 2-core cyberproxy. On deimos (x86_64, 8 cores, one used) the import ran 55 minutes and was committing one 64-event batch every ≈ 40 s when stopped, consistent with ≈ 150 of 610 batches done.

For the launch this means: the chaosnet log cannot be imported in an operational window, and the bostrom genesis replay (2,949,732 links, row 14) cannot run at all on this path — it would take years.

## what has to change

1. **the root is refreshed per block, not per operation.** With blocks as ticks (the epoch design of 2026-09-27) a root is needed once per block; per-operation receipts bind to the block root. This removes the factor "operations per block" but leaves the recommit linear in state.
2. **commitments are incremental.** Each dimension's commitment must be updatable in O(log n) per changed record (a Merkle or tensor-Merkle tree with cached inner nodes), so a block costs O(changes · log n) rather than O(state). This is the actual fix and it is bbg's.
3. **import is a bulk build.** A legacy import should build the dimension tables first and commit once at the end, then write the per-batch history against the final root or a per-block root; the "exact prefix" resumability the current import guarantees can be kept by batching at block granularity.

Until 2 lands, the main-line node is not deployable on spacepussy-test at its current size, and the 0.10.0 node (signal log, no per-signal recommit) stays live. Tracker: `cyber/launch.md` rows 14 and 43.

## reproduce

```
# build spt-import against the four PR branches (see cyber/launch.md decisions 2026-09-27)
spt-import HOME LOG 250 ; spt-import HOME LOG 500 ; spt-import HOME LOG 1000
# stack of a running import: gdb -p PID -batch -ex "bt 12"
```
