#![cfg(feature = "local-storage")]

use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use bbg::storage::{
    StorageError,
    database::{Backend, Database, RecordDomain as D},
};
use cybergraph::{
    SELF_NETWORK, Signal,
    native::{Event, NativeNode, Operation},
};

const GENESIS: &[u8] = b"native-import-test-genesis";
static NEXT: AtomicU64 = AtomicU64::new(0);

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "native-import-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
    fn db(&self) -> PathBuf {
        self.0.join("bbg")
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn signal(id: u8) -> Signal {
    Signal {
        neuron: [id; 32],
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

fn tape(signals: &[Signal]) -> Vec<u8> {
    signals
        .iter()
        .flat_map(foculus::encode_signal_frame)
        .collect()
}

fn historical_frame(signal: &Signal) -> Vec<u8> {
    let mut frame = foculus::encode_signal_frame(signal);
    assert_eq!(frame.len(), 96);
    frame.truncate(frame.len() - 8); // Historical omission of empty optional suffixes.
    frame[3] -= 8;
    frame
}

fn source(log: &[u8]) -> [u8; 32] {
    let mut hash = hemera::Hasher::new();
    hash.update(b"cybergraph/legacy-source/v1\0");
    hash.update(log);
    *hash.finalize().as_bytes()
}

fn digest(domain: &[u8], parts: &[&[u8]]) -> [u8; 32] {
    let mut hash = hemera::Hasher::new();
    hash.update(domain);
    for part in parts {
        hash.update(&(part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    *hash.finalize().as_bytes()
}

fn marker(complete: bool, source: &[u8; 32], events: u64) -> Vec<u8> {
    let mut bytes = b"CGIMPORT\x01".to_vec();
    bytes.push(u8::from(complete));
    bytes.extend(source);
    bytes.extend(events.to_le_bytes());
    bytes
}

fn set_marker(path: &Path, value: &[u8]) {
    let db = Database::open(path, Backend::Ssd).unwrap();
    db.transaction::<_, StorageError>(|tx| tx.put_record(D::NativeMetadata, b"import", value))
        .unwrap();
}

#[test]
fn completed_marker_source_and_event_count_are_checked_against_history() {
    for change_source in [false, true] {
        let root = Temp::new();
        let log = tape(&[signal(1)]);
        NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
        let mut digest = source(&log);
        if change_source {
            digest[0] ^= 1;
        }
        set_marker(
            &root.db(),
            &marker(true, &digest, if change_source { 1 } else { 2 }),
        );
        assert!(NativeNode::open(&root.db(), GENESIS).is_err());
        assert!(NativeNode::import_legacy(&root.db(), GENESIS, &log).is_err());
    }
}

#[test]
fn copied_completion_marker_cannot_claim_an_empty_store_imported_a_log() {
    let root = Temp::new();
    let log = tape(&[signal(1)]);
    drop(NativeNode::open(&root.db(), GENESIS).unwrap());
    set_marker(&root.db(), &marker(true, &source(&log), 1));
    assert!(NativeNode::open(&root.db(), GENESIS).is_err());
    assert!(NativeNode::import_legacy(&root.db(), GENESIS, &log).is_err());
}

#[test]
fn imported_history_requires_its_source_marker() {
    let root = Temp::new();
    let log = tape(&[signal(1)]);
    NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
    let db = Database::open(root.db(), Backend::Ssd).unwrap();
    db.transaction::<_, StorageError>(|tx| tx.remove_record(D::NativeMetadata, b"import"))
        .unwrap();
    drop(db);
    assert!(NativeNode::open(&root.db(), GENESIS).is_err());
}

#[test]
fn wrong_genesis_import_preserves_readiness_of_existing_empty_store() {
    let root = Temp::new();
    drop(NativeNode::open(&root.db(), GENESIS).unwrap());
    assert!(NativeNode::import_legacy(&root.db(), b"wrong genesis", &tape(&[signal(1)])).is_err());
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    assert_eq!(node.height(), 0);
    drop(node);
    let db = Database::open(root.db(), Backend::Ssd).unwrap();
    assert!(
        db.read_record(D::NativeMetadata, b"import", 128)
            .unwrap()
            .is_none()
    );
}

#[test]
fn unknown_uninitialized_records_are_preserved_without_writing_import_metadata() {
    let root = Temp::new();
    let db = Database::open(root.db(), Backend::Ssd).unwrap();
    db.transaction::<_, StorageError>(|tx| tx.put_record(D::NativeState, b"unknown", b"original"))
        .unwrap();
    drop(db);
    assert!(NativeNode::import_legacy(&root.db(), GENESIS, &tape(&[signal(1)])).is_err());
    let db = Database::open(root.db(), Backend::Ssd).unwrap();
    for key in [b"config".as_slice(), b"head", b"import"] {
        assert!(
            db.read_record(D::NativeMetadata, key, 8192)
                .unwrap()
                .is_none()
        );
    }
    assert_eq!(
        db.read_record(D::NativeState, b"unknown", 32)
            .unwrap()
            .unwrap(),
        b"original"
    );
}

#[test]
fn ordinary_operations_follow_import_without_changing_its_exact_prefix() {
    let root = Temp::new();
    let log = tape(&[signal(1)]);
    NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
    let mut node = NativeNode::open(&root.db(), GENESIS).unwrap();
    node.accept(
        Some([5; 32]),
        Operation::Events(vec![Event::Signal(signal(2))]),
        123,
    )
    .unwrap();
    drop(node);
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    assert_eq!(node.height(), 2);
    assert!(node.legacy_source_matches(&log).unwrap());
    assert!(node.wire_log(0, 4096).unwrap().starts_with(&log));
}

/// Produce the state reached by a committed import batch before a process kill.
/// Its native state/receipts come from the real coordinator; only the import
/// provenance and original-byte field are attached to the known initial batch.
fn interrupted_prefix(path: &Path, full_log: &[u8], prefix: &[Signal], events: u64) {
    let source = source(full_log);
    let id = digest(
        b"cybergraph/import-request/v1\0",
        &[&source, &0u64.to_le_bytes()],
    );
    let mut node = NativeNode::open(path, GENESIS).unwrap();
    node.accept(
        Some(id),
        Operation::Events(prefix.iter().cloned().map(Event::Signal).collect()),
        0,
    )
    .unwrap();
    drop(node);
    let db = Database::open(path, Backend::Ssd).unwrap();
    let key = 1u64.to_be_bytes();
    let mut history = db
        .read_record(D::NativeHistory, &key, 16 * 1024 * 1024)
        .unwrap()
        .unwrap();
    assert_eq!(history.pop(), Some(0)); // Replace absent original_wire.
    history.push(1);
    history.extend((prefix.len() as u32).to_le_bytes());
    for signal in prefix {
        let frame = foculus::encode_signal_frame(signal);
        history.extend((frame.len() as u32).to_le_bytes());
        history.extend(frame);
    }
    let mut head = db
        .read_record(D::NativeMetadata, b"head", 256)
        .unwrap()
        .unwrap();
    let offset = head.len() - 32;
    head[offset..].copy_from_slice(&digest(
        b"cybergraph/native-history/v1\0",
        &[&[0; 32], &history],
    ));
    db.transaction::<_, StorageError>(|tx| {
        tx.put_record(D::NativeHistory, &key, &history)?;
        tx.put_record(D::NativeMetadata, b"head", &head)?;
        tx.put_record(
            D::NativeMetadata,
            b"import",
            &marker(false, &source, events),
        )
    })
    .unwrap();
}

#[test]
fn interrupted_import_resumes_after_exact_committed_frames_without_rebatching_them() {
    let root = Temp::new();
    let signals = vec![signal(1), signal(2), signal(3)];
    let log = tape(&signals);
    interrupted_prefix(&root.db(), &log, &signals[..1], signals.len() as u64);
    assert!(NativeNode::open(&root.db(), GENESIS).is_err());
    let imported = NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
    assert_eq!(imported.events, 3);
    assert_eq!(imported.height, 3);
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    assert_eq!(node.history(None, 64).unwrap().len(), 2);
    assert_eq!(node.wire_log(0, 4096).unwrap(), log);
    drop(node);
    assert_eq!(
        NativeNode::import_legacy(&root.db(), GENESIS, &log)
            .unwrap()
            .height,
        3
    );
}

#[test]
fn interrupted_import_rejects_different_exact_prefix_even_with_valid_chain_and_marker() {
    let root = Temp::new();
    let log = tape(&[signal(1), signal(2)]);
    interrupted_prefix(&root.db(), &log, &[signal(3)], 2);
    assert!(NativeNode::import_legacy(&root.db(), GENESIS, &log).is_err());
    assert!(NativeNode::open(&root.db(), GENESIS).is_err());
}

#[test]
fn import_preserves_historical_bytes_and_resume_compares_representation_exactly() {
    let first = signal(1);
    let old_frame = historical_frame(&first);
    let root = Temp::new();
    NativeNode::import_legacy(&root.db(), GENESIS, &old_frame).unwrap();
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    assert_eq!(node.wire_log(0, 4096).unwrap(), old_frame);
    drop(node);

    let other = Temp::new();
    let mut log = old_frame;
    log.extend(foculus::encode_signal_frame(&signal(2)));
    // Same decoded first event, with a different historical representation.
    interrupted_prefix(&other.db(), &log, &[first], 2);
    assert!(NativeNode::import_legacy(&other.db(), GENESIS, &log).is_err());
}

#[test]
fn aggregate_legacy_log_exceeding_operation_limit_is_split_into_valid_requests() {
    let root = Temp::new();
    let mut first = signal(1);
    first.delta_pi = vec![([3; 32], 0); 53_000];
    let mut second = signal(2);
    second.delta_pi = first.delta_pi.clone();
    let log = tape(&[first, second]);
    assert!(log.len() > cybergraph::native::MAX_OPERATION_BYTES);
    let imported = NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
    assert_eq!(imported.events, 2);
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    assert_eq!(node.history(None, 64).unwrap().len(), 2);
    assert_eq!(node.wire_log(0, 8 * 1024 * 1024).unwrap(), log);
}

#[test]
fn intent_import_respects_event_count_when_byte_budget_has_room() {
    let root = Temp::new();
    let intent = bbg::IntentRecord {
        neuron: [1; 32],
        h0: 0,
        scope_hash: [3; 32],
        signature: [4; 64],
    };
    let frame = foculus::encode_intent_frame(&intent);
    let log = frame.repeat(65);
    let imported = NativeNode::import_legacy(&root.db(), GENESIS, &log).unwrap();
    assert_eq!(imported.events, 65);
    assert_eq!(imported.signals, 0);
    let node = NativeNode::open(&root.db(), GENESIS).unwrap();
    let counts: Vec<_> = node
        .history(None, 64)
        .unwrap()
        .iter()
        .map(|(r, _)| r.applied)
        .collect();
    assert_eq!(counts, [64, 1]);
    assert_eq!(node.wire_log(0, 16_384).unwrap(), log);
}
