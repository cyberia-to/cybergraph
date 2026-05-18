// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox look pattern (tag 17) → BBG state → zheng proof.
//!
//! Pipeline under test:
//!   nox::reduce()         — executes look formula, records 4 BBG root limbs in trace
//!   ProofLookProvider     — reads BbgState, produces LookOpening with bbg_root
//!   zheng::commit()       — folds main + verifier_steps + 4 eq_steps (root binding)
//!   zheng::verify()       — checks SuperSpartan proof
//!
//! Three tests cover:
//!   1. Time dimension — primary natural use case (u64 key = block height)
//!   2. Neurons dimension — particle-keyed dimension with u64-compatible key
//!   3. Agreement between ProofLookProvider (inline) and collect_look_openings (post-exec)

mod common;

use common::{bbg_object_from_state, default_params, make_look_formula, seeded_bbg_state,
    zero_statement};

use nox::{reduce, Order, VecTrace, Outcome};
use bbg::{ProofLookProvider, collect_look_openings, verify_opening};
use zheng::{commit, verify};

const ORDER_SIZE: usize = 1024;

/// Full pipeline: look(Time, height=0) with real BBG state → zheng commit → verify.
#[test]
fn look_time_dimension_full_proof_roundtrip() {
    let state = seeded_bbg_state();
    let prov  = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 8, 0); // Dim::Time = 8, height = 0

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "look must succeed");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1);

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("Time look proof must verify");
}

/// Full pipeline: look(Neurons, key=42) where key bytes 8..32 are zero → commit → verify.
///
/// Particle-keyed dimensions require keys whose first 8 bytes encode the u64 key and
/// bytes 8..32 are zero; nox passes the look key as a single Goldilocks element.
#[test]
fn look_neurons_dimension_full_proof_roundtrip() {
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&42u64.to_le_bytes());

    let mut state = bbg::BbgState::new();
    state.neurons.insert(key, bbg::types::NeuronRecord { focus: 1_000, karma: 0, stake: 0 });

    let prov = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 3, 42); // Dim::Neurons = 3, key = 42

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "neuron look must succeed");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1);

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("Neurons look proof must verify");
}

/// ProofLookProvider (inline during execution) and collect_look_openings (post-execution)
/// must produce openings that:
///   (a) individually verify via verify_opening
///   (b) carry identical bbg_root limbs (same state root at time of query)
///   (c) both produce proofs that pass zheng::verify
#[test]
fn look_provider_and_collect_openings_both_verify() {
    let state = seeded_bbg_state();
    let prov  = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 8, 0); // Time, height = 0

    let mut trace = VecTrace::default();
    reduce(&mut order, obj, formula, 1000, &prov, &mut trace);

    let prov_openings = prov.take_look_openings();
    assert_eq!(prov_openings.len(), 1);
    assert!(verify_opening(&prov_openings[0]), "provider opening must be valid");

    let col_openings = collect_look_openings(&state, &trace.0);
    assert_eq!(col_openings.len(), 1);
    assert!(verify_opening(&col_openings[0]), "collected opening must be valid");

    assert_eq!(
        prov_openings[0].bbg_root, col_openings[0].bbg_root,
        "both paths must record the same BBG root limbs",
    );

    let stmt   = zero_statement();
    let params = default_params();

    let proof_prov = commit(&trace, &[], &[], &prov_openings, &stmt, &params).unwrap();
    verify(&proof_prov, &stmt, &params).expect("provider-path proof must verify");

    let proof_col = commit(&trace, &[], &[], &col_openings, &stmt, &params).unwrap();
    verify(&proof_col, &stmt, &params).expect("collect-path proof must verify");
}
