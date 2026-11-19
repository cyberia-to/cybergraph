//! Unit tests of the five verbs over an in-memory cybergraph.
use super::*;
use foculus::CyberlinkRecord;
use std::sync::{Arc, Mutex};

fn n(seed: u8) -> NeuronId {
    [seed; 32]
}
fn p(seed: u8) -> Particle {
    [seed; 32]
}

fn scope(target: Particle) -> Scope {
    Scope {
        target,
        predicate: b"do-thing".to_vec(),
        deadline: Some(100),
        constraints: vec![],
    }
}

fn intent(neuron: NeuronId, h0: u64, target: Particle) -> Intent {
    Intent {
        neuron,
        h0,
        scope: scope(target),
        signature: [0u8; 64],
    }
}

fn empty_signal(neuron: NeuronId, step: u64, prev: Particle) -> Signal {
    Signal {
        neuron,
        network: foculus::SELF_NETWORK,
        links: vec![],
        delta_pi: vec![],
        box_moves: vec![],
        prev,
        step,
        height: 0,
        proof: None,
    }
}

fn one_link_signal(
    neuron: NeuronId,
    step: u64,
    prev: Particle,
    from: Particle,
    to: Particle,
) -> Signal {
    Signal {
        neuron,
        network: foculus::SELF_NETWORK,
        links: vec![CyberlinkRecord {
            neuron,
            from,
            to,
            token: p(0),
            amount: 1,
            valence: 1,
            height: 0,
        }],
        delta_pi: vec![],
        box_moves: vec![],
        prev,
        step,
        height: 0,
        proof: None,
    }
}

#[test]
fn intend_persists_and_returns_key() {
    let mut g = Cybergraph::new();
    let key = g.intend(intent(n(1), 0, p(2))).unwrap();
    assert!(g.bbg.state.intents.contains_key(&key));
}

#[test]
fn seal_unknown_intent_fails() {
    let mut g = Cybergraph::new();
    let s = empty_signal(n(1), 0, [0u8; 32]);
    let err = g.seal(p(99), s);
    assert!(matches!(err, Err(ApiError::UnknownIntent(_))));
}

#[test]
fn intend_then_seal_succeeds() {
    let mut g = Cybergraph::new();
    let key = g.intend(intent(n(1), 0, p(2))).unwrap();
    let s = empty_signal(n(1), 0, [0u8; 32]);
    g.seal(key, s).unwrap();
    assert!(g.bbg.state.signals.contains_key(&bbg::signal_key(&n(1), 0)));
}

#[test]
fn link_atomic_path() {
    let mut g = Cybergraph::new();
    let s = empty_signal(n(1), 0, [0u8; 32]);
    g.link(s).unwrap();
    assert!(g.bbg.state.signals.contains_key(&bbg::signal_key(&n(1), 0)));
}

/// Launch row 9: two neurons' first signals are both step 0. Both headers
/// persist, and the order a node received them in does not move the root.
#[test]
fn first_signals_of_two_neurons_both_persist_in_any_order() {
    let link_both = |order: [u8; 2]| {
        let mut g = Cybergraph::new();
        for i in order {
            g.link(one_link_signal(n(i), 0, [0u8; 32], p(10 + i), p(20)))
                .unwrap();
        }
        g
    };
    let (ab, ba) = (link_both([1, 2]), link_both([2, 1]));
    for g in [&ab, &ba] {
        assert_eq!(g.bbg.state.signals.len(), 2);
        for i in [1, 2] {
            let record = g.bbg.state.signals.get(&bbg::signal_key(&n(i), 0)).unwrap();
            assert_eq!((record.neuron, record.step), (n(i), 0));
        }
    }
    assert_eq!(ab.bbg.state.root(), ba.bbg.state.root());
}

#[test]
fn link_applies_cyberlinks_to_bbg_state() {
    let mut g = Cybergraph::new();
    let s = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
    g.link(s).unwrap();
    // The cyberlink (p2 → p3, amount 1) must reach bbg state, not just the header.
    let target = g
        .bbg
        .state
        .particles
        .get(&p(3))
        .expect("target particle materialized");
    assert_eq!(target.energy, 1, "target energy reflects the staked link");
    assert!(
        g.bbg.state.axons_out.get(&p(2)).is_some(),
        "outgoing axon recorded"
    );
    assert!(
        g.bbg.state.axons_in.get(&p(3)).is_some(),
        "incoming axon recorded"
    );
}

#[test]
fn link_moves_the_root() {
    let mut g = Cybergraph::new();
    let before = g.bbg.state.root();
    g.link(one_link_signal(n(1), 0, [0u8; 32], p(2), p(3)))
        .unwrap();
    assert_ne!(
        g.bbg.state.root(),
        before,
        "applying cyberlinks advances BBG_root"
    );
}

#[test]
fn seal_applies_cyberlinks_to_bbg_state() {
    let mut g = Cybergraph::new();
    let key = g.intend(intent(n(1), 0, p(3))).unwrap();
    let s = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
    g.seal(key, s).unwrap();
    assert_eq!(g.bbg.state.particles.get(&p(3)).unwrap().energy, 1);
}

#[test]
fn link_accumulates_across_signals() {
    let mut g = Cybergraph::new();
    let s0 = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
    let prev = s0.hash(); // chain link for the next signal
    g.link(s0).unwrap();
    let s1 = one_link_signal(n(1), 1, prev, p(2), p(3));
    g.link(s1).unwrap();
    // Two staked links onto p3 accumulate energy.
    assert_eq!(g.bbg.state.particles.get(&p(3)).unwrap().energy, 2);
}

#[test]
fn subscribe_receives_matching_events() {
    let mut g = Cybergraph::new();
    let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
    let log2 = log.clone();
    g.subscribe(Filter::All, move |e| {
        let tag = match e {
            Event::IntentDeclared { .. } => "intent",
            Event::SignalSealed { .. } => "seal",
            Event::Linked { .. } => "link",
        };
        log2.lock().unwrap().push(tag);
    });
    g.intend(intent(n(1), 0, p(2))).unwrap();
    let s = empty_signal(n(1), 0, [0u8; 32]);
    g.link(s).unwrap();
    assert_eq!(*log.lock().unwrap(), vec!["intent", "link"]);
}

#[test]
fn subscribe_by_neuron_filters() {
    let mut g = Cybergraph::new();
    let counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
    let c2 = counter.clone();
    g.subscribe(Filter::ByNeuron(n(1)), move |_| {
        *c2.lock().unwrap() += 1;
    });
    g.link(empty_signal(n(1), 0, [0u8; 32])).unwrap();
    g.link(empty_signal(n(2), 0, [0u8; 32])).unwrap();
    assert_eq!(*counter.lock().unwrap(), 1);
}

#[test]
fn query_runs_inf_over_bbg_state() {
    // Seed two particles via a link, then query the focus relation.
    let mut g = Cybergraph::new();
    g.link(one_link_signal(n(1), 0, [0u8; 32], p(2), p(3)))
        .unwrap();

    // particles relation exposes every materialized particle.
    let out = g
        .query("?[particle, energy] := particles{particle, energy}")
        .expect("query runs");
    assert_eq!(out.columns, vec!["particle", "energy"]);
    // target p3 (energy 1) + axon-particle H(2,3) both materialize.
    assert!(
        !out.rows.is_empty(),
        "query returns the materialized particles"
    );
}

#[test]
fn query_parse_error_is_reported() {
    let g = Cybergraph::new();
    let err = g.query("this is not inf");
    assert!(matches!(err, Err(QueryError::Parse(_))));
}
