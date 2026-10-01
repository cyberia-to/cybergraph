use super::*;

const SOURCE_DOMAIN: &[u8] = b"cybergraph/legacy-source/v1\0";

#[derive(Debug)]
pub struct ImportReport {
    pub events: u64,
    pub signals: u64,
    pub height: u64,
    pub root: Particle,
    pub source: Particle,
}

pub(super) struct Marker {
    pub complete: bool,
    pub source: Particle,
    pub events: u64,
}

fn encode_marker(m: &Marker) -> Vec<u8> {
    let mut b = b"CGIMPORT\x01".to_vec();
    b.push(u8::from(m.complete));
    b.extend(m.source);
    b.extend(m.events.to_le_bytes());
    b
}

pub(super) fn decode_marker(bytes: &[u8]) -> Result<Marker, Error> {
    let mut r = codec::Reader::new(bytes);
    r.expect(b"CGIMPORT\x01")?;
    let complete = match r.byte()? {
        0 => false,
        1 => true,
        _ => return Err(corrupt("import status")),
    };
    let source = r.array()?;
    let events = r.u64()?;
    if events > foculus::frames::MAX_LEGACY_EVENTS as u64 {
        return Err(corrupt("import event limit"));
    }
    r.end()?;
    Ok(Marker {
        complete,
        source,
        events,
    })
}

fn source_hasher() -> hemera::Hasher {
    let mut hash = hemera::Hasher::new();
    hash.update(SOURCE_DOMAIN);
    hash
}

fn source(log: &[u8]) -> Particle {
    let mut hash = source_hasher();
    hash.update(log);
    *hash.finalize().as_bytes()
}

fn request_id(source: &Particle, batch: u64) -> Particle {
    digest(
        b"cybergraph/import-request/v1\0",
        &[source, &batch.to_le_bytes()],
    )
}

#[derive(Default)]
pub(super) struct Progress {
    events: usize,
    batches: u64,
    bytes: usize,
}

/// Check the source assertion against the actual imported history prefix.
/// For interrupted imports the caller also supplies the complete source bytes.
pub(super) fn validate_import(db: &Database, source_log: Option<&[u8]>) -> Result<Progress, Error> {
    let marker = db
        .read_record(D::NativeMetadata, b"import", 128)?
        .map(|bytes| decode_marker(&bytes))
        .transpose()?;
    let mut progress = Progress::default();
    let mut hash = source_hasher();
    let mut after = None;
    let mut ordinary = false;
    loop {
        let rows = db.scan_records(D::NativeHistory, after.as_deref(), b"", recovery::page())?;
        if rows.is_empty() {
            break;
        }
        for (key, bytes) in rows {
            let history = codec::decode_history(&bytes)?;
            if let Some(frames) = &history.original_wire {
                let marker = marker
                    .as_ref()
                    .ok_or_else(|| corrupt("imported frames without source marker"))?;
                if ordinary
                    || key != (progress.batches + 1).to_be_bytes()
                    || history.receipt.request_id != request_id(&marker.source, progress.batches)
                    || history.receipt.timestamp != 0
                    || frames.is_empty()
                    || history.receipt.applied != frames.len() as u64
                {
                    return Err(corrupt("invalid imported history prefix"));
                }
                for frame in frames {
                    let end = progress
                        .bytes
                        .checked_add(frame.len())
                        .ok_or_else(|| corrupt("import byte overflow"))?;
                    if end > foculus::frames::MAX_LEGACY_LOG_BYTES || frame.is_empty() {
                        return Err(corrupt("import byte limit"));
                    }
                    if let Some(log) = source_log
                        && log.get(progress.bytes..end) != Some(frame.as_slice())
                    {
                        return Err(corrupt("committed import prefix differs from source"));
                    }
                    hash.update(frame);
                    progress.bytes = end;
                    progress.events += 1;
                    if progress.events as u64 > marker.events {
                        return Err(corrupt("extra imported event"));
                    }
                }
                progress.batches += 1;
            } else {
                ordinary = true;
                if let Some(marker) = &marker
                    && (!marker.complete || progress.events as u64 != marker.events)
                {
                    return Err(corrupt("ordinary history precedes completed import"));
                }
            }
            after = Some(key);
        }
    }
    if let Some(marker) = marker {
        if marker.complete && progress.events as u64 != marker.events {
            return Err(corrupt("missing imported events"));
        }
        if progress.events as u64 == marker.events {
            if hash.finalize().as_bytes() != &marker.source {
                return Err(corrupt("import source digest differs from exact prefix"));
            }
            if source_log.is_some_and(|log| progress.bytes != log.len()) {
                return Err(corrupt("imported prefix omits source bytes"));
            }
        }
    }
    Ok(progress)
}

impl NativeNode {
    pub fn legacy_source_matches(&self, log: &[u8]) -> Result<bool, Error> {
        let Some(bytes) = self.db.read_record(D::NativeMetadata, b"import", 128)? else {
            return Ok(false);
        };
        let marker = decode_marker(&bytes)?;
        Ok(marker.complete && marker.source == source(log))
    }

    /// Strict import preserves the source and resumes only its exact committed prefix.
    pub fn import_legacy(path: &Path, genesis: &[u8], log: &[u8]) -> Result<ImportReport, Error> {
        let decoded = foculus::frames::decode_events_strict_with_spans(log)?;
        let total_events = decoded.len() as u64;
        let source = source(log);
        let db = Database::open(path, Backend::Ssd)?;
        // Validate the pinned configuration and uninitialized domains before
        // an import marker can change an existing store's readiness.
        recovery::configuration(&db, genesis)?;
        let existing = db.read_record(D::NativeMetadata, b"import", 128)?;
        let mut node = if let Some(bytes) = existing {
            let marker = decode_marker(&bytes)?;
            if marker.source != source || marker.events != total_events {
                return Err(corrupt(
                    "legacy import source differs; source files are never overwritten",
                ));
            }
            let node = Self::open_database(db, genesis, !marker.complete)?;
            if marker.complete {
                validate_import(&node.db, Some(log))?;
                return Ok(report(&node, total_events, source));
            }
            node
        } else {
            let node = Self::open_database(db, genesis, false)?;
            if node.head.position != 0 {
                return Err(invalid("legacy import requires an empty native store"));
            }
            node.db.transaction::<_, Error>(|tx| {
                tx.put_record(
                    D::NativeMetadata,
                    b"import",
                    &encode_marker(&Marker {
                        complete: false,
                        source,
                        events: total_events,
                    }),
                )?;
                Ok(())
            })?;
            node
        };
        let progress = validate_import(&node.db, Some(log))?;
        let mut cursor = progress.events;
        let mut batch = progress.batches;
        while cursor < decoded.len() {
            let mut count = 0;
            let mut bytes = 10; // CGOP version, event tag and event count.
            for (_, event) in &decoded[cursor..] {
                let size = event_size(event)?;
                if size + 10 > MAX_OPERATION_BYTES {
                    return Err(invalid("legacy event exceeds native operation byte limit"));
                }
                if count == MAX_EVENTS || size > MAX_OPERATION_BYTES - bytes {
                    break;
                }
                count += 1;
                bytes += size;
            }
            loop {
                let chunk = &decoded[cursor..cursor + count];
                let operation =
                    Operation::Events(chunk.iter().map(|(_, event)| clone_event(event)).collect());
                let command = codec::operation(&operation)?;
                let frames = chunk
                    .iter()
                    .map(|(span, _)| log[span.clone()].to_vec())
                    .collect();
                match node.accept_encoded(
                    Some(request_id(&source, batch)),
                    operation,
                    command,
                    0,
                    Some(frames),
                    None,
                ) {
                    Ok(_) => break,
                    Err(error) if count > 1 && known_limit(&error) => count /= 2,
                    Err(error) => return Err(error),
                }
            }
            cursor += count;
            batch += 1;
        }
        if node.head.position != batch {
            return Err(corrupt("import destination has history outside its source"));
        }
        node.validate_records()?;
        validate_import(&node.db, Some(log))?;
        node.db.transaction::<_, Error>(|tx| {
            tx.put_record(
                D::NativeMetadata,
                b"import",
                &encode_marker(&Marker {
                    complete: true,
                    source,
                    events: total_events,
                }),
            )?;
            Ok(())
        })?;
        Ok(report(&node, total_events, source))
    }
}

fn report(node: &NativeNode, events: u64, source: Particle) -> ImportReport {
    ImportReport {
        events,
        signals: node.head.signals,
        height: node.height(),
        root: node.root(),
        source,
    }
}

fn event_size(event: &foculus::CyberFrame) -> Result<usize, Error> {
    Ok(match event {
        foculus::CyberFrame::Signal(signal) => {
            5 + foculus::signal_codec::encode_signal(signal)?.len()
        }
        foculus::CyberFrame::Intent(_) => 137,
    })
}

fn clone_event(event: &foculus::CyberFrame) -> Event {
    match event {
        foculus::CyberFrame::Signal(signal) => Event::Signal(signal.clone()),
        foculus::CyberFrame::Intent(i) => Event::Intent(IntentRecord {
            neuron: i.neuron,
            h0: i.h0,
            scope_hash: i.scope_hash,
            signature: i.signature,
        }),
    }
}

fn known_limit(error: &Error) -> bool {
    matches!(
        error,
        Error::Limit(_) | Error::Storage(StorageError::Limit(_))
    )
}
