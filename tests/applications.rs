#![cfg(feature = "local-storage")]
use bbg::storage::ShardStore;
use cybergraph::application::{
    ApplicationGraph, Archive, Backend, Database, Error, Head, MAX_INSPECTION_BYTES,
    MAX_INSPECTION_ROWS, Proposal,
};
use cybergraph::content::{Codec, Content, ContentError};
use nebu::Goldilocks;
use std::sync::atomic::{AtomicUsize, Ordering};
struct Directory(std::path::PathBuf);
impl Directory {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "graph-app-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn open(&self) -> ApplicationGraph {
        ApplicationGraph::open(self.0.join("bbg")).unwrap()
    }
}
impl Drop for Directory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn proposal(content: Vec<Content>, root: [u8; 32]) -> Proposal {
    Proposal {
        namespace: [1; 32],
        request: [2; 32],
        expected: None,
        head: Head {
            index: 0,
            commit: root,
        },
        content,
        required: vec![],
        claims: vec![],
    }
}

#[test]
fn missing_closure_and_application_rejection_publish_nothing() {
    let dir = Directory::new();
    let graph = dir.open();
    let a = Content::atom(1).unwrap();
    let b = Content::atom(2).unwrap();
    let pair = Content::pair(a.id(), b.id()).unwrap();
    let mut p = proposal(vec![a.clone(), pair.clone()], pair.id());
    assert!(matches!(
        graph.commit(&p, |_| Ok(())),
        Err(Error::MissingContent(_))
    ));
    assert!(graph.get(&a.id()).unwrap().is_none());
    p.content.push(b);
    assert!(matches!(
        graph.commit(&p, |_| Err(Error::Rejected("policy".into()))),
        Err(Error::Rejected(_))
    ));
    assert!(graph.head(&p.namespace).unwrap().is_none());
    graph.commit(&p, |_| Ok(())).unwrap();
    drop(graph);
    let graph = dir.open();
    assert_eq!(
        graph.get(&pair.id()).unwrap().unwrap().children(),
        pair.children()
    );
    assert_eq!(
        graph
            .commit(&p, |_| panic!("duplicate must resolve first"))
            .unwrap(),
        p.head
    );
}

#[test]
fn fresh_publication_allows_one_winner_even_after_both_preflight_reads() {
    let dir = Directory::new();
    let graph = dir.open();
    let value = Content::atom(19).unwrap();
    let p = proposal(vec![value.clone()], value.id());
    let barrier = std::sync::Barrier::new(2);
    let winners = AtomicUsize::new(0);
    std::thread::scope(|scope| {
        for _ in 0..2 {
            scope.spawn(|| {
                let result = graph.commit_fresh(&p, |_| {
                    barrier.wait();
                    Ok(())
                });
                match result {
                    Ok(head) => {
                        assert_eq!(head, p.head);
                        winners.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(Error::Storage(bbg::storage::application::Error::Conflict)) => {}
                    other => panic!("unexpected fresh-claim result: {other:?}"),
                }
            });
        }
    });
    assert_eq!(winners.load(Ordering::SeqCst), 1);
    drop(graph);
    let graph = dir.open();
    assert!(
        graph
            .commit_fresh(&p, |_| panic!("old receipt is never fresh"))
            .is_err()
    );
    assert_eq!(
        graph
            .commit(&p, |_| panic!("ordinary retry resolves"))
            .unwrap(),
        p.head
    );
}

#[test]
fn retained_application_references_are_required_and_fingerprint_binds_claims() {
    let dir = Directory::new();
    let graph = dir.open();
    let a = Content::atom(1).unwrap();
    let mut p = proposal(vec![a.clone()], a.id());
    p.required.push([9; 32]);
    assert!(matches!(
        graph.commit(&p, |_| Ok(())),
        Err(Error::MissingContent(_))
    ));
    p.required.clear();
    p.claims.push(([3; 32], [4; 32]));
    graph.commit(&p, |_| Ok(())).unwrap();
    p.claims[0].1 = [5; 32];
    assert!(graph.commit(&p, |_| Ok(())).is_err());
}

#[test]
fn noncanonical_atoms_and_child_particles_do_not_get_normalized() {
    assert!(Content::atom(0xffff_ffff_0000_0001).is_err());
    assert!(matches!(
        Content::new(Codec::Data, vec![255; 64]),
        Err(ContentError::NoncanonicalParticle)
    ));
    assert!(Content::new(Codec::Data, vec![1; 9]).is_err());
}

#[test]
fn corruption_is_reported_on_read_without_reinitializing_history() {
    let dir = Directory::new();
    let a = Content::atom(1).unwrap();
    let p = proposal(vec![a.clone()], a.id());
    let graph = dir.open();
    graph.commit(&p, |_| Ok(())).unwrap();
    drop(graph);
    let db = fjall::Config::new(dir.0.join("bbg")).open().unwrap();
    let content = db
        .open_partition(
            "application_content",
            fjall::PartitionCreateOptions::default(),
        )
        .unwrap();
    let mut batch = db.batch().durability(Some(fjall::PersistMode::SyncAll));
    batch.insert(&content, a.id(), [0u8; 9]);
    batch.commit().unwrap();
    drop(content);
    drop(db);
    let graph = dir.open();
    assert!(matches!(
        graph.get(&a.id()),
        Err(Error::Content(ContentError::IdentityMismatch))
    ));
    assert_eq!(graph.head(&p.namespace).unwrap(), Some(p.head));
}

#[test]
fn application_views_share_one_database_owner_and_receipts() {
    let dir = Directory::new();
    let database = Database::open(dir.0.join("bbg"), Backend::Ssd).unwrap();
    let mut shards = database.shards();
    let writer = ApplicationGraph::from_database(database.clone());
    let reader = ApplicationGraph::from_database(database);
    let content = Content::atom(7).unwrap();
    let p = proposal(vec![content.clone()], content.id());
    writer.commit(&p, |_| Ok(())).unwrap();
    assert_eq!(reader.head(&p.namespace).unwrap(), Some(p.head));
    assert_eq!(
        reader.resolve(&p.namespace, &p.request).unwrap(),
        Some(p.head)
    );
    assert_eq!(
        reader.get(&content.id()).unwrap().unwrap().id(),
        content.id()
    );
    shards.put(0, [8; 32], vec![Goldilocks::ONE]).unwrap();
    shards.commit().unwrap();
    assert_eq!(
        shards.read(0, &[8; 32], 1).unwrap(),
        Some(vec![Goldilocks::ONE])
    );
    assert_eq!(reader.head(&p.namespace).unwrap(), Some(p.head));
    assert!(ApplicationGraph::open(dir.0.join("bbg")).is_err());
    drop(writer);
    drop(reader);
    drop(shards);
    assert_eq!(dir.open().head(&p.namespace).unwrap(), Some(p.head));
}

/// specs/applications.md: an archive seals exactly once for one target, and the
/// sealed source stages its namespaces into a fresh shared owner.
#[test]
fn a_sealed_archive_stages_its_namespaces_into_a_fresh_target() {
    let source = Directory::new();
    let target = Directory::new();
    let value = Content::atom(11).unwrap();
    let p = proposal(vec![value.clone()], value.id());
    let graph = source.open();
    let head = graph.commit(&p, |_| Ok(())).unwrap();
    drop(graph);

    let archive = Archive::open(&source.0.join("bbg")).unwrap();
    assert_eq!(archive.sources().len(), 1);
    assert_eq!(archive.sources()[0], (p.namespace, head));
    assert!(archive.seal().is_none());
    let summary = archive
        .inspect(MAX_INSPECTION_ROWS, MAX_INSPECTION_BYTES)
        .unwrap();
    // one content row, no claim, one request, one history entry, one head
    assert_eq!((summary.rows, summary.tables), (4, [1, 0, 1, 1, 1]));
    let transfer = archive.seal_for_transfer([9; 32], [8; 32]).unwrap();
    assert_eq!(transfer.sources()[0], (p.namespace, head));

    let into = target.open();
    let mut progress = transfer.stage(&into, 1).unwrap();
    for _ in 0..16 {
        if progress.complete {
            break;
        }
        progress = transfer.stage(&into, 1).unwrap();
    }
    assert!(progress.complete, "staging did not finish: {progress:?}");
    assert_eq!(progress.manifest, transfer.manifest());
    assert_eq!(progress.rows, summary.rows);
    assert_eq!(into.head(&p.namespace).unwrap(), Some(head));
    assert_eq!(into.resolve(&p.namespace, &p.request).unwrap(), Some(head));
    assert_eq!(into.get(&value.id()).unwrap().unwrap().id(), value.id());
    assert_eq!(into.history(&p.namespace, None, 8).unwrap(), vec![head]);

    // The source now carries its export seal: it serves that one target only.
    drop(transfer);
    let archive = Archive::open(&source.0.join("bbg")).unwrap();
    assert_eq!(archive.seal().unwrap().target, [9; 32]);
    assert!(archive.seal_for_transfer([7; 32], [8; 32]).is_err());
}

/// specs/applications.md ## legacy import: the explicit redb import carries
/// content, heads, history, claims and request fingerprints into a fresh
/// directory, retains the source, and never overwrites an existing destination.
#[cfg(feature = "legacy-redb-migration")]
#[test]
fn a_legacy_redb_application_file_imports_into_a_fresh_directory() {
    const STORAGE_MARKER: redb::TableDefinition<'static, &'static [u8], &'static [u8]> =
        redb::TableDefinition::new("bbg_storage_v1");
    let dir = Directory::new();
    let legacy = dir.0.join("legacy.redb");
    let value = Content::atom(23).unwrap();
    let mut p = proposal(vec![value.clone()], value.id());
    p.claims.push(([3; 32], [4; 32]));
    let head = {
        let graph = ApplicationGraph::from_database(Database::open(&legacy, Backend::Hdd).unwrap());
        graph.commit(&p, |_| Ok(())).unwrap()
    };
    // A legacy application file holds the five application tables only; the
    // storage marker belongs to the engine that wrote it, not to the export.
    let db = redb::Database::open(&legacy).unwrap();
    let write = db.begin_write().unwrap();
    assert!(write.delete_table(STORAGE_MARKER).unwrap());
    write.commit().unwrap();
    drop(db);

    ApplicationGraph::migrate_redb(&legacy, dir.0.join("bbg")).unwrap();
    assert!(legacy.exists(), "the source is retained");
    let graph = dir.open();
    assert_eq!(graph.head(&p.namespace).unwrap(), Some(head));
    assert_eq!(graph.resolve(&p.namespace, &p.request).unwrap(), Some(head));
    assert_eq!(graph.get(&value.id()).unwrap().unwrap().id(), value.id());
    assert_eq!(graph.history(&p.namespace, None, 8).unwrap(), vec![head]);
    // The global claim arrived with it: the same key cannot be reused.
    let next = Content::atom(24).unwrap();
    let mut conflicting = proposal(vec![next.clone()], next.id());
    conflicting.request = [5; 32];
    conflicting.expected = Some(head);
    conflicting.head = Head {
        index: 1,
        commit: next.id(),
    };
    conflicting.claims.push(([3; 32], [9; 32]));
    assert!(graph.commit(&conflicting, |_| Ok(())).is_err());
    drop(graph);
    // A second import never overwrites the directory it already produced.
    assert!(ApplicationGraph::migrate_redb(&legacy, dir.0.join("bbg")).is_err());
}
