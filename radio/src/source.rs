use crate::{blocking, storage};
use cybergraph::{Particle, files::Files};
use radio::files::{Descriptor, FileId, Source};
use std::{fmt, io};

/// One sealed file in one host-authorized local namespace.
#[derive(Clone)]
pub struct FileSource {
    files: Files,
    namespace: Particle,
    descriptor: Descriptor,
}
impl fmt::Debug for FileSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSource").finish_non_exhaustive()
    }
}
impl FileSource {
    /// Call only after the host has authorized the peer and file.
    pub async fn open(files: Files, namespace: Particle, particle: Particle) -> io::Result<Self> {
        blocking(move || {
            let info = files
                .info(namespace, particle)
                .map_err(storage)?
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            Ok(Self {
                files,
                namespace,
                descriptor: Descriptor {
                    file: FileId {
                        particle,
                        profile: info.spec.profile,
                    },
                    length: info.spec.length,
                },
            })
        })
        .await
    }
}
impl Source for FileSource {
    fn descriptor(&self) -> Descriptor {
        self.descriptor
    }
    async fn read(&self, offset: u64, length: usize) -> io::Result<Vec<u8>> {
        let files = self.files.clone();
        let namespace = self.namespace;
        let particle = self.descriptor.file.particle;
        blocking(move || {
            files
                .read_range(namespace, particle, offset, length)
                .map_err(storage)
        })
        .await
    }
}
