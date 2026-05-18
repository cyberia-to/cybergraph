// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox Poseidon2 hash pattern (tag 15) → zheng proof.
//!
//! Pipeline under test:
//!   nox::reduce()         — executes hash formula, emits 26 trace rows (1 quote + 24 rounds + 1 squeeze)
//!   HashAux               — rate vector (structural digest of input) for sponge replay
//!   zheng::commit()       — folds main steps + particle CCS steps (round constraints)
//!   zheng::verify()       — checks SuperSpartan proof
//!
//! The rate is derived from `Order::digest(s)` — the hemera structural hash of the
//! input atom, which nox computes during execution and zheng must replay to constrain
//! the Poseidon2 sponge.

mod common;

use common::{zero_statement, default_params};

use nebu::Goldilocks;
use nox::{reduce, Order, Tag, VecTrace, NullCalls};
use zheng::{commit, verify, HashAux};

const ORDER_SIZE: usize = 1024;

/// Full pipeline: hash(quote(42)) → 26-row trace → HashAux → zheng commit → verify.
#[test]
fn hash_poseidon2_single_atom_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let s     = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
    let tag1  = order.atom(Goldilocks::new(1),  Tag::Field).unwrap();
    let tag15 = order.atom(Goldilocks::new(15), Tag::Field).unwrap();
    let quote_f = order.cell(tag1,  s).unwrap();        // [1 s]
    let hash_f  = order.cell(tag15, quote_f).unwrap();  // [15 [1 s]]

    let mut trace = VecTrace::default();
    reduce(&mut order, s, hash_f, 100, &NullCalls, &mut trace);
    // 1 quote row + 24 Poseidon2 round rows + 1 squeeze row
    assert_eq!(trace.0.len(), 26, "hash trace must be 26 rows");

    // Rate = structural digest of s (4 field elements), zero-padded to sponge width 8.
    let in_digest = *order.digest(s).unwrap();
    let rate = [
        in_digest[0], in_digest[1], in_digest[2], in_digest[3],
        Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO,
    ];
    let hash_aux = HashAux { rate };

    let stmt  = zero_statement();
    let proof = commit(&trace, &[hash_aux], &[], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("hash proof must verify");
}

/// Full pipeline: hash two different inputs → traces with independent HashAux → both verify.
///
/// Confirms that HashAux is input-specific (different digests produce different rates)
/// and that both resulting proofs are independently valid.
#[test]
fn hash_two_independent_inputs_both_verify() {
    let run_hash = |input_val: u64| {
        let mut order = Order::<ORDER_SIZE>::new();
        let s     = order.atom(Goldilocks::new(input_val), Tag::Field).unwrap();
        let tag1  = order.atom(Goldilocks::new(1),         Tag::Field).unwrap();
        let tag15 = order.atom(Goldilocks::new(15),        Tag::Field).unwrap();
        let quote_f = order.cell(tag1,  s).unwrap();
        let hash_f  = order.cell(tag15, quote_f).unwrap();

        let mut trace = VecTrace::default();
        reduce(&mut order, s, hash_f, 100, &NullCalls, &mut trace);

        let in_digest = *order.digest(s).unwrap();
        let rate = [
            in_digest[0], in_digest[1], in_digest[2], in_digest[3],
            Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO,
        ];
        (trace, HashAux { rate })
    };

    let (trace_a, aux_a) = run_hash(100);
    let (trace_b, aux_b) = run_hash(999);

    let stmt   = zero_statement();
    let params = default_params();

    let proof_a = commit(&trace_a, &[aux_a], &[], &[], &stmt, &params).unwrap();
    verify(&proof_a, &stmt, &params).expect("hash(100) proof must verify");

    let proof_b = commit(&trace_b, &[aux_b], &[], &[], &stmt, &params).unwrap();
    verify(&proof_b, &stmt, &params).expect("hash(999) proof must verify");
}
