#![cfg(feature = "local-storage")]
use cybergraph::{
    application::{ApplicationGraph, Backend, Database, Error, Head, Proposal},
    content::{Codec, Content, MAX_CONTENT_BYTES},
    files::{Files, Upload},
};
use std::{
    io::{Cursor, Read, Seek, SeekFrom},
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "graph-files-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn open(&self, backend: Backend) -> ApplicationGraph {
        ApplicationGraph::from_database(Database::open(self.0.join("bbg"), backend).unwrap())
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn upload(request: u8) -> Upload {
    Upload {
        namespace: [1; 32],
        request: [request; 32],
    }
}
fn backends() -> Vec<Backend> {
    // The acceptance command enables both BBG profiles. Default local-storage
    // builds retain the SSD-only test path without adding a product dependency.
    let temp = Temp::new();
    match Database::open(temp.0.join("probe"), Backend::Hdd) {
        Ok(db) => {
            drop(db);
            vec![Backend::Ssd, Backend::Hdd]
        }
        Err(bbg::storage::StorageError::Unsupported(_)) => vec![Backend::Ssd],
        Err(error) => panic!("HDD backend probe: {error}"),
    }
}
fn proposal(blob: [u8; 32], index: u64, previous: Option<Head>) -> Proposal {
    let mut bytes = b"file manifest\0".to_vec();
    bytes.extend_from_slice(&blob);
    bytes.extend_from_slice(&index.to_le_bytes());
    let manifest = Content::new(Codec::Blob, bytes).unwrap();
    Proposal {
        namespace: [1; 32],
        request: [index as u8 + 20; 32],
        expected: previous,
        head: Head {
            index,
            commit: manifest.id(),
        },
        content: vec![manifest],
        required: vec![blob],
        claims: vec![],
    }
}

#[test]
fn streamed_blobs_keep_existing_identities_and_support_bounded_seek_reads() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let files = graph.files();
        for (n, len) in [0usize, 1, 55, 56, 57, 127, 128, 129, 1025]
            .into_iter()
            .enumerate()
        {
            let mut bytes: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            if let Some(last) = bytes.last_mut() {
                *last = 0;
            }
            let content = Content::new(Codec::Blob, bytes.clone()).unwrap();
            files
                .import(
                    upload(n as u8),
                    content.id(),
                    len as u64,
                    127,
                    Cursor::new(&bytes),
                )
                .unwrap();
            let mut reader = files.reader([1; 32], content.id()).unwrap();
            let mut out = Vec::new();
            reader.read_to_end(&mut out).unwrap();
            assert_eq!(out, bytes);
            if len > 0 {
                reader.seek(SeekFrom::End(-1)).unwrap();
                let mut last = [99];
                reader.read_exact(&mut last).unwrap();
                assert_eq!(last, [0]);
                reader.seek(SeekFrom::Start(0)).unwrap();
                reader.read_exact(&mut last).unwrap();
                assert_eq!(last[0], bytes[0]);
            }
            assert!(reader.seek(SeekFrom::End(-((len as i64) + 1))).is_err());
            reader.seek(SeekFrom::Start(u64::MAX)).unwrap();
            assert_eq!(reader.read(&mut [0; 1]).unwrap(), 0);
        }
    }
}

#[test]
fn application_publication_requires_namespace_scoped_sealed_content_and_binds_retry() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let bytes = b"private application payload";
        let id = *hemera::hash(bytes).as_bytes();
        let files = graph.files();
        files.begin(upload(2), id, bytes.len() as u64, 7).unwrap();
        let p = proposal(id, 0, None);
        assert!(matches!(
            graph.commit_with_blobs(&p, &[id], |_| Ok(())),
            Err(Error::MissingContent(_))
        ));
        assert!(graph.head(&[1; 32]).unwrap().is_none());
        files
            .import(upload(2), id, bytes.len() as u64, 7, Cursor::new(bytes))
            .unwrap();
        let mut wrong_namespace = proposal(id, 0, None);
        wrong_namespace.namespace = [2; 32];
        assert!(matches!(
            graph.commit_with_blobs(&wrong_namespace, &[id], |_| Ok(())),
            Err(Error::MissingContent(_))
        ));
        assert!(matches!(
            graph.commit_with_blobs(&p, &[id], |_| Err(Error::Rejected("policy".into()))),
            Err(Error::Rejected(_))
        ));
        assert!(!files.is_retained([1; 32], id, p.head.commit).unwrap());
        graph.commit_with_blobs(&p, &[id], |_| Ok(())).unwrap();
        let next = proposal(id, 1, Some(p.head));
        graph.commit_with_blobs(&next, &[id], |_| Ok(())).unwrap();
        assert_eq!(
            graph
                .commit_with_blobs(&p, &[id], |_| panic!("retry validates again"))
                .unwrap(),
            p.head
        );
        assert!(
            graph
                .commit(&p, |_| panic!("changed retry validates"))
                .is_err()
        );
        drop(files);
        drop(graph);
        let graph = temp.open(backend);
        assert_eq!(graph.head(&[1; 32]).unwrap(), Some(next.head));
        assert!(
            graph
                .files()
                .is_retained([1; 32], id, p.head.commit)
                .unwrap()
        );
        assert!(
            graph
                .files()
                .is_retained([1; 32], id, next.head.commit)
                .unwrap()
        );
        assert_eq!(
            graph.files().read_range([1; 32], id, 0, 100).unwrap(),
            bytes
        );
    }
}

#[test]
fn concurrent_head_updates_retain_only_the_winning_publication() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let a = *hemera::hash(b"first").as_bytes();
        let b = *hemera::hash(b"second").as_bytes();
        graph
            .files()
            .import(upload(1), a, 5, 3, Cursor::new(b"first"))
            .unwrap();
        graph
            .files()
            .import(upload(2), b, 6, 3, Cursor::new(b"second"))
            .unwrap();
        let p = proposal(a, 0, None);
        let mut q = proposal(b, 0, None);
        q.request = [88; 32];
        let barrier = std::sync::Barrier::new(2);
        std::thread::scope(|scope| {
            let first = scope.spawn(|| {
                graph.commit_with_blobs(&p, &[a], |_| {
                    barrier.wait();
                    Ok(())
                })
            });
            let second = scope.spawn(|| {
                graph.commit_with_blobs(&q, &[b], |_| {
                    barrier.wait();
                    Ok(())
                })
            });
            let (first, second) = (first.join().unwrap(), second.join().unwrap());
            assert_ne!(first.is_ok(), second.is_ok());
            assert_eq!(
                graph
                    .files()
                    .is_retained([1; 32], a, p.head.commit)
                    .unwrap(),
                first.is_ok()
            );
            assert_eq!(
                graph
                    .files()
                    .is_retained([1; 32], b, q.head.commit)
                    .unwrap(),
                second.is_ok()
            );
        });
    }
}

#[test]
fn streaming_crosses_the_old_whole_value_limit() {
    // A real file larger than the old Content allocation cap, generated and
    // checked with one fixed-size buffer. No full-file allocation in the test.
    for backend in backends() {
        let temp = Temp::new();
        let length = MAX_CONTENT_BYTES as u64 + 17;
        let mut hash = hemera::Hasher::new();
        let block = [42u8; 65536];
        let mut remaining = length;
        while remaining > 0 {
            let take = remaining.min(block.len() as u64) as usize;
            hash.update(&block[..take]);
            remaining -= take as u64;
        }
        let particle = *hash.finalize().as_bytes();
        {
            let graph = temp.open(backend);
            graph
                .files()
                .import(
                    upload(9),
                    particle,
                    length,
                    block.len() as u32,
                    std::io::repeat(42).take(length),
                )
                .unwrap();
            let p = proposal(particle, 0, None);
            graph
                .commit_with_blobs(&p, &[particle], |_| Ok(()))
                .unwrap();
        }
        let files = Files::from_database(Database::open(temp.0.join("bbg"), backend).unwrap());
        let mut reader = files.reader([1; 32], particle).unwrap();
        let mut buffer = [0u8; 65536];
        let mut total = 0;
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            assert!(buffer[..n].iter().all(|b| *b == 42));
            total += n as u64;
        }
        assert_eq!(total, length);
    }
}
