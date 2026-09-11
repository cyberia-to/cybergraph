#![cfg(feature = "local-storage")]
use cybergraph::application::{ApplicationGraph, Error, Head, Proposal};
use cybergraph::content::{Codec, Content, ContentError};
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
        ApplicationGraph::open(self.0.join("graph.redb")).unwrap()
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
    let db = redb::Database::open(dir.0.join("graph.redb")).unwrap();
    let tx = db.begin_write().unwrap();
    {
        let table: redb::TableDefinition<&[u8], &[u8]> =
            redb::TableDefinition::new("application_content");
        tx.open_table(table)
            .unwrap()
            .insert(a.id().as_slice(), [0u8; 9].as_slice())
            .unwrap();
    }
    tx.commit().unwrap();
    drop(db);
    let graph = dir.open();
    assert!(matches!(
        graph.get(&a.id()),
        Err(Error::Content(ContentError::IdentityMismatch))
    ));
    assert_eq!(graph.head(&p.namespace).unwrap(), Some(p.head));
}
