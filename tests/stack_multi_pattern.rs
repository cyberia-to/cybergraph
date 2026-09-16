// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! One actual composed formula per trace. Public execution binds all operations;
//! current TensorMerkle recursion is tested as an explicit unsupported profile.
mod common;
use bbg::ProofLookProvider;
use common::{
    bbg_object_from_state, g, make_field_binop, make_look_formula, noun_leaves, result,
    seeded_bbg_state, verify_public_execution, verify_state_execution,
};
use nox::{NullCalls, Reduction, VecTrace, reduce};
use zheng::execution::ExecutionNoun;

#[test]
fn mixed_add_hash_axis_public_execution() {
    let mut arena = Reduction::<1024>::new();
    let object = arena.atom(g(0)).unwrap();
    let add = make_field_binop(&mut arena, 5, 3, 7);
    let input = arena.atom(g(42)).unwrap();
    let one = arena.atom(g(1)).unwrap();
    let quote = arena.pair(one, input).unwrap();
    let fifteen = arena.atom(g(15)).unwrap();
    let hash = arena.pair(fifteen, quote).unwrap();
    let zero = arena.atom(g(0)).unwrap();
    let axis = arena.pair(zero, one).unwrap();
    let three = arena.atom(g(3)).unwrap();
    let hash_axis = arena.pair(hash, axis).unwrap();
    let pair_formula = arena.pair(three, hash_axis).unwrap();
    let body = arena.pair(add, pair_formula).unwrap();
    let formula = arena.pair(three, body).unwrap();
    let mut trace = VecTrace::default();
    let output = result(reduce(
        &mut arena, object, formula, 1000, &NullCalls, &mut trace,
    ));
    assert_eq!(trace.0.iter().filter(|r| r.r()[0] == 15).count(), 25);
    assert_eq!(trace.0.iter().filter(|r| r.r()[0] == 0).count(), 1);
    let values = noun_leaves(&arena, output);
    assert_eq!(values.len(), 6);
    assert_eq!(values[0], 10);
    assert_eq!(values[5], 0);
    // Independently obtain the hash result through the actual nox hash formula.
    let hash_output = result(reduce(
        &mut arena,
        object,
        hash,
        1000,
        &NullCalls,
        &mut nox::NoTrace,
    ));
    assert_eq!(&values[1..5], noun_leaves(&arena, hash_output));
    verify_public_execution(&arena, object, formula, output);

    // A different arithmetic leg must reject the same public execution proof.
    let program = common::execution_noun(&arena, formula);
    let (statement, proof) = zheng::execution::prove_execution(&program, &[], 1000).unwrap();
    assert_eq!(statement.public_output, values);
    let mut wrong = statement.clone();
    wrong.program = zheng::execution::ExecutionStatement::encode_program(&ExecutionNoun::Pair(
        Box::new(ExecutionNoun::Atom(1)),
        Box::new(ExecutionNoun::Atom(10)),
    ))
    .unwrap();
    assert!(zheng::execution::verify_execution(&wrong, &proof).is_err());
}

#[test]
fn mixed_add_look_public_state_execution() {
    let state = seeded_bbg_state();
    let provider = ProofLookProvider::new(&state);
    let mut arena = Reduction::<1024>::new();
    let object = bbg_object_from_state(&mut arena, &state);
    let add = make_field_binop(&mut arena, 5, 4, 8);
    let index = bbg::prove_time(&state, 0).unwrap().context.unwrap().index;
    let look = make_look_formula(&mut arena, 8, index);
    let five = arena.atom(g(5)).unwrap();
    let body = arena.pair(add, look).unwrap();
    let formula = arena.pair(five, body).unwrap();
    let mut trace = VecTrace::default();
    let output = result(reduce(
        &mut arena, object, formula, 1000, &provider, &mut trace,
    ));
    let expected = 12 + common::authenticated_value(&state, &state.root(), 8, index).unwrap();
    assert_eq!(noun_leaves(&arena, output), vec![expected]);
    assert_eq!(trace.0.iter().filter(|r| r.r()[0] == 17).count(), 1);
    let openings = provider.take_look_openings();
    common::verify_look_openings(&state, &trace, &openings);
    verify_state_execution(&arena, formula, &state, &[expected]);
    common::refuse_recursive_look(&trace, &state, &openings);
}
