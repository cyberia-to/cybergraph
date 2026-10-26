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

use common::{default_params, g, make_call_formula, zero_statement};

use nebu::Goldilocks;
use nox::trace::NoTrace;
use nox::{
    CallProvider, ErrorKind, LookProvider, NullCalls, Order, Outcome, Reduction, VecTrace, reduce,
};
use zheng::{commit, verify};

const ORDER_SIZE: usize = 1024;

// ── local providers ───────────────────────────────────────────────────────────

/// Always returns witness=42 for any tag; look always returns None.
struct FixedWitness42;

impl LookProvider for FixedWitness42 {
    fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> {
        None
    }
}

impl<const N: usize> CallProvider<N> for FixedWitness42 {
    fn provide(&self, order: &mut Reduction<N>, _tag: Goldilocks, _object: Order) -> Option<Order> {
        Some(order.atom(g(42)).unwrap())
    }
}

/// Returns witness=99; the check formula [1 99] always evaluates to 99 ≠ 0 → rejected.
struct BadWitness99;

impl LookProvider for BadWitness99 {
    fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> {
        None
    }
}

impl<const N: usize> CallProvider<N> for BadWitness99 {
    fn provide(&self, order: &mut Reduction<N>, _tag: Goldilocks, _object: Order) -> Option<Order> {
        Some(order.atom(g(99)).unwrap())
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// FixedWitness42 supplies witness=42; check=[1 0] always returns 0 (accepted).
/// Outcome::Ok; trace committed and verified via zheng.
#[test]
fn call_with_accepted_witness_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(g(0)).unwrap();
    // formula: [16 [[1 call_tag] [1 0]]] — check always returns 0 (quote(0) = 0)
    let formula = make_call_formula(&mut order, 7); // call_tag=7

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &FixedWitness42, &mut trace);

    let witness_id = match outcome {
        Outcome::Ok(w, _) => w,
        o => panic!("expected Ok, got {:?}", o),
    };
    let val = order.atom_value(witness_id).expect("witness must be atom");
    assert_eq!(val.as_u64(), 42, "returned witness must be 42");

    // Current public direct proof over the independently regenerated relation.
    // The fixture witness is public test data; this backend is not a ZK claim.
    let program = common::execution_noun(&order, formula);
    let (statement, prepared, columns) =
        zheng::execution::private::prepare_execution(&program, &[], &[42], 1000)
            .expect("checked call witness preparation");
    assert_eq!(statement.execution.public_output, vec![42]);
    assert!(zheng::execution::private::prepare_execution(&program, &[], &[], 1000).is_err());
    assert!(zheng::execution::private::prepare_execution(&program, &[], &[42, 99], 1000).is_err());
    let witness = zheng::CCSWitness {
        z: columns.into_iter().map(g).collect(),
    };
    let public: Vec<_> = prepared
        .public_coordinates
        .iter()
        .map(|&(i, v)| (i, g(v)))
        .collect();
    let direct = zheng::execution::proof::prove(
        &prepared.relation.instance,
        &witness,
        &statement.transcript_bytes(),
        &public,
    )
    .unwrap();
    let verifier = statement.prepare().unwrap();
    let bindings: Vec<_> = verifier
        .public_coordinates
        .iter()
        .map(|&(i, v)| (i, g(v)))
        .collect();
    zheng::execution::proof::verify(
        &verifier.relation.instance,
        &direct,
        &statement.transcript_bytes(),
        &bindings,
    )
    .unwrap();
    let mut wrong = statement.clone();
    wrong.execution.public_output[0] = 99;
    let wrong_verifier = wrong.prepare().unwrap();
    let wrong_bindings: Vec<_> = wrong_verifier
        .public_coordinates
        .iter()
        .map(|&(i, v)| (i, g(v)))
        .collect();
    assert!(
        zheng::execution::proof::verify(
            &wrong_verifier.relation.instance,
            &direct,
            &wrong.transcript_bytes(),
            &wrong_bindings
        )
        .is_err()
    );

    let stmt = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("call proof must verify");
}

/// BadWitness99 provides witness=99; check formula quotes 99 → check=99 ≠ 0 → CallRejected.
///
/// The check formula is `[1 99]` (quote(99)), which evaluates to 99 regardless of
/// the witness_object. Since 99 ≠ 0, the call pattern returns CallRejected.
#[test]
fn call_rejected_witness_returns_error() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(g(0)).unwrap();

    // Build formula [16 [[1 0] [1 99]]]:  tag=quote(0), check=quote(99) → always rejects
    let t16 = order.atom(g(16)).unwrap();
    let t1 = order.atom(g(1)).unwrap();
    let zero = order.atom(g(0)).unwrap();
    let n99 = order.atom(g(99)).unwrap();
    let tag_f = order.pair(t1, zero).unwrap();
    let check_f = order.pair(t1, n99).unwrap();
    let body = order.pair(tag_f, check_f).unwrap();
    let formula = order.pair(t16, body).unwrap();

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &BadWitness99, &mut trace);
    assert!(
        matches!(outcome, Outcome::Error(ErrorKind::CallRejected)),
        "expected CallRejected, got {:?}",
        outcome,
    );
}

/// NullCalls provides no witness (None) → Halt.
#[test]
fn call_null_provider_halts() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(g(0)).unwrap();
    let formula = make_call_formula(&mut order, 0);

    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut NoTrace);
    assert!(
        matches!(outcome, Outcome::Halt(_)),
        "expected Halt with NullCalls, got {:?}",
        outcome,
    );
}
