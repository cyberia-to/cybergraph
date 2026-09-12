#![cfg(feature = "local-storage")]
use bbg::storage::database::{Backend, Database, RecordDomain as D, RecordLimits};
use cybergraph::native::{Error, Event, NativeNode, Operation};
use cybergraph::{CyberlinkRecord, IntentRecord, SELF_NETWORK, Signal};
use std::{
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const GENESIS: &[u8] = b"native regression genesis";
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "cybergraph-native-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn db(&self) -> PathBuf {
        self.0.join("bbg")
    }
    fn open(&self) -> NativeNode {
        NativeNode::open(&self.db(), GENESIS).unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn link(neuron: u8, amount: u64) -> Operation {
    Operation::Link {
        neuron: [neuron; 32],
        from: [20; 32],
        to: [21; 32],
        token: [0; 32],
        amount,
        valence: 1,
    }
}
fn reward(neuron: u8, amount: u64) -> Operation {
    Operation::Link {
        neuron: [neuron; 32],
        from: *hemera::hash(b"zheng").as_bytes(),
        to: *hemera::hash(b"pussy").as_bytes(),
        token: [0; 32],
        amount,
        valence: 1,
    }
}
fn signal(neuron: u8) -> Signal {
    Signal {
        neuron: [neuron; 32],
        network: SELF_NETWORK,
        links: vec![],
        delta_pi: vec![],
        box_moves: vec![],
        prev: [0; 32],
        step: 0,
        height: 0,
        proof: None,
    }
}
fn intent() -> IntentRecord {
    IntentRecord {
        neuron: [8; 32],
        h0: 0,
        scope_hash: [2; 32],
        signature: [3; 64],
    }
}

#[test]
fn original_payment_receipt_survives_other_commits_and_restart() {
    let t = Temp::new();
    let mut n = t.open();
    n.accept(Some([1; 32]), reward(1, 100), 10).unwrap();
    let pay = || Operation::Pay {
        from: [1; 32],
        to: [2; 32],
        amount: 30,
    };
    let receipt = n.accept(Some([2; 32]), pay(), 11).unwrap();
    assert_eq!(receipt.balance, Some(70));
    assert_eq!(receipt.height, 2);
    n.accept(Some([3; 32]), pay(), 12).unwrap();
    assert_eq!(n.balance(&[1; 32]), 40);
    assert_eq!(n.accept(Some([2; 32]), pay(), 99).unwrap(), receipt);
    assert!(matches!(
        n.accept(Some([2; 32]), link(1, 1), 99),
        Err(Error::Conflict)
    ));
    let root = n.root();
    drop(n);
    let mut n = t.open();
    assert_eq!(n.root(), root);
    assert_eq!(n.accept(Some([2; 32]), pay(), 100).unwrap(), receipt);
    assert_eq!(
        (n.balance(&[1; 32]), n.balance(&[2; 32]), n.supply()),
        (40, 60, 100)
    );
    assert_eq!(n.height(), 3);
}

#[test]
fn equal_commands_without_identity_are_distinct_operations() {
    let t = Temp::new();
    let mut n = t.open();
    let a = n.accept(None, link(1, 3), 1).unwrap();
    let b = n.accept(None, link(1, 3), 1).unwrap();
    assert_ne!(a.request_id, b.request_id);
    assert_eq!(b.height, 2);
    assert_eq!(n.accept(Some(a.request_id), link(1, 3), 9).unwrap(), a);
}

#[test]
fn multi_neuron_steps_have_distinct_global_records() {
    let t = Temp::new();
    let mut n = t.open();
    n.accept(None, link(1, 1), 1).unwrap();
    n.accept(None, link(2, 1), 2).unwrap();
    assert_eq!(n.graph().bbg.state.signals.len(), 2);
    assert_eq!(n.graph().bbg.state.signals[&0].neuron, [1; 32]);
    assert_eq!(n.graph().bbg.state.signals[&1].neuron, [2; 32]);
    let root = n.root();
    drop(n);
    assert_eq!(t.open().root(), root);
}

#[test]
fn rejected_batch_restores_graph_ledger_chain_and_disk() {
    let t = Temp::new();
    let mut n = t.open();
    let root = n.root();
    let mut a = signal(1);
    a.links.push(CyberlinkRecord {
        neuron: [1; 32],
        from: *hemera::hash(b"zheng").as_bytes(),
        to: *hemera::hash(b"pussy").as_bytes(),
        token: [0; 32],
        amount: 99,
        valence: 1,
        height: 0,
    });
    a.box_moves.push(foculus::BoxMoveRecord {
        nullifier: [8; 32],
        commitment: None,
    });
    let mut b = signal(2);
    b.box_moves.push(foculus::BoxMoveRecord {
        nullifier: [8; 32],
        commitment: None,
    });
    assert!(
        n.accept(
            Some([1; 32]),
            Operation::Events(vec![Event::Signal(a), Event::Signal(b)]),
            1
        )
        .is_err()
    );
    assert_eq!(n.height(), 0);
    assert_eq!(n.root(), root);
    assert_eq!(n.supply(), 0);
    assert!(n.graph().chains.is_empty());
    assert!(n.history(None, 64).unwrap().is_empty());
    drop(n);
    let mut n = t.open();
    assert_eq!(n.root(), root);
    n.accept(Some([1; 32]), link(1, 1), 2).unwrap();
}

#[test]
fn native_rewards_intents_and_networks_are_retained() {
    let t = Temp::new();
    let mut n = t.open();
    let mut s = signal(1);
    s.network = [99; 32];
    s.links.push(CyberlinkRecord {
        neuron: s.neuron,
        from: *hemera::hash(b"zheng").as_bytes(),
        to: *hemera::hash(b"pussy").as_bytes(),
        token: [0; 32],
        amount: 7,
        valence: 1,
        height: 42,
    });
    s.height = 42;
    s.delta_pi.push(([2; 32], 3));
    n.accept(
        Some([8; 32]),
        Operation::Events(vec![Event::Intent(intent()), Event::Signal(s)]),
        123,
    )
    .unwrap();
    assert_eq!((n.balance(&[1; 32]), n.balance(&[2; 32])), (4, 3));
    assert_eq!(n.graph().bbg.state.signals[&0].network, [99; 32]);
    assert_eq!(n.block(1).unwrap().unwrap().signal.height, 42);
    let root = n.root();
    drop(n);
    let n = t.open();
    assert_eq!(n.root(), root);
    assert_eq!(n.graph().bbg.state.intents.len(), 1);
    assert_eq!(n.supply(), 7);
    assert_eq!(n.history(None, 64).unwrap().len(), 1);
    let prefix = n.wire_log(0, 4096).unwrap();
    assert!(!prefix.is_empty());
    assert!(matches!(
        n.wire_log(prefix.len(), 4096),
        Err(Error::Unsupported(_))
    ));
}

#[test]
fn economics_overflow_never_publishes_a_partial_batch() {
    let t = Temp::new();
    let mut n = t.open();
    n.accept(None, reward(1, u64::MAX), 1).unwrap();
    let root = n.root();
    assert!(n.accept(None, reward(2, 1), 2).is_err());
    assert_eq!(n.root(), root);
    assert_eq!(n.supply(), u64::MAX);
    let self_pay = Operation::Pay {
        from: [1; 32],
        to: [1; 32],
        amount: u64::MAX,
    };
    n.accept(None, self_pay, 3).unwrap();
    assert_eq!(n.balance(&[1; 32]), u64::MAX);
    drop(n);
    assert_eq!(t.open().balance(&[1; 32]), u64::MAX);
}

#[test]
fn legacy_import_keeps_bytes_and_requires_complete_input() {
    let t = Temp::new();
    let mut a = signal(1);
    a.links.push(CyberlinkRecord {
        neuron: a.neuron,
        from: [1; 32],
        to: [2; 32],
        token: [0; 32],
        amount: 4,
        valence: 1,
        height: 0,
    });
    let mut log = foculus::encode_signal_frame(&a);
    let first = log.len();
    log.extend(foculus::encode_intent_frame(&intent()));
    assert!(NativeNode::import_legacy(&t.db(), GENESIS, &log[..log.len() - 1]).is_err());
    assert!(!t.db().exists());
    let report = NativeNode::import_legacy(&t.db(), GENESIS, &log).unwrap();
    assert_eq!((report.events, report.signals), (2, 1));
    let mut n = t.open();
    assert!(n.legacy_source_matches(&log).unwrap());
    assert_eq!(n.wire_log(0, first).unwrap(), log[..first]);
    assert_eq!(n.wire_log(first, 4096).unwrap(), log[first..]);
    assert!(n.wire_log(1, 4096).is_err());
    assert!(n.wire_log(0, first - 1).is_err());
    n.accept(None, link(2, 1), 12).unwrap();
    let exported = n.wire_log(0, 8192).unwrap();
    assert!(exported.starts_with(&log));
    drop(n);
    assert_eq!(t.open().wire_log(0, 8192).unwrap(), exported);
}

#[test]
fn changed_genesis_or_corrupt_persisted_state_refuses_readiness() {
    for domain in [
        D::NativeState,
        D::NativeBalances,
        D::NativeBlocks,
        D::NativeRequests,
        D::NativeHistory,
        D::NativeExport,
    ] {
        let t = Temp::new();
        let mut n = t.open();
        n.accept(Some([1; 32]), reward(1, 9), 1).unwrap();
        drop(n);
        assert!(NativeNode::open(&t.db(), b"different genesis").is_err());
        let db = Database::open(t.db(), Backend::Ssd).unwrap();
        let rows = db
            .scan_records(
                domain,
                None,
                b"",
                RecordLimits {
                    max_entries: 1,
                    max_bytes: 1024 * 1024,
                },
            )
            .unwrap();
        let key = rows[0].0.clone();
        db.transaction::<_, bbg::storage::StorageError>(|tx| {
            tx.put_record(domain, &key, b"corrupt")
        })
        .unwrap();
        drop(db);
        assert!(
            NativeNode::open(&t.db(), GENESIS).is_err(),
            "accepted corruption in {domain:?}"
        );
    }
}

#[test]
fn extra_or_missing_records_are_not_silently_skipped() {
    for extra in [false, true] {
        let t = Temp::new();
        let mut n = t.open();
        n.accept(Some([1; 32]), link(1, 2), 1).unwrap();
        drop(n);
        let db = Database::open(t.db(), Backend::Ssd).unwrap();
        db.transaction::<_, bbg::storage::StorageError>(|tx| {
            if extra {
                tx.put_record(D::NativeRequests, &[77; 32], b"extra")
            } else {
                tx.remove_record(D::NativeState, &[0])
            }
        })
        .unwrap();
        drop(db);
        assert!(NativeNode::open(&t.db(), GENESIS).is_err());
    }
}

#[test]
fn authenticated_proof_bytes_and_commitment_survive_durable_recovery() {
    let t = Temp::new();
    let statement = foculus::PayStatement {
        content_id: [71; 32],
        total_out: 100,
        leg_count: 1,
    };
    let mut signal = signal(1);
    signal.network = [88; 32];
    signal.proof = Some(foculus::prove_pay(&statement).unwrap());
    let original = foculus::signal_codec::encode_signal(&signal).unwrap();
    let proof_hash = foculus::signal_codec::proof_hash(&signal).unwrap();
    let mut node = t.open();
    let receipt = node
        .accept(
            Some([45; 32]),
            Operation::Events(vec![Event::Signal(signal)]),
            9,
        )
        .unwrap();
    assert_eq!(node.graph().bbg.state.signals[&0].proof_hash, proof_hash);
    drop(node);
    let node = t.open();
    assert_eq!(node.root(), receipt.root);
    let block = node.block(1).unwrap().unwrap();
    assert_eq!(
        foculus::signal_codec::encode_signal(&block.signal).unwrap(),
        original
    );
    assert!(foculus::verify_pay(
        block.signal.proof.as_ref().unwrap(),
        &statement
    ));
    assert_eq!(node.graph().bbg.state.signals[&0].proof_hash, proof_hash);
}
