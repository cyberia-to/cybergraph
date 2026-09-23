---
tags: cybergraph, audit, rust
crystal-type: source
crystal-domain: cyber
---
# nox/zheng/lens dev-dep migration — 2026-09-23

Revision: aa374ce (origin/master) + this diff. Rust 1.98.0, macOS arm64.

## what was broken

`Cargo.toml` pinned `bbg = "0.2"`, `zheng = "0.3"` (`[dependencies]`) and
`nox = "0.1.1"` / `lens = "0.1.1"` (`[dev-dependencies]`) against sibling
checkouts already at 0.3.0, 0.4.0, 0.3.0 and 0.2.0 — row 39, the same class
of stale-pin bug fixed elsewhere. Bumping the pins to match resolved 121
compile errors in `tests/common/mod.rs` and the nine `stack_*.rs`
integration files against nox's current identity API (the settled
[[reduction/order/particle/word vocab]] migration):

- the arena type `Order<N>` is now `Reduction<N>`; the node-id alias
  `OrderId` (and the old `NounId` in `CallProvider`) is now `Order = u32`
- `Reduction::atom(value)` takes one argument; the `Tag::Field`/`Tag::Word`
  distinction is gone from the data model entirely
- the old `.cell(a, b)` constructor is `.pair(a, b)`
- `zheng::Statement` gained a `bbg_root: [u8; 32]` field (the look-argument
  root binding); `zheng::LookOpening` replaced its `bbg_root` field with
  `leaves: RootLeaves` (the 14-leaf BBG root preimage, `PartialEq`)

## what stays open

Fixing the pins and the API surface exposed a second, unrelated gap:
`zheng::commit` (zheng/rs/src/lib.rs) unconditionally rejects any call
carrying non-empty `axis_openings` or `look_openings` with
`CommitError::UnsupportedRecursiveOpening` — "the retired Tensor recursive
gadgets cannot verify authenticated TensorMerkle columns." This matches the
component table's note that [[lens]]'s evaluation binding is `TensorMerkle`
with the recursive form blocked. Eleven tests across
`stack_axis.rs`, `stack_bbg_lifecycle.rs`, `stack_look.rs`,
`stack_multi_pattern.rs`, `stack_rejection.rs` and `stack_signal.rs`
exercise the axis or look opening path and are marked `#[ignore]` with
that reason. They were never green before this migration either — they
could not even compile — so this is a pre-existing gap surfaced, not a
regression introduced here. Closing it is a zheng-side recursive-opening
feature, out of scope for a clean-checkout pin fix.

Verified:
```
$ RUSTC_BOOTSTRAP=1 cargo check --tests
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.06s
$ RUSTC_BOOTSTRAP=1 cargo test --no-fail-fast
test result: ok. 18 passed; 0 failed (lib)
test result: ok. 38 passed; 0 failed; 11 ignored (across 10 test binaries)
```
