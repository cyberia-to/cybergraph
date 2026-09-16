// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! BBG entity navigation → flat nox lookup → complete native query authentication
//! → current public state execution proof. Recursion is an explicit separate refusal.
mod common;
use bbg::{Dim, ProofLookProvider, collect_look_openings};
use common::{
    bbg_object_from_state, make_look_formula, noun_leaves, refuse_recursive_look, result,
    seeded_bbg_state, verify_look_openings, verify_state_execution,
};
use nox::{Reduction, VecTrace, reduce};

fn run_query(
    state: &bbg::BbgState,
    query: bbg::QueryProof,
    dim: Dim,
    key: [u8; 32],
    expected: u64,
) {
    assert!(bbg::verify_entity(&query, &state.root(), dim, &key));
    let index = query.context.as_ref().unwrap().index;
    let provider = ProofLookProvider::new(state);
    let mut arena = Reduction::<1024>::new();
    let object = bbg_object_from_state(&mut arena, state);
    let formula = make_look_formula(&mut arena, dim as u64, index);
    let mut trace = VecTrace::default();
    let output = result(reduce(
        &mut arena, object, formula, 1000, &provider, &mut trace,
    ));
    assert_eq!(noun_leaves(&arena, output), vec![expected]);
    let openings = provider.take_look_openings();
    verify_look_openings(state, &trace, &openings);
    verify_state_execution(&arena, formula, state, &[expected]);
    refuse_recursive_look(&trace, state, &openings);
}

#[test]
fn look_time_native_authentication_and_public_state_proof() {
    let state = seeded_bbg_state();
    run_query(
        &state,
        bbg::prove_time(&state, 0).unwrap(),
        Dim::Time,
        [0; 32],
        u32::from_le_bytes([99; 4]) as u64,
    );
}

#[test]
fn look_neuron_entity_to_flat_index_and_public_state_proof() {
    let mut key = [0u8; 32];
    key[..8].copy_from_slice(&42u64.to_le_bytes());
    let mut state = bbg::BbgState::new();
    state.neurons.insert(
        key,
        bbg::NeuronRecord {
            focus: 1000,
            karma: 0,
            stake: 0,
        },
    );
    state.refresh_root();
    // Entity 42 is resolved by the owner; nox's key is the authenticated cell
    // index, never the integer prefix of NeuronId.
    run_query(
        &state,
        bbg::prove_neuron(&state, &key).unwrap(),
        Dim::Neurons,
        key,
        1000,
    );
}

#[test]
fn inline_and_collected_openings_authenticate_the_same_state() {
    let state = seeded_bbg_state();
    let query = bbg::prove_time(&state, 0).unwrap();
    let index = query.context.as_ref().unwrap().index;
    let provider = ProofLookProvider::new(&state);
    let mut arena = Reduction::<1024>::new();
    let object = bbg_object_from_state(&mut arena, &state);
    let formula = make_look_formula(&mut arena, 8, index);
    let mut trace = VecTrace::default();
    let output = result(reduce(
        &mut arena, object, formula, 1000, &provider, &mut trace,
    ));
    let inline = provider.take_look_openings();
    let collected = collect_look_openings(&state, &trace.0);
    verify_look_openings(&state, &trace, &inline);
    verify_look_openings(&state, &trace, &collected);
    assert_eq!(inline[0].leaves, collected[0].leaves);
    assert_eq!(inline[0].commitment, collected[0].commitment);
    assert_eq!(inline[0].point, collected[0].point);
    assert_eq!(inline[0].value, collected[0].value);
    assert_eq!(inline[0].opening, collected[0].opening);
    verify_state_execution(&arena, formula, &state, &noun_leaves(&arena, output));
    refuse_recursive_look(&trace, &state, &inline);
    refuse_recursive_look(&trace, &state, &collected);
}
