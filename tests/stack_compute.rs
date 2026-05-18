// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox arithmetic and bitwise patterns → zheng proof.
//!
//! Pipeline under test:
//!   nox::reduce()     — evaluates field-arithmetic (tags 5–8) and bitwise (tag 11) formulas
//!   VecTrace          — collects rows; no auxiliary data needed for these patterns
//!   zheng::commit()   — folds main steps; hash/axis/look slices are empty
//!   zheng::verify()   — checks SuperSpartan proof

mod common;

use common::{zero_statement, default_params, make_field_binop, make_word_binop};

use nox::{reduce, Order, Tag, VecTrace, NullCalls, Outcome};
use nebu::Goldilocks;
use zheng::{commit, verify};

const ORDER_SIZE: usize = 1024;

// ── field arithmetic ──────────────────────────────────────────────────────────

/// Formula [5 [[1 3] [1 5]]] (add) → 3-row trace → zheng commit → verify.
#[test]
fn add_field_full_proof_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 5, 3, 5);  // add(3, 5)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "add must succeed");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("add proof must verify");
}

/// Formula [6 [[1 10] [1 3]]] (sub) → 3-row trace → zheng commit → verify.
#[test]
fn sub_field_full_proof_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 6, 10, 3);  // sub(10, 3)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "sub must succeed");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("sub proof must verify");
}

/// Formula [7 [[1 6] [1 7]]] (mul) → 3-row trace → zheng commit → verify.
#[test]
fn mul_field_full_proof_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 7, 6, 7);  // mul(6, 7)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "mul must succeed");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("mul proof must verify");
}

// ── bitwise ───────────────────────────────────────────────────────────────────

/// Formula [11 [[1 a] [1 b]]] (xor) → 34-row trace (2 quotes + 32 bit rows) → zheng commit → verify.
///
/// Each of the 32 bit rows emits one row with r[0]=11.
/// build_ccs_from_trace generates intra-block pairs for consecutive xor rows
/// and boundary trivial_ccs for the quote→xor transition.
#[test]
fn xor_bitwise_full_proof_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_word_binop(&mut order, 11, 0b1100_1010, 0b1010_0101);  // xor

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 5000, &NullCalls, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "xor must succeed");
    let xor_rows = trace.0.iter().filter(|r| r.r()[0] == 11).count();
    assert_eq!(xor_rows, 32, "xor must emit 32 bit rows");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("xor proof must verify");
}
