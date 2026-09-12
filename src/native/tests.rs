use super::*;
use std::cell::Cell;

thread_local! { static FAIL_AFTER_STAGING:Cell<bool> = const { Cell::new(false) }; }
pub(super) fn after_staging() -> Result<(), Error> {
    if FAIL_AFTER_STAGING.replace(false) {
        return Err(Error::Storage(StorageError::Io(
            "injected staging failure".into(),
        )));
    }
    Ok(())
}

#[test]
fn staged_native_commit_error_restores_every_published_view_and_request_identity() {
    let path = std::env::temp_dir().join(format!("native-staging-failure-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let db = path.join("bbg");
    let mut node = NativeNode::open(&db, b"genesis").unwrap();
    let root = node.root();
    let request = || Operation::Link {
        neuron: [1; 32],
        from: *hemera::hash(b"zheng").as_bytes(),
        to: *hemera::hash(b"pussy").as_bytes(),
        token: [0; 32],
        amount: 10,
        valence: 1,
    };
    FAIL_AFTER_STAGING.set(true);
    assert!(matches!(
        node.accept(Some([9; 32]), request(), 11),
        Err(Error::Storage(StorageError::Io(_)))
    ));
    assert_eq!(node.root(), root);
    assert_eq!(node.height(), 0);
    assert_eq!(node.supply(), 0);
    assert!(node.graph().chains.is_empty());
    assert!(node.history(None, 64).unwrap().is_empty());
    assert!(node.wire_log(0, 1024).unwrap().is_empty());
    assert!(node.block(1).unwrap().is_none());
    assert!(
        node.db
            .read_record(D::NativeRequests, &[9; 32], 256)
            .unwrap()
            .is_none()
    );
    drop(node);
    let mut node = NativeNode::open(&db, b"genesis").unwrap();
    assert_eq!(node.root(), root);
    let receipt = node.accept(Some([9; 32]), request(), 12).unwrap();
    assert_eq!(receipt.height, 1);
    assert_eq!(receipt.supply, 10);
    drop(node);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn missing_live_history_or_block_is_corruption() {
    let path = std::env::temp_dir().join(format!("native-read-failure-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let mut node = NativeNode::open(&path.join("bbg"), b"genesis").unwrap();
    node.accept(
        None,
        Operation::Pay {
            from: [1; 32],
            to: [2; 32],
            amount: 1,
        },
        0,
    )
    .unwrap_err();
    node.accept(
        None,
        Operation::Link {
            neuron: [1; 32],
            from: [2; 32],
            to: [3; 32],
            token: [0; 32],
            amount: 1,
            valence: 1,
        },
        1,
    )
    .unwrap();
    node.db
        .transaction::<_, StorageError>(|tx| {
            tx.remove_record(D::NativeBlocks, &1u64.to_be_bytes())?;
            tx.remove_record(D::NativeHistory, &1u64.to_be_bytes())
        })
        .unwrap();
    assert!(matches!(node.block(1), Err(Error::Corrupt(_))));
    assert!(matches!(node.history(None, 64), Err(Error::Corrupt(_))));
    assert!(node.block(2).unwrap().is_none());
    drop(node);
    std::fs::remove_dir_all(path).unwrap();
}
