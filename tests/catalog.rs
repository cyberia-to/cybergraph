#![cfg(feature = "local-storage")]
use cybergraph::{
    Particle,
    application::{ApplicationGraph, Backend, Database, Head},
    catalog::{Catalog, Change, Entry, Error, MAX_PATH_BYTES},
    files::Upload,
};
use std::{
    collections::BTreeMap,
    io::Cursor,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

const NAMESPACE: Particle = [61; 32];
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "catalog-{}-{}",
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
fn backends() -> Vec<Backend> {
    let temp = Temp::new();
    match Database::open(temp.0.join("probe"), Backend::Hdd) {
        Ok(db) => {
            drop(db);
            vec![Backend::Ssd, Backend::Hdd]
        }
        Err(bbg::storage::StorageError::Unsupported(_)) => vec![Backend::Ssd],
        Err(error) => panic!("backend probe: {error}"),
    }
}
fn request(n: u64) -> Particle {
    let mut id = [0; 32];
    id[..8].copy_from_slice(&n.to_be_bytes());
    id
}
fn seal(graph: &ApplicationGraph, bytes: &[u8]) -> Particle {
    let particle = *hemera::hash(bytes).as_bytes();
    graph
        .files()
        .import(
            Upload {
                namespace: NAMESPACE,
                request: particle,
            },
            particle,
            bytes.len() as u64,
            31,
            Cursor::new(bytes),
        )
        .unwrap();
    particle
}
fn all(catalog: &Catalog<'_>, at: Option<Head>, page_size: usize) -> Vec<(String, Entry)> {
    let mut after = None;
    let mut entries = Vec::new();
    loop {
        let page = catalog.list(at, after.as_deref(), page_size).unwrap();
        assert!(page.entries.len() <= page_size);
        entries.extend(page.entries);
        after = page.next;
        if after.is_none() {
            return entries;
        }
    }
}

#[test]
fn rename_edit_remove_and_reopen_keep_old_states_and_payloads() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let first = seal(&graph, b"original content");
        let second = seal(&graph, b"new content");
        let catalog = Catalog::new(&graph, NAMESPACE);
        let create = Change::Create {
            path: "notes/start",
            particle: first,
        };
        let h0 = catalog.apply(request(1), None, create).unwrap();
        let original = catalog.resolve(Some(h0), "notes/start").unwrap().unwrap();
        let rename = Change::Rename {
            from: "notes/start",
            to: "archive/moved",
            expected: original,
        };
        let h1 = catalog.apply(request(2), Some(h0), rename).unwrap();
        assert_eq!(
            catalog.resolve(Some(h1), "archive/moved").unwrap(),
            Some(original)
        );
        assert_eq!(catalog.resolve(Some(h1), "notes/start").unwrap(), None);
        let h2 = catalog
            .apply(
                request(3),
                Some(h1),
                Change::Edit {
                    path: "archive/moved",
                    expected: original,
                    particle: second,
                },
            )
            .unwrap();
        let revised = catalog.resolve(Some(h2), "archive/moved").unwrap().unwrap();
        assert_eq!(revised.binding, original.binding);
        assert_ne!(revised.revision, original.revision);
        assert_eq!(revised.particle, second);
        let h3 = catalog
            .apply(
                request(4),
                Some(h2),
                Change::Remove {
                    path: "archive/moved",
                    expected: revised,
                },
            )
            .unwrap();
        assert_eq!(catalog.list(Some(h3), None, 1).unwrap().entries, vec![]);
        assert_eq!(catalog.apply(request(1), None, create).unwrap(), h0);
        assert_eq!(catalog.apply(request(2), Some(h0), rename).unwrap(), h1);
        assert!(
            catalog
                .apply(
                    request(2),
                    Some(h0),
                    Change::Remove {
                        path: "notes/start",
                        expected: original
                    }
                )
                .is_err()
        );
        assert_eq!(catalog.head().unwrap(), Some(h3));
        drop(graph);
        let graph = temp.open(backend);
        let catalog = Catalog::new(&graph, NAMESPACE);
        assert_eq!(catalog.head().unwrap(), Some(h3));
        assert_eq!(catalog.history(None, 2).unwrap(), [h0, h1]);
        assert_eq!(catalog.history(Some(1), 2).unwrap(), [h2, h3]);
        assert_eq!(
            catalog.resolve(Some(h0), "notes/start").unwrap(),
            Some(original)
        );
        assert_eq!(
            catalog.resolve(Some(h1), "archive/moved").unwrap(),
            Some(original)
        );
        assert_eq!(
            catalog.resolve(Some(h2), "archive/moved").unwrap(),
            Some(revised)
        );
        assert_eq!(
            graph.files().read_range(NAMESPACE, first, 0, 100).unwrap(),
            b"original content"
        );
        assert_eq!(
            graph.files().read_range(NAMESPACE, second, 0, 100).unwrap(),
            b"new content"
        );
        assert!(
            graph
                .files()
                .is_retained(NAMESPACE, first, h0.commit)
                .unwrap()
        );
        assert!(
            graph
                .files()
                .is_retained(NAMESPACE, second, h2.commit)
                .unwrap()
        );
    }
}

#[test]
fn ordered_pages_match_a_model_across_prefixes_unicode_and_mutations() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let particle = seal(&graph, b"shared content");
        let catalog = Catalog::new(&graph, NAMESPACE);
        let mut head = None;
        let mut model = BTreeMap::new();
        let mut paths = vec![
            "a".to_owned(),
            "a/b".to_owned(),
            "aa".to_owned(),
            "α/β".to_owned(),
            "e\u{301}".to_owned(),
            "é".to_owned(),
            "A".to_owned(),
        ];
        paths.extend((0..151).map(|n| format!("dir/{:03}", (n * 67) % 151)));
        paths.extend(["x".repeat(4096), "x".repeat(4095), "z".repeat(4080)]);
        for (i, path) in paths.iter().enumerate() {
            head = Some(
                catalog
                    .apply(
                        request(i as u64 + 1),
                        head,
                        Change::Create { path, particle },
                    )
                    .unwrap(),
            );
            model.insert(path.clone(), catalog.resolve(head, path).unwrap().unwrap());
        }
        let snapshot = head;
        assert_eq!(model["a"].particle, model["A"].particle);
        assert_ne!(model["a"].binding, model["A"].binding);
        assert!(matches!(
            catalog.apply(
                request(9999),
                head,
                Change::Rename {
                    from: "a",
                    to: "A",
                    expected: model["a"],
                }
            ),
            Err(Error::Conflict)
        ));
        assert_eq!(catalog.head().unwrap(), snapshot);
        let expected: Vec<_> = model.clone().into_iter().collect();
        for page_size in [1, 7, 31, 4096] {
            assert_eq!(all(&catalog, head, page_size), expected);
        }
        for (i, path) in paths.iter().enumerate() {
            if i % 3 == 0 {
                let entry = model.remove(path).unwrap();
                head = Some(
                    catalog
                        .apply(
                            request(i as u64 + 1000),
                            head,
                            Change::Remove {
                                path,
                                expected: entry,
                            },
                        )
                        .unwrap(),
                );
            } else if i % 3 == 1 {
                let entry = model.remove(path).unwrap();
                let to = format!("moved/{path}");
                head = Some(
                    catalog
                        .apply(
                            request(i as u64 + 1000),
                            head,
                            Change::Rename {
                                from: path,
                                to: &to,
                                expected: entry,
                            },
                        )
                        .unwrap(),
                );
                model.insert(to, entry);
            }
        }
        assert_eq!(
            all(&catalog, head, 7),
            model.into_iter().collect::<Vec<_>>()
        );
        assert_eq!(all(&catalog, snapshot, 7), expected);
        assert!(matches!(
            catalog.list(head, Some("missing"), 1),
            Err(Error::InvalidCursor)
        ));
        assert!(matches!(catalog.list(head, None, 0), Err(Error::Limit)));
        assert!(matches!(catalog.list(head, None, 4097), Err(Error::Limit)));
    }
}

#[test]
fn invalid_paths_unsealed_payloads_stale_bindings_and_competing_writers_publish_nothing() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let catalog = Catalog::new(&graph, NAMESPACE);
        let missing = request(99);
        for path in [
            "",
            "/start",
            "end/",
            "two//parts",
            ".",
            "..",
            "a/../b",
            "a/./b",
            "nul\0",
        ] {
            assert!(matches!(
                catalog.apply(
                    request(1),
                    None,
                    Change::Create {
                        path,
                        particle: missing
                    }
                ),
                Err(Error::InvalidPath)
            ));
        }
        let long = "x".repeat(MAX_PATH_BYTES + 1);
        assert!(matches!(
            catalog.apply(
                request(1),
                None,
                Change::Create {
                    path: &long,
                    particle: missing
                }
            ),
            Err(Error::Limit)
        ));
        assert!(
            catalog
                .apply(
                    request(1),
                    None,
                    Change::Create {
                        path: "file",
                        particle: missing
                    }
                )
                .is_err()
        );
        assert!(catalog.head().unwrap().is_none());
        let first = seal(&graph, b"one");
        let second = seal(&graph, b"two");
        let h0 = catalog
            .apply(
                request(1),
                None,
                Change::Create {
                    path: "file",
                    particle: first,
                },
            )
            .unwrap();
        let entry = catalog.resolve(Some(h0), "file").unwrap().unwrap();
        let wrong = Entry {
            revision: request(999),
            ..entry
        };
        assert!(matches!(
            catalog.apply(
                request(2),
                Some(h0),
                Change::Remove {
                    path: "file",
                    expected: wrong
                }
            ),
            Err(Error::Conflict)
        ));
        assert!(
            Catalog::new(&graph, [62; 32])
                .resolve(Some(h0), "file")
                .is_err()
        );
        let barrier = std::sync::Barrier::new(2);
        let (rename, edit) = std::thread::scope(|scope| {
            let a = scope.spawn(|| {
                barrier.wait();
                catalog.apply(
                    request(2),
                    Some(h0),
                    Change::Rename {
                        from: "file",
                        to: "renamed",
                        expected: entry,
                    },
                )
            });
            let b = scope.spawn(|| {
                barrier.wait();
                catalog.apply(
                    request(3),
                    Some(h0),
                    Change::Edit {
                        path: "file",
                        expected: entry,
                        particle: second,
                    },
                )
            });
            (a.join().unwrap(), b.join().unwrap())
        });
        assert_ne!(rename.is_ok(), edit.is_ok());
        let selected = catalog.head().unwrap();
        let entries = all(&catalog, selected, 1);
        assert_eq!(entries.len(), 1);
        if rename.is_ok() {
            assert_eq!(entries[0], ("renamed".to_owned(), entry));
        } else {
            assert_eq!(entries[0].0, "file");
            assert_eq!(entries[0].1.particle, second);
        }
        assert_eq!(catalog.history(None, 10).unwrap().len(), 2);
        assert_eq!(catalog.resolve(Some(h0), "file").unwrap(), Some(entry));
    }
}

#[test]
fn retained_revision_count_exceeds_a_history_page_and_resumes_after_reopen() {
    for backend in backends() {
        let temp = Temp::new();
        let graph = temp.open(backend);
        let particle = seal(&graph, b"same bytes, separate revisions");
        let catalog = Catalog::new(&graph, NAMESPACE);
        let mut head = catalog
            .apply(
                request(1),
                None,
                Change::Create {
                    path: "history",
                    particle,
                },
            )
            .unwrap();
        let first = head;
        for n in 2..=4100 {
            let entry = catalog.resolve(Some(head), "history").unwrap().unwrap();
            head = catalog
                .apply(
                    request(n),
                    Some(head),
                    Change::Edit {
                        path: "history",
                        expected: entry,
                        particle,
                    },
                )
                .unwrap();
        }
        let final_head = head;
        drop(graph);
        let graph = temp.open(backend);
        let catalog = Catalog::new(&graph, NAMESPACE);
        assert_eq!(catalog.head().unwrap(), Some(final_head));
        let first_page = catalog.history(None, 4096).unwrap();
        assert_eq!(first_page.len(), 4096);
        let rest = catalog
            .history(Some(first_page.last().unwrap().index), 4096)
            .unwrap();
        assert_eq!(rest.len(), 4);
        assert_eq!(rest.last().copied(), Some(final_head));
        assert_eq!(
            catalog
                .resolve(Some(first), "history")
                .unwrap()
                .unwrap()
                .particle,
            particle
        );
        assert_eq!(
            catalog
                .resolve(Some(final_head), "history")
                .unwrap()
                .unwrap()
                .particle,
            particle
        );
    }
}
