//! Retire an offline legacy append-file path while preserving its exact bytes.
#[cfg(all(
    test,
    any(target_vendor = "apple", target_os = "linux", target_os = "android")
))]
mod tests;
use crate::Particle;
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};
const MARKER: &str = "retirement.json";
const SOURCE: &str = "source";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema: String,
    kind: String,
    hash: Particle,
}
pub struct LegacyFile {
    path: PathBuf,
    stage: PathBuf,
    manifest: Manifest,
    bytes: Vec<u8>,
    retired: bool,
    // Hold the cooperative writer lock across import and retirement.
    source: File,
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn metadata(path: &Path) -> io::Result<Option<std::fs::Metadata>> {
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => {
            Err(invalid("legacy retirement rejects symlink aliases"))
        }
        Ok(m) => Ok(Some(m)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}
fn manifest(path: &Path) -> io::Result<Manifest> {
    if !metadata(&path.join(MARKER))?.is_some_and(|m| m.is_file() && m.len() <= 8192) {
        return Err(invalid("retirement marker is missing or invalid"));
    }
    let mut bytes = Vec::new();
    File::open(path.join(MARKER))?
        .take(8193)
        .read_to_end(&mut bytes)?;
    if bytes.len() > 8192 {
        return Err(invalid("retirement marker byte limit"));
    }
    let m: Manifest = serde_json::from_slice(&bytes).map_err(|e| invalid(&e.to_string()))?;
    if m.schema != "cybergraph/legacy-file-retirement/1" {
        return Err(invalid("unsupported retirement marker"));
    }
    Ok(m)
}
fn sync_dir(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}
fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}
fn rename_new(from: &Path, to: &Path) -> io::Result<()> {
    #[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
    {
        return rustix::fs::renameat_with(
            rustix::fs::CWD,
            from,
            rustix::fs::CWD,
            to,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(Into::into);
    }
    #[cfg(not(any(target_vendor = "apple", target_os = "linux", target_os = "android")))]
    {
        let _ = (from, to);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "atomic file retirement is unsupported on this platform",
        ))
    }
}
impl LegacyFile {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn is_retired(&self) -> bool {
        self.retired
    }
    /// None means absent source or a normal data directory. A discovered staged
    /// retirement is resumed; a retired directory always validates its source.
    pub fn load(path: &Path, kind: &str, max_bytes: usize) -> io::Result<Option<Self>> {
        let mut stage = path.as_os_str().to_os_string();
        stage.push(".retiring-v1");
        let stage = PathBuf::from(stage);
        let current = metadata(path)?;
        let staged = metadata(&stage)?;
        let (location, record, retired) = match current {
            Some(m) if m.is_file() => {
                let record = if staged.is_some() {
                    Some(manifest(&stage)?)
                } else {
                    None
                };
                (path.to_owned(), record, false)
            }
            Some(m) if m.is_dir() => {
                if metadata(&path.join(MARKER))?.is_none() {
                    if staged.is_some() {
                        return Err(invalid("legacy path conflicts with staged retirement"));
                    }
                    return Ok(None);
                }
                if staged.is_some() {
                    return Err(invalid("both retired and staged paths exist"));
                }
                (path.join(SOURCE), Some(manifest(path)?), true)
            }
            Some(_) => return Err(invalid("legacy source is not a regular file")),
            None if staged.is_some() => (stage.join(SOURCE), Some(manifest(&stage)?), false),
            None => return Ok(None),
        };
        let info = metadata(&location)?.ok_or_else(|| invalid("retired source is missing"))?;
        if !info.is_file() || info.len() > max_bytes as u64 {
            return Err(invalid("legacy source size/type limit"));
        }
        let mut source = File::open(&location)?;
        source
            .try_lock()
            .map_err(|e| io::Error::new(io::ErrorKind::WouldBlock, e))?;
        let mut bytes = Vec::new();
        (&mut source)
            .take(max_bytes as u64 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > max_bytes {
            return Err(invalid("legacy source grew beyond byte limit"));
        }
        let expected = Manifest {
            schema: "cybergraph/legacy-file-retirement/1".into(),
            kind: kind.into(),
            hash: *hemera::hash(&bytes).as_bytes(),
        };
        if record.as_ref().is_some_and(|m| m != &expected) {
            return Err(invalid("legacy source or retirement kind changed"));
        }
        Ok(Some(Self {
            path: path.to_owned(),
            stage,
            manifest: expected,
            bytes,
            retired,
            source,
        }))
    }
    /// Call only after the corresponding importer resolved durable acceptance.
    pub fn retire(&mut self) -> io::Result<()> {
        if self.retired {
            return Ok(());
        }
        if metadata(&self.stage)?.is_none() {
            let temp = tempfile::Builder::new()
                .prefix(".graph-retirement-")
                .tempdir_in(parent(&self.path))?;
            let marker = temp.path().join(MARKER);
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(marker)?;
            file.write_all(
                &serde_json::to_vec(&self.manifest).map_err(|e| invalid(&e.to_string()))?,
            )?;
            file.sync_all()?;
            sync_dir(temp.path())?;
            let prepared = temp.keep();
            rename_new(&prepared, &self.stage)?;
            sync_dir(parent(&self.stage))?;
        }
        if manifest(&self.stage)? != self.manifest {
            return Err(invalid("conflicting retirement manifest"));
        }
        let saved = self.stage.join(SOURCE);
        if metadata(&saved)?.is_none() {
            rename_new(&self.path, &saved)?;
        }
        self.source.sync_all()?;
        sync_dir(&self.stage)?;
        sync_dir(parent(&self.path))?;
        // Recheck the moved inode. A changed source is retained for diagnosis,
        // never declared successfully migrated or replaced by an empty file.
        let mut actual = Vec::new();
        File::open(&saved)?
            .take(self.bytes.len() as u64 + 1)
            .read_to_end(&mut actual)?;
        if actual != self.bytes {
            return Err(invalid("legacy source changed during retirement"));
        }
        rename_new(&self.stage, &self.path)?;
        sync_dir(parent(&self.path))?;
        self.retired = true;
        Ok(())
    }
}
