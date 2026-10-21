//! Streamed Blob content over BBG, preserving existing Hemera identities.
use crate::Particle;
pub use bbg::storage::content::{
    Coverage, Error, FileInfo, MAX_PART_BYTES, Progress, Result, Spec, State, Upload, Verification,
    Verifier,
};
use bbg::storage::{StorageError, content::ContentStore, database::Database};
use std::io::{self, Read, Seek, SeekFrom};

/// Internal verifier binding; this preserves Content::Blob, pending S1.
pub fn blob_profile() -> Particle {
    *hemera::hash(b"cybergraph/blob/hemera-sponge").as_bytes()
}

pub struct BlobVerifier(hemera::Hasher);
impl Verifier for BlobVerifier {
    fn profile(&self) -> Particle {
        blob_profile()
    }
    fn update(&mut self, bytes: &[u8]) -> Result<()> {
        self.0.update(bytes);
        Ok(())
    }
    fn finish(self) -> Result<Particle> {
        Ok(*self.0.finalize().as_bytes())
    }
}

/// Trusted local adapter. Hosts enforce namespace authorization before handing
/// out operations; knowing a namespace or particle alone grants no authority.
#[derive(Clone)]
pub struct Files {
    store: ContentStore,
}
impl Files {
    pub fn from_database(database: Database) -> Self {
        Self {
            store: ContentStore::from_database(database),
        }
    }
    pub fn begin(
        &self,
        upload: Upload,
        particle: Particle,
        length: u64,
        part_bytes: u32,
    ) -> Result<Progress> {
        self.store.begin(
            upload,
            Spec {
                particle,
                profile: blob_profile(),
                length,
                part_bytes,
            },
        )
    }
    pub fn progress(&self, upload: Upload) -> Result<Option<Progress>> {
        self.store.progress(upload)
    }
    pub fn uploads(
        &self,
        namespace: Particle,
        after: Option<Particle>,
        limit: usize,
    ) -> Result<Vec<(Upload, Progress)>> {
        self.store.uploads(namespace, after, limit)
    }
    pub fn write_part(&self, upload: Upload, index: u64, bytes: &[u8]) -> Result<()> {
        self.store.write_part(upload, index, bytes)
    }
    pub fn coverage(&self, upload: Upload, from: u64, limit: usize) -> Result<Coverage> {
        self.store.coverage(upload, from, limit)
    }
    pub fn verify(&self, upload: Upload) -> Result<Verification<BlobVerifier>> {
        self.store
            .verify(upload, BlobVerifier(hemera::Hasher::new()))
    }
    pub fn cancel(&self, upload: Upload, limit: usize) -> Result<bool> {
        self.store.cancel(upload, limit)
    }
    pub fn info(&self, namespace: Particle, particle: Particle) -> Result<Option<FileInfo>> {
        let info = self.store.file(namespace, particle)?;
        if info.is_some_and(|i| i.spec.profile != blob_profile()) {
            return Err(Error::ProfileMismatch);
        }
        Ok(info)
    }
    pub fn read_range(
        &self,
        namespace: Particle,
        particle: Particle,
        offset: u64,
        max_bytes: usize,
    ) -> Result<Vec<u8>> {
        self.store
            .read_range(namespace, particle, blob_profile(), offset, max_bytes)
    }
    pub fn is_retained(
        &self,
        namespace: Particle,
        particle: Particle,
        root: Particle,
    ) -> Result<bool> {
        self.store
            .is_retained(namespace, particle, blob_profile(), root)
    }
    pub fn reader(&self, namespace: Particle, particle: Particle) -> Result<FileReader> {
        let info = self.info(namespace, particle)?.ok_or(Error::Missing)?;
        Ok(FileReader {
            files: self.clone(),
            info,
            position: 0,
            cache_start: 0,
            cache: Vec::new(),
        })
    }

    /// Convenience for a local stream with a known expected identity/length.
    /// Errors leave durable progress for inspection or an explicit retry.
    pub fn import(
        &self,
        upload: Upload,
        particle: Particle,
        length: u64,
        part_bytes: u32,
        mut input: impl Read,
    ) -> Result<FileInfo> {
        let progress = self.begin(upload, particle, length, part_bytes)?;
        let mut buffer = vec![0; part_bytes as usize];
        for index in 0..progress.spec.parts() {
            let count =
                (length - index * u64::from(part_bytes)).min(u64::from(part_bytes)) as usize;
            input.read_exact(&mut buffer[..count]).map_err(io_error)?;
            self.write_part(upload, index, &buffer[..count])?;
        }
        let mut extra = [0];
        if input.read(&mut extra).map_err(io_error)? != 0 {
            return Err(Error::InvalidRange);
        }
        let mut verification = self.verify(upload)?;
        loop {
            if let Some(info) = verification.step(1)? {
                return Ok(info);
            }
        }
    }
}

fn io_error(error: io::Error) -> Error {
    StorageError::Io(error.to_string()).into()
}

/// One cached physical part, even when a consumer reads one byte at a time.
/// Sealed files remain protected from reclamation in this implementation.
pub struct FileReader {
    files: Files,
    info: FileInfo,
    position: u64,
    cache_start: u64,
    cache: Vec<u8>,
}
impl FileReader {
    pub fn info(&self) -> FileInfo {
        self.info
    }
}
impl Read for FileReader {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        if buffer.is_empty() || self.position >= self.info.spec.length {
            return Ok(0);
        }
        if self.position < self.cache_start
            || self.position - self.cache_start >= self.cache.len() as u64
        {
            let size = u64::from(self.info.spec.part_bytes);
            let start = self.position / size * size;
            let bytes = self
                .files
                .read_range(
                    self.info.upload.namespace,
                    self.info.spec.particle,
                    start,
                    size as usize,
                )
                .map_err(io::Error::other)?;
            self.cache_start = start;
            self.cache = bytes;
        }
        let start = (self.position - self.cache_start) as usize;
        let count = buffer.len().min(self.cache.len() - start);
        buffer[..count].copy_from_slice(&self.cache[start..start + count]);
        self.position += count as u64;
        Ok(count)
    }
}
impl Seek for FileReader {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        let position = match from {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.info.spec.length) + i128::from(n),
        };
        self.position = u64::try_from(position)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "file seek overflow"))?;
        Ok(self.position)
    }
}
