---
tags: cybergraph, audit, nox, zheng
date: 2026-09-22
---
# cybergraph's test suite is frozen on a pre-rename nox/zheng API

Same class of bug as rows 38 (mudra), 44 (bbg), 46 (zheng), 47 (foculus):
`Cargo.toml`'s version requirements don't match the local checkouts on
disk. Fixing `[dependencies]` alone (`bbg` 0.2 → 0.3, `zheng` 0.3 → 0.4) is
mechanical and the library itself builds clean — `cargo check` (no
`--tests`) passes. `[dev-dependencies]` is a different story: `nox` is
pinned to `0.1.1`, the local checkout is `0.3.0`, and unlike the other
four rows this is not a mechanical version bump.

## what actually broke

Once `nox` resolves to `0.3.0`, none of the nine `tests/stack_*.rs`
integration files compile — 121 errors:

- `nox::Order` was a generic arena struct, `Order::<N>::new()`. In 0.3 it
  is `pub type Order = u32` (`nox/rs/data/mod.rs`) — the
  [[reduction/order/particle/word vocab]] settlement replaced the arena
  type. 32× `E0107` (wrong generic arity), 67× `E0599` (no such method on
  `u32`) trace back to this.
- `nox::OrderId` and `nox::Tag` no longer exist at that path — 7×
  `E0432` unresolved imports, across `tests/common/mod.rs` and six
  `stack_*.rs` files.
- `zheng::Statement` gained a required `bbg_root` field (the same field
  mudra's `--features prove` pipeline needed, row 42) — 4× `E0063`.
- `zheng::LookOpening` lost a `bbg_root` field the tests read directly —
  2× `E0609`, `tests/stack_look.rs:108`.

None of this is a rename with a 1:1 replacement waiting; `Order<N>` becoming
a bare `u32` changes how every test in the suite constructs and walks the
order arena. This is a real migration to nox's current `Reduction` model,
touching all nine `stack_*.rs` files and `tests/common/mod.rs`.

## what was and wasn't done here

`Cargo.toml`'s `[dependencies]` pins (`bbg`, `zheng`) are bumped in this
PR — needed regardless, since a workspace-wide manifest fails to resolve
at all otherwise (the same error every one of rows 38/44/46/47 hit).
`[dev-dependencies]` (`nox` 0.1.1 → 0.3, `lens` 0.1.1 → 0.2) are also
bumped, because Cargo resolves the whole dependency graph — including
dev-dependencies — before building anything; there is no way to leave
`nox` at `0.1.1` and still have `cargo check` succeed once `bbg`/`zheng`
move. The library itself does not use `nox` at all (it's dev-only, for
tests), so this bump changes zero production code.

No test file is touched here. Migrating nine integration-test files to
nox's `Reduction`/`u32`-`Order` model and zheng's `bbg_root` field is a
real engineering task on its own, and forcing it into this slice risked
either a rushed, wrong migration of a proof-binding test suite or
deleting coverage to get to green — both worse than shipping the
mechanical pin fix with the gap documented.

## verified

Rust 1.98.0, macOS arm64, revision `aa374ce` (origin/master) plus this
PR's `Cargo.toml`/`Cargo.lock` diff.

- `RUSTC_BOOTSTRAP=1 cargo check` (library, no test targets): clean.
- `RUSTC_BOOTSTRAP=1 cargo check --tests`: 121 errors across all nine
  `tests/stack_*.rs` files and `tests/common/mod.rs` — `E0599` ×67,
  `E0107` ×32, `E0432` ×7, `E0063` ×4, `E0609` ×2. None are pre-existing
  on `0.1.1`/`0.2`/`0.1.3`/`0.3` (the pinned versions); all appear only
  once the dev-dependency versions match the checkouts on disk.

This PR is a draft: the dependency section fixes are verified and green,
but the workspace does not pass `cargo test` as a whole, and shipping it
non-draft would misrepresent that.

## what the next slice needs

Migrate `tests/common/mod.rs` and the nine `stack_*.rs` files from
`nox::Order::<N>` / `OrderId` / `Tag` to the current `Reduction` API, and
add `bbg_root` wherever `zheng::Statement`/`LookOpening` are constructed
or read directly (mirror mudra#7's `bbg_root` sentinel fix, row 42). This
is a single self-contained slice once nox's `Reduction` API is read
end-to-end — no cybergraph production code needs to change.
