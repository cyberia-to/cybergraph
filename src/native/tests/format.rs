use super::*;

#[test]
fn failed_first_write_to_legacy_store_rolls_back_version_and_exact_retry() {
    let backends = [
        Backend::Ssd,
        #[cfg(feature = "legacy-redb-migration")]
        Backend::Hdd,
    ];
    for backend in backends {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("bbg");
        let node =
            NativeNode::from_database(Database::open(&path, backend).unwrap(), b"format fault")
                .unwrap();
        let db = node.database();
        drop(node);
        let mut original = db.read_record(D::NativeState, &[0], 256).unwrap().unwrap();
        original[..4].copy_from_slice(&1u32.to_le_bytes());
        db.transaction::<_, StorageError>(|tx| tx.put_record(D::NativeState, &[0], &original))
            .unwrap();
        let mut node = NativeNode::from_database(db.clone(), b"format fault").unwrap();
        let last = db.last_transaction().unwrap();
        let operation = || {
            Operation::Events(vec![Event::Intent(IntentRecord {
                neuron: [1; 32],
                h0: 0,
                scope_hash: [2; 32],
                signature: [3; 64],
            })])
        };
        FAIL_AFTER_STAGING.set(true);
        assert!(node.accept(Some([4; 32]), operation(), 1).is_err());
        assert_eq!(db.last_transaction().unwrap(), last);
        assert_eq!(
            db.read_record(D::NativeState, &[0], 256).unwrap().unwrap(),
            original
        );
        assert_eq!(node.state_metadata, original);
        assert!(node.history(None, 64).unwrap().is_empty());
        let receipt = node.accept(Some([4; 32]), operation(), 2).unwrap();
        assert_eq!(node.state_metadata[..4], 2u32.to_le_bytes());
        drop(node);
        drop(db);
        let mut node =
            NativeNode::from_database(Database::open(&path, backend).unwrap(), b"format fault")
                .unwrap();
        assert_eq!(
            node.accept(Some([4; 32]), operation(), 99).unwrap(),
            receipt
        );
    }
}
