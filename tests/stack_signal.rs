// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: cybergraph signal lifecycle → BBG state → zheng proof.
//!
//! These tests exercise the full coordinator path:
//!   SignalChain           — layer-2 ordering: hash chain + equivocation detection
//!   VDF                   — timing primitive derived from signal hash
//!   Signal → BbgState     — conversion and insertion into authenticated state
//!   BbgState → look proof — ProofLookProvider + zheng commit/verify
//!
//! The cybergraph layer owns signal ordering (L2); BBG owns authenticated state (L3).
//! Tests here verify they compose correctly end-to-end.

mod common;

use common::{bbg_object_from_state, default_params, make_look_formula, zero_statement};

use cybergraph::{Signal, SignalChain, SELF_NETWORK, vdf_evaluate, vdf_verify, challenge_from_hash};
use bbg::{BbgState, Signal as BbgSignal, Cyberlink as BbgCyberlink, Particle, NeuronId};
use bbg::types::NeuronRecord;
use bbg::ProofLookProvider;
use nox::{reduce, Order, VecTrace, Outcome};
use zheng::{commit, verify};

const ORDER_SIZE: usize = 1024;

fn neuron(seed: u8) -> NeuronId  { [seed; 32] }
fn particle(seed: u8) -> Particle { [seed; 32] }

fn make_signal(n: NeuronId, step: u64, prev: Particle) -> Signal {
    Signal { neuron: n, network: SELF_NETWORK, links: vec![], delta_pi: vec![], prev, step, height: 0, proof: None }
}

// ── signal chain ordering ─────────────────────────────────────────────────────

/// Three sequential signals append without error; wrong step and prev hash are rejected.
#[test]
fn signal_chain_ordering_and_rejection() {
    let n = neuron(1);
    let mut chain = SignalChain::new();

    let s0 = make_signal(n, 0, [0u8; 32]);
    let h0 = s0.hash();
    chain.append(s0).unwrap();

    let s1 = make_signal(n, 1, h0);
    let h1 = s1.hash();
    chain.append(s1).unwrap();

    let s2 = make_signal(n, 2, h1);
    chain.append(s2).unwrap();

    assert_eq!(chain.entries.len(), 3);

    // Out-of-sequence step.
    assert!(chain.append(make_signal(n, 5, [0u8; 32])).is_err(), "wrong step must fail");
    // Correct step but wrong prev hash.
    assert!(chain.append(make_signal(n, 3, [0xffu8; 32])).is_err(), "wrong prev must fail");
    // Equivocation: same (neuron, step, prev) as an existing entry.
    assert!(chain.append(make_signal(n, 0, [0u8; 32])).is_err(), "equivocation must fail");
}

// ── VDF ───────────────────────────────────────────────────────────────────────

/// VDF challenge derived from a signal hash → evaluate → verify roundtrip.
#[test]
fn vdf_challenge_from_signal_hash_roundtrip() {
    let n  = neuron(1);
    let s0 = make_signal(n, 0, [0u8; 32]);
    let hash = s0.hash();

    let challenge = challenge_from_hash(&hash);
    assert_ne!(challenge, 0, "challenge must be non-zero");

    let proof = vdf_evaluate(challenge, 500);
    assert!(vdf_verify(&proof), "VDF proof must verify");
    assert_ne!(proof.output, challenge, "VDF must advance the value");

    // Sequential property: half+half equals full.
    let half   = vdf_evaluate(challenge, 250);
    let second = vdf_evaluate(half.output, 250);
    assert_eq!(second.output, proof.output, "VDF must be sequentially composable");
}

/// Two independent inputs produce different VDF outputs (non-trivial function).
#[test]
fn vdf_different_challenges_produce_different_outputs() {
    let p1 = vdf_evaluate(challenge_from_hash(&make_signal(neuron(1), 0, [0u8; 32]).hash()), 100);
    let p2 = vdf_evaluate(challenge_from_hash(&make_signal(neuron(2), 0, [0u8; 32]).hash()), 100);
    assert_ne!(p1.output, p2.output, "different neurons → different VDF outputs");
}

// ── signal → BbgState → look → zheng proof ───────────────────────────────────

/// Full path: cybergraph Signal → BbgState insert → look(Time) → zheng commit → verify.
///
/// Exercises the boundary between the cybergraph coordinator (L2) and BBG state (L3):
/// signals are ordered by SignalChain, converted to bbg::Signal, inserted into BbgState,
/// and the resulting authenticated state is queried via the nox look pattern to produce
/// a full zheng proof.
#[test]
fn signal_to_bbg_state_to_look_proof() {
    // Layer 2: build a signal chain.
    let n   = neuron(1);
    let mut chain = SignalChain::new();
    let s0  = make_signal(n, 0, [0u8; 32]);
    chain.append(s0).unwrap();
    assert_eq!(chain.entries.len(), 1);

    // Layer 3: insert into BBG state (conversion from cybergraph::Signal to bbg::Signal).
    let mut state = BbgState::new();
    state.neurons.insert(n, NeuronRecord { focus: 100_000, karma: 0, stake: 0 });
    state.insert(&BbgSignal {
        neuron:    n,
        links:     vec![BbgCyberlink {
            from: particle(2), to: particle(3), token: particle(0), amount: 1, valence: 1,
        }],
        box_moves: vec![],
        height:    0,
    }).unwrap();
    // Add a time snapshot so the look(Time, 0) has something to find.
    state.time.insert(0, particle(42));

    // Look pattern: execute look(Time, height=0) with ProofLookProvider.
    let prov = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let obj     = bbg_object_from_state(&mut order, &state);
    let formula = make_look_formula(&mut order, 8, 0); // Dim::Time = 8, height = 0

    let mut trace = VecTrace::default();
    let outcome = reduce(&mut order, obj, formula, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "look must succeed");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1);

    // Generate and verify the zheng proof.
    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params())
        .expect("signal→bbg→look proof must verify");
}

/// Multiple signals from the same neuron → all inserted into BbgState → state is consistent.
#[test]
fn multiple_signals_same_neuron_bbg_state_consistent() {
    let n   = neuron(1);
    let mut chain = SignalChain::new();

    let s0   = make_signal(n, 0, [0u8; 32]);
    let h0   = s0.hash();
    let s1   = make_signal(n, 1, h0);

    chain.append(s0).unwrap();
    chain.append(s1).unwrap();

    let mut state = BbgState::new();
    state.neurons.insert(n, NeuronRecord { focus: 200_000, karma: 0, stake: 0 });

    // Insert both as bbg signals.
    for step in 0u64..=1 {
        state.insert(&BbgSignal {
            neuron:    n,
            links:     vec![BbgCyberlink {
                from:    particle(2 + step as u8),
                to:      particle(10 + step as u8),
                token:   particle(0),
                amount:  1,
                valence: 1,
            }],
            box_moves: vec![],
            height:    step,
        }).unwrap();
    }
    // Particles for both links should be present.
    assert!(state.particles.contains_key(&bbg::state::axon_id(&particle(2), &particle(10))));
    assert!(state.particles.contains_key(&bbg::state::axon_id(&particle(3), &particle(11))));
}
