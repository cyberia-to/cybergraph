#![cfg(feature = "local-storage")]
use bbg::storage::{
    StorageError,
    database::{Backend, Database, RecordDomain as D, RecordLimits},
};
use cybergraph::{
    IntentRecord,
    native::{Error, Event, NativeNode, Operation},
};

const GENESIS: &[u8] = b"native format regression";
const DOMAINS: [D; 7] = [
    D::NativeState,
    D::NativeHistory,
    D::NativeRequests,
    D::NativeMetadata,
    D::NativeBalances,
    D::NativeBlocks,
    D::NativeExport,
];

fn reward() -> Operation {
    Operation::Link {
        neuron: [1; 32],
        from: *hemera::hash(b"zheng").as_bytes(),
        to: *hemera::hash(b"pussy").as_bytes(),
        token: [0; 32],
        amount: 25,
        valence: 1,
    }
}
fn intent() -> Operation {
    Operation::Events(vec![Event::Intent(IntentRecord {
        neuron: [1; 32],
        h0: 0,
        scope_hash: [2; 32],
        signature: [3; 64],
    })])
}
fn metadata(db: &Database) -> Vec<u8> {
    db.read_record(D::NativeState, &[0], 256).unwrap().unwrap()
}
fn replace(db: &Database, bytes: &[u8]) {
    db.transaction::<_, StorageError>(|tx| tx.put_record(D::NativeState, &[0], bytes))
        .unwrap();
}
fn legacy(db: &Database) {
    let mut bytes = metadata(db);
    bytes[..4].copy_from_slice(&1u32.to_le_bytes());
    replace(db, &bytes);
}
fn snapshot(db: &Database) -> Vec<Vec<(Vec<u8>, Vec<u8>)>> {
    DOMAINS
        .iter()
        .map(|&domain| {
            db.scan_records(
                domain,
                None,
                b"",
                RecordLimits {
                    max_entries: 256,
                    max_bytes: 1 << 20,
                },
            )
            .unwrap()
        })
        .collect()
}

#[test]
fn compatible_legacy_open_and_retry_are_read_only_then_rootless_write_upgrades_atomically() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bbg");
    let mut node = NativeNode::open(&path, GENESIS).unwrap();
    let receipt = node.accept(Some([7; 32]), reward(), 9).unwrap();
    let db = node.database();
    legacy(&db);
    drop(node);
    let before = snapshot(&db);
    let transaction = db.last_transaction().unwrap();
    let mut node = NativeNode::from_database(db.clone(), GENESIS).unwrap();
    assert_eq!(node.accept(Some([7; 32]), reward(), 99).unwrap(), receipt);
    assert_eq!(node.root(), receipt.root);
    assert_eq!((node.balance(&[1; 32]), node.supply()), (25, 25));
    assert_eq!(snapshot(&db), before);
    assert_eq!(db.last_transaction().unwrap(), transaction);
    assert!(
        node.accept(
            Some([8; 32]),
            Operation::Pay {
                from: [1; 32],
                to: [2; 32],
                amount: 26
            },
            10
        )
        .is_err()
    );
    assert_eq!(snapshot(&db), before);
    let new = node.accept(Some([8; 32]), intent(), 11).unwrap();
    assert_eq!(new.root, receipt.root);
    assert_eq!(new.height, receipt.height);
    assert_eq!(&metadata(&db)[..4], &2u32.to_le_bytes());
    for (domain, rows) in DOMAINS.iter().zip(&before) {
        if matches!(
            domain,
            D::NativeHistory | D::NativeRequests | D::NativeBlocks
        ) {
            for (key, value) in rows {
                assert_eq!(
                    db.read_record(*domain, key, 1 << 20).unwrap().as_ref(),
                    Some(value)
                );
            }
        }
    }
    drop(node);
    drop(db);
    let mut node = NativeNode::open(&path, GENESIS).unwrap();
    assert_eq!(node.accept(Some([7; 32]), reward(), 100).unwrap(), receipt);
    assert_eq!(node.accept(Some([8; 32]), intent(), 100).unwrap(), new);
}

#[test]
fn unknown_malformed_and_incompatible_metadata_leave_all_records_unchanged() {
    for case in 0..6 {
        let directory = tempfile::tempdir().unwrap();
        let mut node = NativeNode::open(&directory.path().join("bbg"), GENESIS).unwrap();
        node.accept(Some([7; 32]), reward(), 9).unwrap();
        let db = node.database();
        drop(node);
        let mut bytes = metadata(&db);
        match case {
            0 => bytes[..4].copy_from_slice(&9u32.to_le_bytes()),
            1 => bytes.truncate(3),
            2 => bytes[84] = 2,
            3 => {
                bytes[..4].copy_from_slice(&1u32.to_le_bytes());
                bytes[12] ^= 1;
            }
            4 => bytes[12] ^= 1,
            _ => {
                bytes[..4].copy_from_slice(&1u32.to_le_bytes());
                db.transaction::<_, StorageError>(|tx| {
                    tx.remove_record(D::NativeHistory, &1u64.to_be_bytes())
                })
                .unwrap();
            }
        }
        replace(&db, &bytes);
        let before = snapshot(&db);
        let transaction = db.last_transaction().unwrap();
        let error = NativeNode::from_database(db.clone(), GENESIS)
            .err()
            .expect("accepted incompatible store");
        match case {
            0 => assert!(error.to_string().contains("native state format 9")),
            3 | 5 => assert!(error.to_string().contains("legacy native state format 1")),
            _ => assert!(matches!(error, Error::Corrupt(_))),
        }
        assert_eq!(snapshot(&db), before);
        assert_eq!(db.last_transaction().unwrap(), transaction);
    }
}

#[test]
fn metadata_changed_after_recovery_fences_the_writer_without_partial_publication() {
    let directory = tempfile::tempdir().unwrap();
    let db = Database::open(directory.path().join("bbg"), Backend::Ssd).unwrap();
    let mut node = NativeNode::from_database(db.clone(), GENESIS).unwrap();
    let root = node.root();
    let mut bytes = metadata(&db);
    bytes[93] ^= 1; // Pruning policy is outside the root, but controls future state.
    replace(&db, &bytes);
    let before = snapshot(&db);
    let error = node.accept(Some([7; 32]), reward(), 1).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("metadata changed outside its coordinator")
    );
    assert_eq!(node.root(), root);
    assert_eq!(node.height(), 0);
    assert_eq!(node.supply(), 0);
    assert_eq!(snapshot(&db), before);
}
