use cybergraph::{
    application::{ApplicationGraph, Backend, Database, Head, Proposal},
    content::{Codec, Content},
    files::{State, Upload},
};
use cybergraph_radio::{ALPN, Client, FileProtocol, FileSink, FileSource, receive_page};
use radio::{
    Endpoint, EndpointAddr, RelayMode, SecretKey,
    files::{Descriptor, Source},
    protocol::Router,
};
use std::{
    io::{self, Read},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

const DEADLINE: Duration = Duration::from_secs(10);

fn backends() -> Vec<Backend> {
    vec![
        Backend::Ssd,
        #[cfg(feature = "hdd")]
        Backend::Hdd,
    ]
}
fn open(path: &std::path::Path, backend: Backend) -> ApplicationGraph {
    ApplicationGraph::from_database(Database::open(path, backend).unwrap())
}
fn upload() -> Upload {
    Upload {
        namespace: [2; 32],
        request: [3; 32],
    }
}
fn proposal(particle: [u8; 32]) -> Proposal {
    let manifest = Content::new(
        Codec::Blob,
        [b"attachment\0".as_slice(), &particle].concat(),
    )
    .unwrap();
    Proposal {
        namespace: upload().namespace,
        request: [4; 32],
        expected: None,
        head: Head {
            index: 0,
            commit: manifest.id(),
        },
        content: vec![manifest],
        required: vec![particle],
        claims: vec![],
    }
}
async fn endpoint(key: SecretKey) -> Endpoint {
    Endpoint::empty_builder(RelayMode::Disabled)
        .secret_key(key)
        .bind()
        .await
        .unwrap()
}

#[derive(Clone)]
struct Counted {
    source: FileSource,
    bytes: Arc<AtomicU64>,
    block_at: Option<(u64, Arc<tokio::sync::Notify>)>,
}
impl Source for Counted {
    fn descriptor(&self) -> Descriptor {
        self.source.descriptor()
    }
    async fn read(&self, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        if let Some((position, notify)) = &self.block_at
            && offset == *position
        {
            notify.notify_one();
            std::future::pending::<()>().await;
        }
        let bytes = self.source.read(offset, length).await?;
        self.bytes.fetch_add(bytes.len() as u64, Ordering::SeqCst);
        Ok(bytes)
    }
}
async fn serve(source: Counted, allowed: radio::EndpointId) -> (Router, EndpointAddr) {
    let provider = move |peer, file| {
        let source = source.clone();
        async move {
            if peer != allowed || file != source.descriptor().file {
                return Err(io::Error::from(io::ErrorKind::PermissionDenied));
            }
            Ok(source)
        }
    };
    let ep = endpoint(SecretKey::from_bytes(&[11; 32])).await;
    let router = Router::builder(ep)
        .accept(ALPN, FileProtocol::new(provider, 4, DEADLINE).unwrap())
        .spawn();
    let address = router.endpoint().addr();
    (router, address)
}

#[tokio::test]
async fn restart_resumes_only_missing_parts_and_retains_through_the_shared_owner() {
    for (source_backend, backend) in backends()
        .into_iter()
        .flat_map(|source| backends().into_iter().map(move |target| (source, target)))
    {
        let temp = tempfile::tempdir().unwrap();
        let source_path = temp.path().join("source");
        let target_path = temp.path().join("target");
        let source_graph = open(&source_path, source_backend);
        let target_graph = open(&target_path, backend);
        let bytes: Vec<_> = (0..(1024 * 1024 + 57)).map(|i| (i % 251) as u8).collect();
        let id = Content::new(Codec::Blob, bytes.clone()).unwrap().id();
        source_graph
            .files()
            .import(
                Upload {
                    namespace: [1; 32],
                    request: [1; 32],
                },
                id,
                bytes.len() as u64,
                31 * 1024,
                bytes.as_slice(),
            )
            .unwrap();
        target_graph
            .files()
            .begin(upload(), id, bytes.len() as u64, 64 * 1024)
            .unwrap();
        let key = SecretKey::from_bytes(&[12; 32]);
        let client_endpoint = endpoint(key.clone()).await;
        let count = Arc::new(AtomicU64::new(0));
        let interrupted = Arc::new(tokio::sync::Notify::new());
        let source = Counted {
            source: FileSource::open(source_graph.files(), [1; 32], id)
                .await
                .unwrap(),
            bytes: count.clone(),
            block_at: Some((128 * 1024, interrupted.clone())),
        };
        let (router, address) = serve(source, client_endpoint.id()).await;
        let mut client = Client::connect(&client_endpoint, address, DEADLINE)
            .await
            .unwrap();
        let sink = FileSink::open(target_graph.files(), upload())
            .await
            .unwrap();
        let receiving_sink = sink.clone();
        let receiving =
            tokio::spawn(async move { receive_page(&mut client, &receiving_sink, 0, 4).await });
        tokio::time::timeout(DEADLINE, interrupted.notified())
            .await
            .unwrap();
        router.shutdown().await.unwrap();
        assert!(
            tokio::time::timeout(DEADLINE, receiving)
                .await
                .unwrap()
                .unwrap()
                .is_err()
        );
        assert!(sink.seal().await.is_err());
        assert_eq!(target_graph.head(&upload().namespace).unwrap(), None);
        client_endpoint.close().await;
        drop(router);
        drop(sink);
        drop(target_graph);
        drop(source_graph);

        // Reopen both physical owners while every old transport is stopped.
        let source_graph = open(&source_path, source_backend);
        let target_graph = open(&target_path, backend);
        assert_eq!(
            target_graph
                .files()
                .progress(upload())
                .unwrap()
                .unwrap()
                .present_parts,
            2
        );
        let client_endpoint = endpoint(key).await;
        let source = Counted {
            source: FileSource::open(source_graph.files(), [1; 32], id)
                .await
                .unwrap(),
            bytes: count.clone(),
            block_at: None,
        };
        let (router, address) = serve(source, client_endpoint.id()).await;
        let mut client = Client::connect(&client_endpoint, address, DEADLINE)
            .await
            .unwrap();
        let sink = FileSink::open(target_graph.files(), upload())
            .await
            .unwrap();
        let mut next = Some(0);
        while let Some(from) = next {
            next = receive_page(&mut client, &sink, from, 3)
                .await
                .unwrap()
                .next;
        }
        assert_eq!(count.load(Ordering::SeqCst), bytes.len() as u64);
        let info = sink.seal().await.unwrap();
        assert_eq!(info.spec.particle, id);
        let proposed = proposal(id);
        target_graph
            .commit_with_blobs(&proposed, &[id], |_| Ok(()))
            .unwrap();
        drop(client);
        router.shutdown().await.unwrap();
        client_endpoint.close().await;
        drop(router);
        drop(sink);
        drop(target_graph);
        let graph = open(&target_path, backend);
        assert_eq!(
            graph.head(&upload().namespace).unwrap(),
            Some(proposed.head)
        );
        assert!(
            graph
                .files()
                .is_retained(upload().namespace, id, proposed.head.commit)
                .unwrap()
        );
        let mut output = Vec::new();
        graph
            .files()
            .reader(upload().namespace, id)
            .unwrap()
            .read_to_end(&mut output)
            .unwrap();
        assert_eq!(output, bytes);
        let mut entries: Vec<_> = std::fs::read_dir(temp.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        entries.sort();
        assert_eq!(entries, ["source", "target"]);
    }
}

#[tokio::test]
async fn denied_peer_cannot_read_or_advance_receiver_coverage() {
    for backend in backends() {
        let temp = tempfile::tempdir().unwrap();
        let graph = open(&temp.path().join("db"), backend);
        let bytes = b"private ciphertext fixture";
        let id = Content::new(Codec::Blob, bytes.to_vec()).unwrap().id();
        graph
            .files()
            .import(
                Upload {
                    namespace: [1; 32],
                    request: [1; 32],
                },
                id,
                bytes.len() as u64,
                8,
                bytes.as_slice(),
            )
            .unwrap();
        graph
            .files()
            .begin(upload(), id, bytes.len() as u64, 8)
            .unwrap();
        let count = Arc::new(AtomicU64::new(0));
        let source = Counted {
            source: FileSource::open(graph.files(), [1; 32], id).await.unwrap(),
            bytes: count.clone(),
            block_at: None,
        };
        let authorized = SecretKey::from_bytes(&[12; 32]);
        let (router, address) = serve(source, authorized.public()).await;
        let client_endpoint = endpoint(SecretKey::from_bytes(&[13; 32])).await;
        let mut client = Client::connect(&client_endpoint, address, DEADLINE)
            .await
            .unwrap();
        let sink = FileSink::open(graph.files(), upload()).await.unwrap();
        assert_eq!(
            receive_page(&mut client, &sink, 0, 4)
                .await
                .unwrap_err()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert_eq!(count.load(Ordering::SeqCst), 0);
        assert_eq!(
            graph
                .files()
                .progress(upload())
                .unwrap()
                .unwrap()
                .present_parts,
            0
        );
        assert!(
            graph
                .files()
                .info(upload().namespace, id)
                .unwrap()
                .is_none()
        );
        drop(client);
        router.shutdown().await.unwrap();
        client_endpoint.close().await;
    }
}

#[derive(Clone)]
struct WrongBytes {
    descriptor: Descriptor,
}
impl Source for WrongBytes {
    fn descriptor(&self) -> Descriptor {
        self.descriptor
    }
    async fn read(&self, _offset: u64, length: usize) -> io::Result<Vec<u8>> {
        Ok(vec![0; length])
    }
}

#[tokio::test]
async fn received_bytes_cannot_publish_before_full_identity_verification() {
    for backend in backends() {
        let temp = tempfile::tempdir().unwrap();
        let graph = open(&temp.path().join("db"), backend);
        let id = Content::new(Codec::Blob, b"correct data".to_vec())
            .unwrap()
            .id();
        graph.files().begin(upload(), id, 12, 4).unwrap();
        let sink = FileSink::open(graph.files(), upload()).await.unwrap();
        let descriptor = radio::files::Sink::descriptor(&sink);
        let server = endpoint(SecretKey::from_bytes(&[14; 32])).await;
        let client_endpoint = endpoint(SecretKey::from_bytes(&[15; 32])).await;
        let allowed = client_endpoint.id();
        let protocol = FileProtocol::new(
            move |peer, file| async move {
                if peer != allowed || file != descriptor.file {
                    return Err(io::Error::from(io::ErrorKind::PermissionDenied));
                }
                Ok(WrongBytes { descriptor })
            },
            2,
            DEADLINE,
        )
        .unwrap();
        let router = Router::builder(server).accept(ALPN, protocol).spawn();
        let mut client = Client::connect(&client_endpoint, router.endpoint().addr(), DEADLINE)
            .await
            .unwrap();
        assert_eq!(
            receive_page(&mut client, &sink, 0, 3)
                .await
                .unwrap()
                .received,
            3
        );
        assert!(sink.seal().await.is_err());
        assert!(
            graph
                .commit_with_blobs(&proposal(id), &[id], |_| Ok(()))
                .is_err()
        );
        assert_eq!(graph.head(&upload().namespace).unwrap(), None);
        assert_eq!(
            graph.files().progress(upload()).unwrap().unwrap().state,
            State::Staging
        );
        assert!(graph.files().cancel(upload(), 3).unwrap());
        assert!(FileSink::open(graph.files(), upload()).await.is_err());
        drop(client);
        router.shutdown().await.unwrap();
        client_endpoint.close().await;
    }
}
