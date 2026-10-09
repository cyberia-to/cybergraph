// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox arithmetic and bitwise patterns → zheng public v3.
//!
//! Pipeline under test:
//!   nox::reduce()                — evaluates field-arithmetic (tags 5–7) and bitwise (tag 11) formulas
//!   zheng certify_execution      — certificate v3 of the exact formula and object
//!   zheng verify_certificate     — recompiles the relation, checks every row exactly
//! Each case also rejects a substituted output.

mod common;

use common::{make_field_binop, make_word_binop};

use nebu::Goldilocks;
use nox::{NullCalls, Outcome, Reduction, VecTrace, reduce};

const ORDER_SIZE: usize = 1024;

// ── field arithmetic ──────────────────────────────────────────────────────────

/// Formula [5 [[1 3] [1 5]]] (add) → 3-row trace → public v3 certificate.
#[test]
fn add_field_full_proof_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(Goldilocks::new(0)).unwrap();
    let formula = make_field_binop(&mut order, 5, 3, 5); // add(3, 5)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "add must succeed");
    let output = common::result(outcome);
    assert_eq!(order.atom_value(output).unwrap().as_u64(), 8);
    common::verify_public_execution(&order, obj, formula, output);
}

/// Formula [6 [[1 10] [1 3]]] (sub) → 3-row trace → public v3 certificate.
#[test]
fn sub_field_full_proof_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(Goldilocks::new(0)).unwrap();
    let formula = make_field_binop(&mut order, 6, 10, 3); // sub(10, 3)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "sub must succeed");
    let output = common::result(outcome);
    assert_eq!(order.atom_value(output).unwrap().as_u64(), 7);
    common::verify_public_execution(&order, obj, formula, output);
}

/// Formula [7 [[1 6] [1 7]]] (mul) → 3-row trace → public v3 certificate.
#[test]
fn mul_field_full_proof_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(Goldilocks::new(0)).unwrap();
    let formula = make_field_binop(&mut order, 7, 6, 7); // mul(6, 7)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "mul must succeed");
    let output = common::result(outcome);
    assert_eq!(order.atom_value(output).unwrap().as_u64(), 42);
    common::verify_public_execution(&order, obj, formula, output);
}

// ── bitwise ───────────────────────────────────────────────────────────────────

/// Formula [11 [[1 a] [1 b]]] (xor) → 34-row trace (2 quotes + 32 bit rows) → public v3 certificate.
///
/// Each of the 32 bit rows emits one row with r[0]=11.
/// The verifier-derived execution relation binds the complete formula.
#[test]
fn xor_bitwise_full_proof_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let obj = order.atom(Goldilocks::new(0)).unwrap();
    let formula = make_word_binop(&mut order, 11, 0b1100_1010, 0b1010_0101); // xor

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 5000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "xor must succeed");
    let output = common::result(outcome);
    assert_eq!(order.atom_value(output).unwrap().as_u64(), 0b0110_1111);
    common::verify_public_execution(&order, obj, formula, output);
    let xor_rows = trace.0.iter().filter(|r| r.r()[0] == 11).count();
    assert_eq!(xor_rows, 32, "xor must emit 32 bit rows");
}
