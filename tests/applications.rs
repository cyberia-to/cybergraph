#![cfg(feature = "local-storage")]
use bbg::storage::ShardStore;
use cybergraph::application::{ApplicationGraph, Backend, Database, Error, Head, Proposal};
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
                let result = graph.commit_fresh(&p, |_| { barrier.wait(); Ok(()) });
                match result {
                    Ok(head) => { assert_eq!(head, p.head); winners.fetch_add(1, Ordering::SeqCst); }
                    Err(Error::Storage(bbg::storage::application::Error::Conflict)) => {}
                    other => panic!("unexpected fresh-claim result: {other:?}"),
                }
            });
        }
    });
    assert_eq!(winners.load(Ordering::SeqCst), 1);
    drop(graph);
    let graph = dir.open();
    assert!(graph.commit_fresh(&p, |_| panic!("old receipt is never fresh")).is_err());
    assert_eq!(graph.commit(&p, |_| panic!("ordinary retry resolves")).unwrap(), p.head);
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
