use crate::{blocking, storage};
use cybergraph::files::{FileInfo, Files, Spec, State, Upload, blob_profile};
use radio::files::{Client, Descriptor, FileId, MAX_RANGE_BYTES, Sink};
use std::{fmt, io};

/// One persisted upload. Its namespace and request identity never enter framing.
#[derive(Clone)]
pub struct FileSink {
    files: Files,
    upload: Upload,
    spec: Spec,
}
impl fmt::Debug for FileSink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FileSink").finish_non_exhaustive()
    }
}
impl FileSink {
    /// Attach to a previously admitted upload, including after process restart.
    pub async fn open(files: Files, upload: Upload) -> io::Result<Self> {
        blocking(move || {
            let progress = files
                .progress(upload)
                .map_err(storage)?
                .ok_or_else(|| io::Error::from(io::ErrorKind::NotFound))?;
            if progress.state == State::Cancelled
                || progress.spec.profile != blob_profile()
                || progress.spec.part_bytes as usize > MAX_RANGE_BYTES
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "incompatible file upload",
                ));
            }
            Ok(Self {
                files,
                upload,
                spec: progress.spec,
            })
        })
        .await
    }

    /// Fully verify retained parts through bounded blocking steps. Publication
    /// and retention remain an explicit application operation after this call.
    pub async fn seal(&self) -> io::Result<FileInfo> {
        let files = self.files.clone();
        let upload = self.upload;
        let mut verification = blocking(move || files.verify(upload).map_err(storage)).await?;
        loop {
            let (next, complete) = blocking(move || {
                let result = verification.step(1).map_err(storage)?;
                Ok((verification, result))
            })
            .await?;
            if let Some(info) = complete {
                return Ok(info);
            }
            verification = next;
        }
    }
}
impl Sink for FileSink {
    fn descriptor(&self) -> Descriptor {
        Descriptor {
            file: FileId {
                particle: self.spec.particle,
                profile: self.spec.profile,
            },
            length: self.spec.length,
        }
    }
    async fn write(&self, offset: u64, bytes: Vec<u8>) -> io::Result<()> {
        let part = u64::from(self.spec.part_bytes);
        if offset >= self.spec.length
            || !offset.is_multiple_of(part)
            || bytes.len() as u64 != part.min(self.spec.length - offset)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "misaligned file part",
            ));
        }
        let files = self.files.clone();
        let upload = self.upload;
        blocking(move || {
            files
                .write_part(upload, offset / part, &bytes)
                .map_err(storage)
        })
        .await
    }
}

/// Progress through one bounded coverage page, independent of total file size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Page {
    /// Exclusive end of the inspected page, or complete coverage traversal.
    pub next: Option<u64>,
    /// Successful deliveries, including idempotent concurrent writes. This is
    /// local progress, independent of remote retention.
    pub received: u64,
}

/// Request only missing parts in a bounded coverage page, sequentially. The
/// scheduler owns retry, peer choice and advancing/resuming the returned cursor.
pub async fn receive_page(
    client: &mut Client,
    sink: &FileSink,
    from: u64,
    limit: usize,
) -> io::Result<Page> {
    let files = sink.files.clone();
    let upload = sink.upload;
    let coverage = blocking(move || files.coverage(upload, from, limit).map_err(storage)).await?;
    let mut received = 0;
    for (index, present) in coverage.present {
        if present {
            continue;
        }
        let offset = index * u64::from(sink.spec.part_bytes);
        let length = (sink.spec.length - offset).min(u64::from(sink.spec.part_bytes)) as usize;
        client.deliver(sink, offset, length).await?;
        received += 1;
    }
    Ok(Page {
        next: coverage.next,
        received,
    })
}
