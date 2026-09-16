# Neuron convergence: stack compatibility and rejection atomicity

Date: 2026-09-13. Scope: P13 stack compatibility, with the discovered P04
in-memory graph/SignalChain rejection invariant. This work changes ten stack
test binaries, their dedicated `tests/common` helpers, and the bounded
`src/api.rs` publication/preflight helpers. No component mathematics, proof
backend, Cargo dependency, native history codec or stored identity changed.

## Current profile and obsolete assumptions

The original `cargo test --tests --no-run` failed all nine historical stack
binaries: nox's arena is now `Reduction<N>`, its address is `Order`, atoms are
tag-free, and `Statement` includes `bbg_root`. Word constraints refine those
same atoms. Native NeuronId and VM data addresses keep separate meanings.

After these API-only adaptations, 20 tests passed and 11 failed. The remaining
failures exposed retired profile assumptions, rather than a missing import:

| Old assumption | Current verified behavior |
|---|---|
| Any valid synthetic axis polynomial can stand for the actual noun | Fixtures commit their actual padded leaves; native openings reject wrong values, while a verifier-derived execution relation binds the real object/formula/result |
| An entity's integer key is its nox lookup coordinate | The BBG entity-query owner resolves NeuronId/height to an authenticated flat cell index; nox reads that index |
| A sampled `LookOpening` alone authenticates BBG state | `verify_opening` deliberately refuses; `verify_opening_with_context` requires the complete authenticated query and caller's exact root/index |
| BBG/axis TensorMerkle openings can enter the old recursive trace fold | `zheng::commit` returns `UnsupportedRecursiveOpening`; tests require that refusal separately |
| Concatenating unrelated reductions establishes one program execution | Mixed tests now execute one composed formula and prove its complete public result |
| A zero-statement trace fold alone binds full program semantics | Raw-fold checks remain primitive tests; the current execution proof additionally binds the actual formula, public object, output and context |

The supported successful execution path uses `zheng::execution::DirectProof`
over a verifier-derived relation. State execution uses `StateStatement` and a
callback that verifies current BBG `QueryProof` values against the exact retained
state root before supplying a coordinate. The state proof binds all four root
limbs, namespace/index/value, program, output and execution context. Tests reject
root/context/output substitution, unauthenticated values and omitted reads.

These are public development proofs: their full tables/witness columns are
disclosed. The call fixture uses public test witness 42 and the independently
regenerated prepared relation. It establishes no zero-knowledge guarantee.
Recursive TensorMerkle execution remains unsupported and is an explicit G14
profile limitation. A successful native opening or public direct proof does not
silently upgrade that recursive profile. Local BBG finalization also establishes
no network consensus or economic finality.

## In-memory API defect and repair

Final source review found that `Cybergraph::commit_signal` appended a SignalChain
entry before calling the fallible BBG insertion. Its outdated comment assumed
empty `box_moves`, while the real bridge already carried nullifiers. Six actual
API regression cases failed before the fix:

- a rejected double-spend advanced an existing chain;
- the same rejection installed a new subject's chain;
- a failed lineage check left a newly created empty chain;
- duplicate nullifiers inside one batch were accepted;
- overflowing a public balance panicked after earlier batch mutations;
- rejected `seal` advanced the chain while retaining the old graph.

The repaired boundary uses bounded-by-input temporary sets/maps for same-batch
nullifier uniqueness and ordered touched-balance arithmetic. An unrepresentable
u64 credit returns the new `ApiError::AmountOverflow` before mutation. The check
preserves BBG's existing credit-then-debit order, saturating debit semantics,
zero amounts and representable u64 boundary values.

SignalChain remains the owner of sequence/equivocation validation. A vacant
entry is published only after its append succeeds. If BBG rejects its structural
check, the coordinator removes exactly the tentative chain row and removes the
whole chain when this operation created it. This uses exclusive `&mut` access
and avoids cloning the existing history. Events and header publication follow
successful insertion. BBG's current returned error is `DoubleSpend`, checked
before any of its state mutations; that owner contract is part of this result.

Eight final API regressions cover those six failures, valid ordered transfers at
u64 boundaries, and wrong-network rejection. They compare graph roots, exact
per-subject chain hashes and event counts, verify preserved intents/nullifiers,
and successfully retry corrected work at the original sequence position.
`stack_signal` now sends the same actual Signal through the Cybergraph bridge
before authenticating its BBG query and proving the resulting execution.

This is the in-memory API's returned-error atomicity contract. Durable receipts,
disk errors, import and recovery remain owned by NativeNode/ApplicationGraph/BBG.
The change adds no signing, focus-authorization or consensus policy to the
prevalidated local API, and changes no token arithmetic for valid operations.

## Evidence

Commands run in `/Users/master/cyber/cybergraph` through
`/opt/homebrew/bin/nu -c 'cargo …'`, using Homebrew Rust 1.95.0
(`59807616e1fa2540724bfbac14d7976d7e4a3860`, aarch64-apple-darwin).
The final logs include the exact command and feature profile in their headers.
Registry versions are locked; these are current sibling-source checks, not an
immutable released artifact or cross-platform claim.

| Evidence | Result |
|---|---|
| [Original compile](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-before.log), `cargo test --tests --no-run` | All nine historical stack binaries failed obsolete APIs |
| [API-only adaptation](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-profile-before.log), explicit nine `--test stack_*` targets | 20 pass / 11 profile failures |
| [Migrated nine targets](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-final.log) | 31 pass / 0 fail / 0 ignored / 0 filtered |
| [Atomicity before](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-atomicity-before.log), `cargo test --test stack_api_atomicity --no-fail-fast` | Six reproduced failures |
| [Atomicity after](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-atomicity-after.log), same target | Eight pass / 0 fail |
| [Complete default suite](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-default-final.log), `cargo test --no-fail-fast` | 57 pass / 0 fail / 0 ignored / 0 filtered; doctests 0 |
| [Complete all-features suite](../../soft3/audit/neuron-cell/implementation-baseline/p13-cybergraph-stack-all-features.log), `cargo test --all-features --no-fail-fast` | 93 pass / 0 fail / 0 ignored / 0 filtered; doctests 0 |

The default suite includes 18 unit and 39 stack tests.
All-features enables `local-storage` and `legacy-redb-migration`. Its 93 tests
include 24 unit, 6 application, 11 native-import, 10 native-storage, 39 stack and
3 text-archive tests. Storage tests are the source state observed during this
run; subsequent independently edited archive/transfer work requires its own
final closure checks. No historical stack test was disabled or ignored.

`git diff --check` passes for the edited production and test files. Remaining
compiler warnings in these logs come from the existing vendored fjall fields
and lifetime spelling, outside this repair.
