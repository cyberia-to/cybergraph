// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox Poseidon2 hash pattern (tag 15) → zheng public v3.
//!
//! Pipeline under test:
//!   nox::reduce()             — executes hash formula, emits 26 trace rows (1 quote + 24 rounds + 1 squeeze)
//!   zheng certify_execution   — certificate v3; the relation compiles the full
//!                               structural hemera hash and final permutation
//!   zheng verify_certificate  — checks every row exactly

mod common;

use nebu::Goldilocks;
use nox::{NullCalls, Order, Reduction, VecTrace, reduce};
use zheng::execution::verify_certificate;

const ORDER_SIZE: usize = 1024;

/// `[15 [1 s]]` over subject `s`.
fn hash_formula<const N: usize>(order: &mut Reduction<N>, s: Order) -> Order {
    let tag1 = order.atom(Goldilocks::new(1)).unwrap();
    let tag15 = order.atom(Goldilocks::new(15)).unwrap();
    let quote_f = order.pair(tag1, s).unwrap(); // [1 s]
    order.pair(tag15, quote_f).unwrap() // [15 [1 s]]
}

/// Full pipeline: hash(quote(42)) → 26-row trace → public v3 certificate.
#[test]
fn hash_poseidon2_single_atom_roundtrip() {
    let mut order = Reduction::<ORDER_SIZE>::new();
    let s = order.atom(Goldilocks::new(42)).unwrap();
    let hash_f = hash_formula(&mut order, s);

    let mut trace = VecTrace::default();
    let output = common::result(reduce(&mut order, s, hash_f, 100, &NullCalls, &mut trace));
    // 1 quote row + 24 Poseidon2 round rows + 1 squeeze row
    assert_eq!(trace.0.len(), 26, "hash trace must be 26 rows");
    let (statement, _) = common::verify_public_execution(&order, s, hash_f, output);
    assert_eq!(statement.public_output.len(), 4, "a 4-limb digest");
}

/// hash over a public input: `[15 [0 2]]` reads the input from the subject.
/// With a quoted constant the relation fixes every wire and the certificate is
/// empty; with an input the hash rounds are free witness positions.
#[test]
fn hash_two_independent_inputs_both_verify() {
    use zheng::execution::{ExecutionNoun as N, certify_execution};
    let program = N::Pair(
        Box::new(N::Atom(15)),
        Box::new(N::Pair(Box::new(N::Atom(0)), Box::new(N::Atom(2)))),
    );
    let native = |input_val: u64| {
        let mut order = Reduction::<ORDER_SIZE>::new();
        let s = order.atom(Goldilocks::new(input_val)).unwrap();
        let hash_f = hash_formula(&mut order, s);
        let mut trace = VecTrace::default();
        let output = common::result(reduce(&mut order, s, hash_f, 100, &NullCalls, &mut trace));
        let (statement, certificate) = common::verify_public_execution(&order, s, hash_f, output);
        assert!(
            certificate.free.is_empty(),
            "a constant hash is fixed by the relation"
        );
        statement.public_output
    };

    let (statement_a, certificate_a) = certify_execution(&program, &[100], 1000).unwrap();
    let (statement_b, certificate_b) = certify_execution(&program, &[999], 1000).unwrap();
    assert_eq!(statement_a.public_output, native(100));
    assert_eq!(statement_b.public_output, native(999));
    assert!(!certificate_a.free.is_empty());
    verify_certificate(&statement_a, &certificate_a).unwrap();
    verify_certificate(&statement_b, &certificate_b).unwrap();
    assert_ne!(
        statement_a.public_output, statement_b.public_output,
        "structural hashes must bind distinct inputs"
    );
    assert!(
        verify_certificate(&statement_b, &certificate_a).is_err(),
        "hash(100)'s witness cannot certify hash(999)"
    );
    assert!(
        verify_certificate(&statement_a, &certificate_b).is_err(),
        "hash(999)'s witness cannot certify hash(100)"
    );
    let mut relabeled = statement_a.clone();
    relabeled.public_output = statement_b.public_output.clone();
    assert!(verify_certificate(&relabeled, &certificate_a).is_err());
}
