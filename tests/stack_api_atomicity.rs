//! In-memory Cybergraph link/seal rejection preserves both the chain and BBG.
//! Durable NativeNode/GraphSession acceptance has a separate storage contract.
use cybergraph::{
    ApiError, Cybergraph, CyberlinkRecord, Filter, Intent, SELF_NETWORK, Scope, Signal,
};
use foculus::BoxMoveRecord;
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

fn signal(neuron: [u8; 32], step: u64, prev: [u8; 32], nullifiers: &[u8]) -> Signal {
    Signal {
        neuron,
        network: SELF_NETWORK,
        links: vec![],
        delta_pi: vec![],
        box_moves: nullifiers
            .iter()
            .map(|&v| BoxMoveRecord {
                nullifier: [v; 32],
                commitment: Some(([v.wrapping_add(1); 32], 7)),
            })
            .collect(),
        prev,
        step,
        height: 0,
        proof: None,
    }
}
fn link(neuron: [u8; 32], to: u8, amount: u64) -> CyberlinkRecord {
    CyberlinkRecord {
        neuron,
        from: [8; 32],
        to: [to; 32],
        token: [0; 32],
        amount,
        valence: 1,
        height: 0,
    }
}
fn setup() -> (Cybergraph, Arc<AtomicUsize>, Signal) {
    let mut graph = Cybergraph::new();
    let events = Arc::new(AtomicUsize::new(0));
    let counter = events.clone();
    graph.subscribe(Filter::All, move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    });
    let first = signal([1; 32], 0, [0; 32], &[11]);
    graph.link(first.clone()).unwrap();
    (graph, events, first)
}
fn snapshot(
    graph: &Cybergraph,
    events: &AtomicUsize,
) -> ([u8; 32], Vec<([u8; 32], Vec<(u64, [u8; 32])>)>, usize) {
    (
        graph.bbg.state.root(),
        graph
            .chains
            .iter()
            .map(|(neuron, chain)| {
                (
                    *neuron,
                    chain
                        .entries
                        .iter()
                        .map(|(step, s)| (*step, s.hash()))
                        .collect(),
                )
            })
            .collect(),
        events.load(Ordering::SeqCst),
    )
}

#[test]
fn rejected_spend_preserves_existing_chain_and_allows_same_step_repair() {
    let (mut graph, events, first) = setup();
    let before = snapshot(&graph, &events);
    let mut bad = signal(first.neuron, 1, first.hash(), &[22, 11]);
    bad.links.push(link(first.neuron, 9, 3));
    assert!(matches!(
        graph.link(bad),
        Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend))
    ));
    assert_eq!(snapshot(&graph, &events), before);
    assert!(!graph.bbg.state.nullifiers.contains(&[22; 32]));
    graph
        .link(signal(first.neuron, 1, first.hash(), &[22]))
        .unwrap();
    assert_eq!(graph.chains[&first.neuron].entries.len(), 2);
}

#[test]
fn rejected_spend_does_not_create_an_empty_or_advanced_new_subject_chain() {
    let (mut graph, events, _) = setup();
    let before = snapshot(&graph, &events);
    assert!(matches!(
        graph.link(signal([2; 32], 0, [0; 32], &[11])),
        Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend))
    ));
    assert_eq!(snapshot(&graph, &events), before);
    assert!(!graph.chains.contains_key(&[2; 32]));
    graph.link(signal([2; 32], 0, [0; 32], &[33])).unwrap();
}

#[test]
fn duplicate_and_bad_lineage_leave_graph_and_chain_unchanged() {
    let (mut graph, events, first) = setup();
    for mut bad in [
        first.clone(),
        signal(first.neuron, 5, first.hash(), &[44]),
        signal(first.neuron, 1, [9; 32], &[44]),
        signal([2; 32], 1, [0; 32], &[44]),
    ] {
        bad.links.push(link(first.neuron, 9, 3));
        let before = snapshot(&graph, &events);
        assert!(matches!(graph.link(bad), Err(ApiError::SyncRejected(_))));
        assert_eq!(snapshot(&graph, &events), before);
    }
}

#[test]
fn repeated_nullifier_inside_one_signal_is_rejected_atomically() {
    let (mut graph, events, first) = setup();
    let before = snapshot(&graph, &events);
    assert!(matches!(
        graph.link(signal(first.neuron, 1, first.hash(), &[44, 44])),
        Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend))
    ));
    assert_eq!(snapshot(&graph, &events), before);
}

#[test]
fn overflowing_batch_amount_is_rejected_before_any_public_or_private_mutation() {
    let (mut graph, events, first) = setup();
    graph
        .bbg
        .state
        .balances
        .insert(bbg::balance_key(&[9; 32], &[0; 32]), u64::MAX);
    graph.bbg.state.refresh_root();
    let before = snapshot(&graph, &events);
    let mut bad = signal(first.neuron, 1, first.hash(), &[44]);
    bad.links = vec![link(first.neuron, 7, 3), link(first.neuron, 9, 1)];
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| graph.link(bad)));
    assert!(
        result.is_ok(),
        "unrepresentable amount must be a rejected operation, not a panic"
    );
    assert!(matches!(result.unwrap(), Err(ApiError::AmountOverflow)));
    assert_eq!(snapshot(&graph, &events), before);
    assert!(!graph.bbg.state.nullifiers.contains(&[44; 32]));
}

#[test]
fn rejected_seal_preserves_intent_chain_state_and_events() {
    let (mut graph, events, first) = setup();
    let intent = graph
        .intend(Intent {
            neuron: first.neuron,
            h0: 0,
            signature: [0; 64],
            scope: Scope {
                target: [7; 32],
                predicate: vec![],
                deadline: None,
                constraints: vec![],
            },
        })
        .unwrap();
    let before = snapshot(&graph, &events);
    assert!(matches!(
        graph.seal(intent, signal(first.neuron, 1, first.hash(), &[11])),
        Err(ApiError::BbgRejected(bbg::InsertError::DoubleSpend))
    ));
    assert_eq!(snapshot(&graph, &events), before);
    assert!(graph.bbg.state.intents.contains_key(&intent));
    graph
        .seal(intent, signal(first.neuron, 1, first.hash(), &[55]))
        .unwrap();
}

#[test]
fn amount_preflight_preserves_ordered_debits_and_u64_boundary_values() {
    let (mut graph, events, first) = setup();
    let key7 = bbg::balance_key(&[7; 32], &[0; 32]);
    let key8 = bbg::balance_key(&[8; 32], &[0; 32]);
    graph.bbg.state.balances.insert(key7, u64::MAX);
    graph.bbg.state.refresh_root();
    let mut next = signal(first.neuron, 1, first.hash(), &[]);
    let mut outbound = link(first.neuron, 8, 5);
    outbound.from = [7; 32];
    next.links = vec![outbound, link(first.neuron, 7, 5), link(first.neuron, 9, 0)];
    graph.link(next.clone()).unwrap();
    assert_eq!(graph.bbg.state.balances[&key7], u64::MAX);
    assert_eq!(graph.bbg.state.balances[&key8], 0);
    assert_eq!(graph.chains[&first.neuron].entries.len(), 2);
    let before = snapshot(&graph, &events);
    let mut overflow = signal(first.neuron, 2, next.hash(), &[]);
    overflow.links.push(link(first.neuron, 7, 1));
    assert!(matches!(
        graph.link(overflow),
        Err(ApiError::AmountOverflow)
    ));
    assert_eq!(snapshot(&graph, &events), before);
}

#[test]
fn wrong_network_cannot_install_a_subject_chain_or_touch_bbg() {
    let mut graph = Cybergraph::serving([3; 32]);
    let events = AtomicUsize::new(0);
    let before = snapshot(&graph, &events);
    let mut wrong = signal([1; 32], 0, [0; 32], &[11]);
    wrong.network = [4; 32];
    wrong.links.push(link(wrong.neuron, 9, 1));
    assert!(matches!(
        graph.link(wrong),
        Err(ApiError::WrongNetwork { .. })
    ));
    assert_eq!(snapshot(&graph, &events), before);
}
