// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: bbg::Bbg facade lifecycle → zheng proof.
//!
//! Exercises the full BBG lifecycle:
//!   Bbg::insert()          — signal insertion, state update
//!   Bbg::finalize_block()  — time snapshot, root recompute, height increment, checkpoint advance
//!   BoxMove / nullifier    — double-spend protection
//!   Checkpoint             — root tracking after finalization
//!   ProofLookProvider      — look proof against post-finalize state

mod common;

use common::{bbg_object_from_state, default_params, make_look_formula, zero_statement};

use nox::{reduce, Order, VecTrace, Outcome};
use zheng::{commit, verify};
use bbg::{Bbg, Signal as BbgSignal, Cyberlink as BbgCyberlink, Particle, NeuronId};
use bbg::types::NeuronRecord;
use bbg::{BoxMove, InsertError, ProofLookProvider};

const ORDER_SIZE: usize = 1024;

fn neuron(seed: u8) -> NeuronId  { [seed; 32] }
fn particle(seed: u8) -> Particle { [seed; 32] }

fn one_link(neuron: NeuronId, from: Particle, to: Particle, height: u64) -> BbgSignal {
    BbgSignal {
        neuron,
        links:     vec![BbgCyberlink { from, to, token: particle(0), amount: 1, valence: 1 }],
        box_moves: vec![],
        height,
    }
}

// ── finalize_block ────────────────────────────────────────────────────────────

/// Insert one signal → finalize_block() → time[0] exists → look(Time, 0) succeeds → proof verifies.
///
/// Verifies the full L2→L3 finalization path: after finalize_block(), the time
/// index has a snapshot at the previous height, and a ProofLookProvider built
/// from the post-finalize state can produce a verifiable opening.
#[test]
fn finalize_block_updates_root_and_look_proof_consistent() {
    let mut bbg = Bbg::new();
    let n = neuron(1);
    bbg.state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });
    bbg.insert(&one_link(n, particle(2), particle(3), 0)).unwrap();

    let root_before = bbg.state.root();
    bbg.finalize_block();

    // finalize_block records time[0]=root_before, recomputes root, increments height.
    assert_eq!(bbg.state.height, 1);
    assert_ne!(bbg.state.root(), root_before, "root must change after finalize");
    assert!(bbg.state.time.contains_key(&0), "time[0] must exist after finalize");

    // Build look proof against the post-finalize state.
    let prov = ProofLookProvider::new(&bbg.state);
    let mut order = Order::<ORDER_SIZE>::new();
    let bbg_obj = bbg_object_from_state(&mut order, &bbg.state);
    let formula = make_look_formula(&mut order, 8, 0);  // Dim::Time=8, height=0

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, bbg_obj, formula, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "look(Time, 0) must succeed");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1);

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params())
        .expect("finalize+look proof must verify");
}

// ── box moves ─────────────────────────────────────────────────────────────────

/// First BoxMove succeeds; a second signal reusing the same nullifier → DoubleSpend.
#[test]
fn box_moves_nullifier_double_spend_rejected() {
    let mut bbg = Bbg::new();
    let n = neuron(1);
    bbg.state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });

    let nullifier = particle(77);
    let mk = |commitment| BbgSignal {
        neuron:    n,
        links:     vec![],
        box_moves: vec![BoxMove { nullifier, commitment }],
        height:    0,
    };

    // First spend: nullifier not yet recorded → Ok.
    bbg.insert(&mk(Some((particle(88), 500)))).expect("first box spend must succeed");

    // Second spend: same nullifier → DoubleSpend.
    assert_eq!(
        bbg.insert(&mk(None)),
        Err(InsertError::DoubleSpend),
        "reusing nullifier must fail with DoubleSpend",
    );
}

// ── checkpoint ────────────────────────────────────────────────────────────────

/// After finalize_block(), checkpoint.root matches state.root and height advances.
#[test]
fn checkpoint_tracks_state_after_finalize() {
    let mut bbg = Bbg::new();
    let n = neuron(1);
    bbg.state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });
    bbg.insert(&one_link(n, particle(2), particle(3), 0)).unwrap();

    bbg.finalize_block();

    assert_eq!(bbg.checkpoint.root, bbg.state.root(),
        "checkpoint.root must match state.root after finalize");
    assert_eq!(bbg.checkpoint.height, bbg.state.height,
        "checkpoint.height must match state.height after finalize");
}

// ── multi-block cycle ─────────────────────────────────────────────────────────

/// Two insert+finalize cycles produce time entries at heights 0 and 1.
/// A look proof at time[1] (the second snapshot) verifies.
#[test]
fn multi_block_insert_finalize_cycle_then_look_proof() {
    let mut bbg = Bbg::new();
    let n = neuron(1);
    bbg.state.neurons.insert(n, NeuronRecord { focus: 200_000, karma: 0, stake: 0 });

    // Block 0
    bbg.insert(&one_link(n, particle(2), particle(3), 0)).unwrap();
    bbg.finalize_block();  // time[0] recorded; height → 1

    // Block 1
    bbg.insert(&one_link(n, particle(4), particle(5), 1)).unwrap();
    bbg.finalize_block();  // time[1] recorded; height → 2

    assert_eq!(bbg.state.height, 2);
    assert!(bbg.state.time.contains_key(&0), "time[0] must exist");
    assert!(bbg.state.time.contains_key(&1), "time[1] must exist");

    // look(Time, 1) → proof of the second time snapshot.
    let prov = ProofLookProvider::new(&bbg.state);
    let mut order = Order::<ORDER_SIZE>::new();
    let bbg_obj = bbg_object_from_state(&mut order, &bbg.state);
    let formula = make_look_formula(&mut order, 8, 1);  // Dim::Time=8, height=1

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, bbg_obj, formula, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "look(Time, 1) must succeed");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1);

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params())
        .expect("multi-block lifecycle look proof must verify");
}
