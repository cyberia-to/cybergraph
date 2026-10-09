// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: negative paths through the stack.
//!
//! Exercises error conditions at each layer:
//!   nox       — Unavailable (absent BBG key)
//!   zheng     — omitted read, budget below cost, wrong input, wrong program
//!               (public and state profile v3)
//!   bbg       — InsertError::DoubleSpend (nullifier reuse)
//!
//! Every test asserts a specific error variant; "should fail" is as important
//! as "should succeed" for proving the constraints are tight.

mod common;

use common::{bbg_object_from_state, make_look_formula, seeded_bbg_state};

use bbg::types::NeuronRecord;
use bbg::{BbgState, NeuronId, Particle, Signal as BbgSignal};
use bbg::{BoxMove, InsertError, ProofLookProvider};
use nebu::Goldilocks;
use nox::{ErrorKind, Outcome, Reduction, VecTrace, reduce};
use zheng::execution::{ExecutionNoun, certify_execution, verify_certificate};

const ORDER_SIZE: usize = 1024;

fn neuron(seed: u8) -> NeuronId {
    [seed; 32]
}
fn particle(seed: u8) -> Particle {
    [seed; 32]
}

// ── nox-level errors ──────────────────────────────────────────────────────────

/// look(Time, index 99) against a state with no such flat coordinate →
/// Outcome::Error(Unavailable), and ProofLookProvider records zero openings.
#[test]
fn look_absent_key_returns_unavailable_no_opening() {
    // The entity API separately confirms missing height 99. Nox reads a flat
    // coordinate; entity identity is resolved by the BBG query owner.
    let state = seeded_bbg_state();
    assert!(bbg::prove_time(&state, 99).is_none());
    let prov = ProofLookProvider::new(&state);

    let mut order = Reduction::<ORDER_SIZE>::new();
    let bbg_obj = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 8, 99); // Dim::Time=8, absent cell index

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, bbg_obj, formula, 1000, &prov, &mut trace);
    assert!(
        matches!(outcome, Outcome::Error(ErrorKind::Unavailable)),
        "absent key must produce Unavailable, got {:?}",
        outcome,
    );
    assert_eq!(
        prov.take_look_openings().len(),
        0,
        "no opening on failed look"
    );
}

// ── zheng state profile v3 ────────────────────────────────────────────────────

/// A statement that omits one of the program's reads fails verification.
#[test]
fn omitted_read_fails_state_verification() {
    // State needs two queryable time entries.
    let mut state = BbgState::new();
    let n = neuron(1);
    state.neurons.insert(
        n,
        NeuronRecord {
            focus: 100_000,
            karma: 0,
            stake: 0,
        },
    );
    state
        .insert(&BbgSignal {
            neuron: n,
            links: vec![],
            box_moves: vec![],
            height: 0,
        })
        .unwrap();
    state.time.insert(0, particle(10));
    state.time.insert(1, particle(11));
    state.refresh_root();

    let prov = ProofLookProvider::new(&state);
    let mut order = Reduction::<ORDER_SIZE>::new();
    let bbg_obj = bbg_object_from_state(&mut order, &state);
    let index0 = bbg::prove_time(&state, 0).unwrap().context.unwrap().index;
    let index1 = bbg::prove_time(&state, 1).unwrap().context.unwrap().index;
    let look_f0 = make_look_formula(&mut order, 8, index0);
    let look_f1 = make_look_formula(&mut order, 8, index1);

    let mut trace = VecTrace::default();
    assert!(matches!(
        reduce(&mut order, bbg_obj, look_f0, 1000, &prov, &mut trace),
        Outcome::Ok(_, _)
    ));
    assert!(matches!(
        reduce(&mut order, bbg_obj, look_f1, 1000, &prov, &mut trace),
        Outcome::Ok(_, _)
    ));

    let all_openings = prov.take_look_openings();
    assert_eq!(all_openings.len(), 2);

    common::verify_look_openings(&state, &trace, &all_openings);
    let tag3 = order.atom(Goldilocks::new(3)).unwrap();
    let body = order.pair(look_f0, look_f1).unwrap();
    let program = order.pair(tag3, body).unwrap();
    let expected = [
        common::authenticated_value(&state, &state.root(), 8, index0).unwrap(),
        common::authenticated_value(&state, &state.root(), 8, index1).unwrap(),
    ];
    let (mut statement, certificate) =
        common::verify_state_execution(&order, program, &state, &expected);
    assert_eq!(statement.reads.len(), 2);
    statement.reads.truncate(1);
    assert!(
        common::verify_state_certificate(&state, &statement, &certificate).is_err(),
        "state certificate v3 rejects an omitted read"
    );
}

// ── zheng public profile v3 ───────────────────────────────────────────────────

fn atom(v: u64) -> ExecutionNoun {
    ExecutionNoun::Atom(v)
}
fn pair(a: ExecutionNoun, b: ExecutionNoun) -> ExecutionNoun {
    ExecutionNoun::Pair(Box::new(a), Box::new(b))
}
/// `[5 [[1 a] [1 b]]]` — add two quoted constants.
fn add(a: u64, b: u64) -> ExecutionNoun {
    pair(
        atom(5),
        pair(pair(atom(1), atom(a)), pair(atom(1), atom(b))),
    )
}

/// A budget below the execution's cost: the prover refuses, and a statement
/// claiming that budget does not verify.
#[test]
fn budget_below_cost_is_refused() {
    let (statement, certificate) = certify_execution(&add(2, 3), &[], 1000).unwrap();
    assert_eq!(statement.public_output, vec![5]);
    assert!(statement.cycles > 1);
    assert!(certify_execution(&add(2, 3), &[], statement.cycles - 1).is_err());
    let mut tight = statement.clone();
    tight.budget = statement.cycles - 1;
    assert!(verify_certificate(&tight, &certificate).is_err());
    let mut exact = statement;
    exact.budget = exact.cycles;
    verify_certificate(&exact, &certificate).expect("budget is an upper bound");
}

/// A certificate for one public input does not verify another input.
#[test]
fn wrong_public_input_fails_verify() {
    // [5 [[0 2] [1 7]]] — subject axis 2 (the input) plus 7.
    let program = pair(
        atom(5),
        pair(pair(atom(0), atom(2)), pair(atom(1), atom(7))),
    );
    let (statement, certificate) = certify_execution(&program, &[5], 1000).unwrap();
    assert_eq!(statement.public_output, vec![12]);
    verify_certificate(&statement, &certificate).unwrap();
    let mut wrong = statement.clone();
    wrong.public_input[0] = 6;
    assert!(verify_certificate(&wrong, &certificate).is_err());
}

/// A's certificate cannot carry A's output over to program B. (A program of
/// quoted constants fixes every wire, so its certificate is empty and any
/// true statement verifies with it; acceptance means the statement is true.)
#[test]
fn false_statement_about_another_program_fails_verify() {
    let (a, certificate) = certify_execution(&add(2, 3), &[], 1000).unwrap();
    assert!(certificate.free.is_empty());
    let (b, _) = certify_execution(&add(2, 4), &[], 1000).unwrap();
    let mut relabeled = b;
    relabeled.public_output = a.public_output.clone();
    assert!(verify_certificate(&relabeled, &certificate).is_err());
    let mut cycles = a.clone();
    cycles.cycles += 1;
    assert!(
        verify_certificate(&cycles, &certificate).is_err(),
        "cost is bound"
    );
}

// ── bbg-level errors ──────────────────────────────────────────────────────────

/// Inserting two signals with the same BoxMove nullifier → InsertError::DoubleSpend.
#[test]
fn double_spend_bbg_insert_rejected() {
    let mut state = BbgState::new();
    let n = neuron(1);
    state.neurons.insert(
        n,
        NeuronRecord {
            focus: 100_000,
            karma: 0,
            stake: 0,
        },
    );

    let nullifier = particle(42);
    let mk_signal = || BbgSignal {
        neuron: n,
        links: vec![],
        box_moves: vec![BoxMove {
            nullifier,
            commitment: None,
        }],
        height: 0,
    };

    state
        .insert(&mk_signal())
        .expect("first insert must succeed");
    assert_eq!(
        state.insert(&mk_signal()),
        Err(InsertError::DoubleSpend),
        "second insert with same nullifier must fail",
    );
}
