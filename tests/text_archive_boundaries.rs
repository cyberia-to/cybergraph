#![cfg(feature = "local-storage")]

use cybergraph::{
    application::{ApplicationGraph, Backend, Database},
    text_archive::{MAX_PROJECTION_BYTES, MAX_TEXT_BYTES, TextArchive},
};
use std::sync::{Arc, Barrier};

const NS: [u8; 32] = [41; 32];

fn archive(db: &Database) -> TextArchive {
    TextArchive::from_database(db.clone(), NS)
}

fn line(text: &str, created: Option<u64>) -> Vec<u8> {
    let particle = hemera::hash(text.as_bytes())
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut bytes = serde_json::to_vec(&serde_json::json!({
        "particle": particle, "text": text, "created": created,
    }))
    .unwrap();
    bytes.push(b'\n');
    bytes
}

#[test]
fn maximum_utf8_text_survives_reopen_and_oversize_changes_nothing() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("db");
    let text = "🧠".repeat(MAX_TEXT_BYTES / 4);
    assert_eq!(text.len(), MAX_TEXT_BYTES);
    let receipt = {
        let db = Database::open(&path, Backend::Ssd).unwrap();
        let archive = archive(&db);
        let receipt = archive.remember([1; 32], &text, Some(u64::MAX)).unwrap();
        assert!(
            archive
                .remember([2; 32], &(text.clone() + "!"), None)
                .is_err()
        );
        assert_eq!(
            ApplicationGraph::from_database(db).head(&NS).unwrap(),
            Some(receipt)
        );
        receipt
    };
    let db = Database::open(path, Backend::Ssd).unwrap();
    let archive = archive(&db);
    assert_eq!(
        archive.remember([1; 32], &text, Some(u64::MAX)).unwrap(),
        receipt
    );
    let loaded = archive.load().unwrap();
    assert_eq!(loaded.len(), 1);
    let retained = loaded.values().next().unwrap();
    assert_eq!(retained.text.as_bytes(), text.as_bytes());
    assert_eq!(retained.created, Some(u64::MAX));
}

#[test]
fn late_valid_json_that_exceeds_atomic_content_budget_imports_no_prefix() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("db");
    let overhead = line("", Some(u64::MAX)).len();
    let text = "a".repeat(MAX_TEXT_BYTES - overhead);
    let large = line(&text, Some(u64::MAX));
    assert_eq!(large.len(), MAX_TEXT_BYTES);
    let first = line("first must remain unpublished", None);
    let source = [first.clone(), large].concat();
    let original = source.clone();
    {
        let db = Database::open(&path, Backend::Ssd).unwrap();
        let archive = archive(&db);
        let error = archive.import_jsonl(&source).unwrap_err().to_string();
        assert!(
            error.contains("line 2") && error.contains("transaction byte limit"),
            "{error}"
        );
        let graph = ApplicationGraph::from_database(db);
        assert!(graph.head(&NS).unwrap().is_none());
        assert!(
            graph
                .get(hemera::hash(b"first must remain unpublished").as_bytes())
                .unwrap()
                .is_none()
        );
    }
    let db = Database::open(path, Backend::Ssd).unwrap();
    assert!(archive(&db).load().unwrap().is_empty());
    assert_eq!(source, original);
    // The exact original first line is still an independently valid source.
    assert_eq!(archive(&db).import_jsonl(&first).unwrap().observations, 1);
}

#[test]
fn multi_transaction_source_preflights_projection_and_fences_concurrent_capacity() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("db");
    let db = Database::open(&path, Backend::Ssd).unwrap();
    let archive = archive(&db);
    // 63 MiB of distinct UTF-8, much larger than one 16 MiB content transaction.
    // Every observation independently fits, and every raw line is retained.
    let mut source = Vec::new();
    for n in 0..9u8 {
        source.extend(line(
            &((b'a' + n) as char).to_string().repeat(7 * 1024 * 1024),
            Some(n.into()),
        ));
    }
    let report = archive.import_jsonl(&source).unwrap();
    assert_eq!(report.observations, 9);
    assert_eq!(report.bytes, source.len() as u64);
    assert_eq!(report.source, *hemera::hash(&source).as_bytes());
    let graph = ApplicationGraph::from_database(db.clone());
    let before = graph.head(&NS).unwrap();
    let overflow = [
        line("would create a premature prefix", None),
        line(&"z".repeat(2 * 1024 * 1024), None),
    ]
    .concat();
    let error = archive.import_jsonl(&overflow).unwrap_err().to_string();
    assert!(
        error.contains("line 2") && error.contains("projection byte limit"),
        "{error}"
    );
    assert_eq!(graph.head(&NS).unwrap(), before);

    let gate = Arc::new(Barrier::new(3));
    let handles: Vec<_> = (0..2u8)
        .map(|n| {
            let db = db.clone();
            let gate = gate.clone();
            std::thread::spawn(move || {
                let text = ((b'x' + n) as char).to_string().repeat(1024 * 1024);
                gate.wait();
                let result =
                    TextArchive::from_database(db, NS).remember([200 + n; 32], &text, None);
                (n, text, result)
            })
        })
        .collect();
    gate.wait();
    let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(
        results
            .iter()
            .filter(|(_, _, result)| result.is_ok())
            .count(),
        1
    );
    let (winner, text, receipt) = results
        .into_iter()
        .find(|(_, _, result)| result.is_ok())
        .unwrap();
    assert_eq!(
        archive.remember([200 + winner; 32], &text, None).unwrap(),
        receipt.unwrap()
    );
    assert!(
        archive
            .remember([230; 32], "one byte past capacity", None)
            .is_err()
    );
    // A new observation of existing text changes metadata, not projection size.
    archive.remember([231; 32], &text, Some(77)).unwrap();
    assert_eq!(archive.import_jsonl(&source).unwrap(), report);
    drop(graph);
    drop(archive);
    drop(db);

    let db = Database::open(path, Backend::Ssd).unwrap();
    let loaded = TextArchive::from_database(db.clone(), NS).load().unwrap();
    assert_eq!(
        loaded.values().map(|m| m.text.len()).sum::<usize>(),
        MAX_PROJECTION_BYTES
    );
    assert_eq!(loaded.len(), 10);
    assert_eq!(
        loaded[hemera::hash(text.as_bytes()).as_bytes()].created,
        Some(77)
    );
    let retained = loaded
        .values()
        .find(|m| m.provenance.as_ref().is_some_and(|p| p.line == 1))
        .unwrap();
    let origin = retained.provenance.as_ref().unwrap();
    assert_eq!(origin.source, report.source);
    let graph = ApplicationGraph::from_database(db);
    let raw = graph.get(&origin.raw).unwrap().unwrap();
    assert_eq!(
        raw.bytes(),
        source.split_inclusive(|b| *b == b'\n').next().unwrap()
    );
}

#[test]
fn shared_database_failure_never_reports_completed_import_and_reopen_can_retry() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("db");
    let source = [line("one\twith\rcontrols", None), line("two", Some(2))].concat();
    {
        let db = Database::open(&path, Backend::Ssd).unwrap();
        let archive = archive(&db);
        archive
            .remember([8; 32], "previous durable state", Some(8))
            .unwrap();
        // Poison the real shared database writer lock after a committed prefix.
        // No fake TextArchive backend and no production fault injection API.
        let failed = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = db.transaction::<(), bbg::storage::StorageError>(|_| {
                panic!("injected caller panic while holding the database transaction");
            });
        }));
        assert!(failed.is_err() && db.is_poisoned());
        assert!(archive.import_jsonl(&source).is_err());
        assert!(archive.remember([9; 32], "must not succeed", None).is_err());
    }
    let db = Database::open(path, Backend::Ssd).unwrap();
    let archive = archive(&db);
    assert_eq!(archive.load().unwrap().len(), 1);
    let report = archive.import_jsonl(&source).unwrap();
    assert_eq!(report.observations, 2);
    assert_eq!(archive.import_jsonl(&source).unwrap(), report);
    let loaded = archive.load().unwrap();
    assert_eq!(loaded.len(), 3);
    assert_eq!(
        loaded[hemera::hash(b"one\twith\rcontrols").as_bytes()].text,
        "one\twith\rcontrols"
    );
}
