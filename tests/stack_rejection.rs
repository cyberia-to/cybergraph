// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: negative paths through the stack.
//!
//! Exercises error conditions at each layer:
//!   nox       — Unavailable (absent BBG key)
//!   zheng     — TraceOverflow, FocusExhausted, StatementMismatch, verify failure
//!   bbg       — InsertError::DoubleSpend (nullifier reuse)
//!
//! Every test asserts a specific error variant; "should fail" is as important
//! as "should succeed" for proving the constraints are tight.

mod common;

use common::{
    zero_statement, default_params,
    make_field_binop, make_look_formula,
    bbg_object_from_state, seeded_bbg_state,
};

use nebu::Goldilocks;
use nox::{reduce, Order, Tag, VecTrace, NullCalls, Outcome, ErrorKind};
use zheng::{commit, CommitError, Statement};
use bbg::{BbgState, Signal as BbgSignal, Particle, NeuronId};
use bbg::types::NeuronRecord;
use bbg::{BoxMove, InsertError, ProofLookProvider};

const ORDER_SIZE: usize = 1024;

fn neuron(seed: u8) -> NeuronId  { [seed; 32] }
fn particle(seed: u8) -> Particle { [seed; 32] }

// ── nox-level errors ──────────────────────────────────────────────────────────

/// look(Time, 99) against a state that has no time entry at height 99 →
/// Outcome::Error(Unavailable), and ProofLookProvider records zero openings.
#[test]
fn look_absent_key_returns_unavailable_no_opening() {
    // seeded_bbg_state() has time[0] but not time[99].
    let state = seeded_bbg_state();
    let prov  = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let bbg_obj = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 8, 99);  // Dim::Time=8, height=99 (absent)

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, bbg_obj, formula, 1000, &prov, &mut trace);
    assert!(
        matches!(outcome, Outcome::Error(ErrorKind::Unavailable)),
        "absent key must produce Unavailable, got {:?}", outcome,
    );
    assert_eq!(prov.take_look_openings().len(), 0, "no opening on failed look");
}

// ── zheng commit errors ───────────────────────────────────────────────────────

/// Two look reduces produce 2 look rows; passing only 1 LookOpening to commit()
/// causes TraceOverflow (the second look row has no matching opening).
#[test]
fn look_openings_undercount_commit_fails() {
    // State needs two queryable time entries.
    let mut state = BbgState::new();
    let n = neuron(1);
    state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });
    state.insert(&BbgSignal {
        neuron: n, links: vec![], box_moves: vec![], height: 0,
    }).unwrap();
    state.time.insert(0, particle(10));
    state.time.insert(1, particle(11));

    let prov = ProofLookProvider::new(&state);
    let mut order = Order::<ORDER_SIZE>::new();
    let bbg_obj  = bbg_object_from_state(&mut order, &state);
    let look_f0  = make_look_formula(&mut order, 8, 0);
    let look_f1  = make_look_formula(&mut order, 8, 1);

    let mut trace = VecTrace::default();
    assert!(matches!(reduce(&mut order, bbg_obj, look_f0, 1000, &prov, &mut trace), Outcome::Ok(_, _)));
    assert!(matches!(reduce(&mut order, bbg_obj, look_f1, 1000, &prov, &mut trace), Outcome::Ok(_, _)));

    let mut all_openings = prov.take_look_openings();
    assert_eq!(all_openings.len(), 2);

    // Provide only the first opening — second look row has no match.
    all_openings.truncate(1);
    let stmt = zero_statement();
    let err = commit(&trace, &[], &[], &all_openings, &stmt, &default_params());
    assert!(
        matches!(err, Err(CommitError::TraceOverflow)),
        "undercount openings must produce TraceOverflow, got {:?}", err,
    );
}

/// Trace has 3 rows; Statement.focus_bound=1 → FocusExhausted.
#[test]
fn focus_bound_exceeded_commit_fails() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 5, 2, 3);  // add(2,3)

    let mut trace = VecTrace::default();
    reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);
    assert!(trace.0.len() >= 2, "add produces at least 2 rows");

    let stmt = Statement {
        program_hash: [0u8; 32],
        input_hash:   [0u8; 32],
        output_hash:  [0u8; 32],
        focus_bound:  1,  // trace.len() > 1 → FocusExhausted
    };
    let err = commit(&trace, &[], &[], &[], &stmt, &default_params());
    assert!(
        matches!(err, Err(CommitError::FocusExhausted)),
        "trace exceeding focus_bound must produce FocusExhausted, got {:?}", err,
    );
}

/// Statement.input_hash=[1u8;32] doesn't match the first trace row → StatementMismatch.
#[test]
fn input_hash_mismatch_commit_fails() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 5, 1, 1);

    let mut trace = VecTrace::default();
    reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);

    let stmt = Statement {
        program_hash: [0u8; 32],
        input_hash:   [1u8; 32],  // non-zero, will not match any trace row hash
        output_hash:  [0u8; 32],
        focus_bound:  0,
    };
    let err = commit(&trace, &[], &[], &[], &stmt, &default_params());
    assert!(
        matches!(err, Err(CommitError::StatementMismatch)),
        "wrong input_hash must produce StatementMismatch, got {:?}", err,
    );
}

// ── zheng verify errors ───────────────────────────────────────────────────────

/// A proof committed under statement A must not verify under a different statement B.
///
/// The statement (program_hash/input_hash/output_hash/focus_bound) is absorbed
/// into the Fiat-Shamir transcript before sumcheck challenges are derived.
/// Mismatching statements cause transcript divergence, which propagates to the
/// Brakedown PCS seed and causes verify() to return LensFailed.
#[test]
fn proof_from_wrong_statement_fails_verify() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = order.atom(Goldilocks::new(0), Tag::Field).unwrap();
    let formula = make_field_binop(&mut order, 5, 2, 3);

    let mut trace = VecTrace::default();
    reduce(&mut order, obj, formula, 1000, &NullCalls, &mut trace);

    let stmt_a = zero_statement();
    let proof  = commit(&trace, &[], &[], &[], &stmt_a, &default_params()).unwrap();

    // Different statement: any non-zero program_hash diverges the transcript.
    let stmt_b = Statement {
        program_hash: [7u8; 32],
        input_hash:   [0u8; 32],
        output_hash:  [0u8; 32],
        focus_bound:  0,
    };
    assert!(
        zheng::verify(&proof, &stmt_b, &default_params()).is_err(),
        "proof committed under stmt_a must not verify under stmt_b",
    );
}

// ── bbg-level errors ──────────────────────────────────────────────────────────

/// Inserting two signals with the same BoxMove nullifier → InsertError::DoubleSpend.
#[test]
fn double_spend_bbg_insert_rejected() {
    let mut state = BbgState::new();
    let n = neuron(1);
    state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });

    let nullifier = particle(42);
    let mk_signal = || BbgSignal {
        neuron:    n,
        links:     vec![],
        box_moves: vec![BoxMove { nullifier, commitment: None }],
        height:    0,
    };

    state.insert(&mk_signal()).expect("first insert must succeed");
    assert_eq!(
        state.insert(&mk_signal()),
        Err(InsertError::DoubleSpend),
        "second insert with same nullifier must fail",
    );
}
