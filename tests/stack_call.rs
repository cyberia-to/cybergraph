// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox call pattern (tag 16) → zheng proof.
//!
//! Pipeline under test:
//!   CallProvider impl  — prover-supplied witness injection
//!   nox::reduce()      — evaluates call formula, checks witness via check_formula
//!   zheng::commit()    — folds main steps for the accepted-witness case
//!   zheng::verify()    — checks SuperSpartan proof

mod common;

use common::{zero_statement, default_params, make_call_formula, g};

use nebu::Goldilocks;
use nox::{reduce, Order, NounId, Tag, VecTrace, NullCalls, Outcome, ErrorKind, CallProvider, LookProvider};
use nox::trace::NoTrace;
use zheng::{commit, verify};

const ORDER_SIZE: usize = 1024;

// ── local providers ───────────────────────────────────────────────────────────

/// Always returns witness=42 for any tag; look always returns None.
struct FixedWitness42;

impl LookProvider for FixedWitness42 {
    fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> { None }
}

impl<const N: usize> CallProvider<N> for FixedWitness42 {
    fn provide(&self, order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        Some(order.atom(g(42), Tag::Field).unwrap())
    }
}

/// Returns witness=99; the check formula [1 99] always evaluates to 99 ≠ 0 → rejected.
struct BadWitness99;

impl LookProvider for BadWitness99 {
    fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> { None }
}

impl<const N: usize> CallProvider<N> for BadWitness99 {
    fn provide(&self, order: &mut Order<N>, _tag: Goldilocks, _object: NounId) -> Option<NounId> {
        Some(order.atom(g(99), Tag::Field).unwrap())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// FixedWitness42 supplies witness=42; check=[1 0] always returns 0 (accepted).
/// Outcome::Ok; trace committed and verified via zheng.
#[test]
fn call_with_accepted_witness_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(g(0), Tag::Field).unwrap();
    // formula: [16 [[1 call_tag] [1 0]]] — check always returns 0 (quote(0) = 0)
    let formula = make_call_formula(&mut order, 7);  // call_tag=7

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &FixedWitness42, &mut trace);

    let witness_id = match outcome {
        Outcome::Ok(w, _) => w,
        o => panic!("expected Ok, got {:?}", o),
    };
    let (val, _) = order.atom_value(witness_id).expect("witness must be atom");
    assert_eq!(val.as_u64(), 42, "returned witness must be 42");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("call proof must verify");
}

/// BadWitness99 provides witness=99; check formula quotes 99 → check=99 ≠ 0 → CallRejected.
///
/// The check formula is `[1 99]` (quote(99)), which evaluates to 99 regardless of
/// the witness_object. Since 99 ≠ 0, the call pattern returns CallRejected.
#[test]
fn call_rejected_witness_returns_error() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj = order.atom(g(0), Tag::Field).unwrap();

    // Build formula [16 [[1 0] [1 99]]]:  tag=quote(0), check=quote(99) → always rejects
    let t16    = order.atom(g(16), Tag::Field).unwrap();
    let t1     = order.atom(g(1),  Tag::Field).unwrap();
    let zero   = order.atom(g(0),  Tag::Field).unwrap();
    let n99    = order.atom(g(99), Tag::Field).unwrap();
    let tag_f   = order.cell(t1, zero).unwrap();
    let check_f = order.cell(t1, n99).unwrap();
    let body    = order.cell(tag_f, check_f).unwrap();
    let formula = order.cell(t16, body).unwrap();

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &BadWitness99, &mut trace);
    assert!(
        matches!(outcome, Outcome::Error(ErrorKind::CallRejected)),
        "expected CallRejected, got {:?}", outcome,
    );
}

/// NullCalls provides no witness (None) → Halt.
#[test]
fn call_null_provider_halts() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(g(0), Tag::Field).unwrap();
    let formula = make_call_formula(&mut order, 0);

    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut NoTrace);
    assert!(
        matches!(outcome, Outcome::Halt(_)),
        "expected Halt with NullCalls, got {:?}", outcome,
    );
}
