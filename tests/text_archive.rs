#![cfg(feature = "local-storage")]
use cybergraph::{
    application::{Backend, Database},
    text_archive::TextArchive,
};
use std::sync::atomic::{AtomicU64, Ordering};
struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "graph-text-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn archive(&self) -> TextArchive {
        TextArchive::from_database(
            Database::open(self.0.join("db"), Backend::Ssd).unwrap(),
            [1; 32],
        )
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn line(text: &str, created: Option<u64>) -> Vec<u8> {
    let id = hemera::hash(text.as_bytes());
    let hash = id
        .as_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let mut object = serde_json::json!({"particle":hash,"text":text});
    if let Some(created) = created {
        object["created"] = created.into();
    }
    let mut bytes = serde_json::to_vec(&object).unwrap();
    bytes.push(b'\n');
    bytes
}
#[test]
fn durable_observations_keep_original_particles_dates_and_request_conflicts() {
    let dir = Directory::new();
    let receipt;
    {
        let archive = dir.archive();
        receipt = archive.remember([2; 32], "hello", Some(17)).unwrap();
        archive.remember([3; 32], "world", None).unwrap();
    }
    let archive = dir.archive();
    assert_eq!(
        archive.remember([2; 32], "hello", Some(17)).unwrap(),
        receipt
    );
    assert!(archive.remember([2; 32], "changed", Some(17)).is_err());
    let texts = archive.load().unwrap();
    assert_eq!(texts.len(), 2);
    assert_eq!(texts[hemera::hash(b"hello").as_bytes()].created, Some(17));
    assert_eq!(texts[hemera::hash(b"world").as_bytes()].created, None);
}
#[test]
fn legacy_lines_keep_duplicates_raw_bytes_and_unknown_dates_across_retry() {
    let dir = Directory::new();
    let raw = [
        line("one", Some(3)),
        line("two\twith\rcontrols", None),
        line("one", None),
    ]
    .concat();
    // Reproduce the old hand-written writer's literal tab/CR string format.
    let raw = String::from_utf8(raw)
        .unwrap()
        .replace("\\t", "\t")
        .replace("\\r", "\r")
        .into_bytes();
    let report = {
        let archive = dir.archive();
        archive.import_jsonl(&raw).unwrap()
    };
    let archive = dir.archive();
    assert_eq!(archive.import_jsonl(&raw).unwrap(), report);
    assert_eq!(report.observations, 3);
    assert_eq!(report.head.unwrap().index, 2);
    let texts = archive.load().unwrap();
    assert_eq!(texts.len(), 2);
    let last = &texts[hemera::hash(b"one").as_bytes()];
    assert_eq!(last.created, None);
    let provenance = last.provenance.as_ref().unwrap();
    assert_eq!(provenance.source, report.source);
    assert_eq!(provenance.line, 3);
    assert_eq!(provenance.raw, *hemera::hash(&line("one", None)).as_bytes());
}
#[test]
fn a_bad_late_line_or_truncated_tail_imports_no_prefix() {
    let dir = Directory::new();
    let archive = dir.archive();
    let good = line("good", None);
    let mut bad = good.clone();
    bad.extend(line("wrong", None));
    // Change text without changing its named particle.
    let bad = String::from_utf8(bad).unwrap().replace("wrong", "fraud");
    assert!(
        archive
            .import_jsonl(bad.as_bytes())
            .unwrap_err()
            .to_string()
            .contains("line 2")
    );
    assert!(archive.load().unwrap().is_empty());
    assert!(archive.import_jsonl(&good[..good.len() - 1]).is_err());
    assert!(archive.load().unwrap().is_empty());
    assert!(archive.import_jsonl(&[]).unwrap().head.is_none());
}
