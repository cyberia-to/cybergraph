//! Strict reader for the old shell/soma JSONL content sidecar.
use super::*;
pub const MAX_SOURCE_BYTES: usize = 256 * 1024 * 1024;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportReport {
    pub source: Particle,
    pub observations: u64,
    pub bytes: u64,
    pub head: Option<Head>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyLine {
    particle: String,
    text: String,
    created: Option<u64>,
}
fn malformed(line: u64, why: &str) -> Error {
    Error::Rejected(format!("legacy text line {line}: {why}"))
}
fn decode(raw: &[u8], line: u64) -> Result<LegacyLine, Error> {
    if raw.len() > MAX_TEXT_BYTES {
        return Err(malformed(line, "line byte limit"));
    }
    if raw.last() != Some(&b'\n') {
        return Err(malformed(line, "unterminated source line"));
    }
    // The former writer escaped backslash/quote/LF but emitted literal tabs and
    // CR inside strings. Decode that exact compatibility form; retain raw bytes.
    let mut normalized = Vec::with_capacity(raw.len());
    let mut quoted = false;
    let mut escaped = false;
    for &c in raw {
        if quoted && !escaped && matches!(c, b'\t' | b'\r') {
            normalized.extend(if c == b'\t' { b"\\t" } else { b"\\r" });
            continue;
        }
        normalized.push(c);
        if escaped {
            escaped = false;
            continue;
        }
        if quoted && c == b'\\' {
            escaped = true;
        } else if c == b'"' {
            quoted = !quoted;
        }
    }
    let parsed: LegacyLine =
        serde_json::from_slice(&normalized).map_err(|e| malformed(line, &e.to_string()))?;
    if parsed.particle.len() != 64 || !parsed.particle.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(malformed(line, "particle must contain 64 hex digits"));
    }
    let mut id = [0; 32];
    for (i, b) in id.iter_mut().enumerate() {
        *b = u8::from_str_radix(&parsed.particle[i * 2..i * 2 + 2], 16)
            .map_err(|_| malformed(line, "invalid particle"))?;
    }
    if *hemera::hash(parsed.text.as_bytes()).as_bytes() != id {
        return Err(malformed(line, "text hash mismatch"));
    }
    Ok(parsed)
}
impl TextArchive {
    /// Resolve the final line's original import receipt without republishing it.
    pub fn has_import(&self, source: Particle, lines: u64) -> Result<bool, Error> {
        if lines == 0 {
            return Ok(true);
        }
        let mut hash = hemera::Hasher::new();
        hash.update(b"cybergraph/text-import-request/1\0");
        hash.update(&source);
        hash.update(&lines.to_le_bytes());
        let request = *hash.finalize().as_bytes();
        let Some(head) = self.graph.resolve(&self.namespace, &request)? else {
            return Ok(false);
        };
        let record = self.read(head.commit)?;
        Ok(record.request == request
            && record
                .provenance
                .is_some_and(|p| p.source == source && p.line == lines))
    }
    /// Validate the complete source, atomic observation budgets and resulting
    /// projection before publishing. Each publication is idempotent. Persistence
    /// failure or concurrent append returns an error; retry resumes via line keys.
    pub fn import_jsonl(&self, bytes: &[u8]) -> Result<ImportReport, Error> {
        if bytes.len() > MAX_SOURCE_BYTES {
            return Err(Error::InvalidProposal);
        }
        let source = *hemera::hash(bytes).as_bytes();
        let start = self.graph.head(&self.namespace)?;
        let (mut sizes, mut projected_bytes) = self.projection_sizes(start)?;
        let mut expected = start;
        // Simulate exact observation heads without writing. In particular, a
        // valid late JSON line must not discover storage/projection limits only
        // after earlier lines have already been published.
        for (i, line) in bytes.split_inclusive(|b| *b == b'\n').enumerate() {
            let number = i as u64 + 1;
            let record = decode(line, number)?;
            let prepared = self
                .prepare_import_line(source, number, line, &record.text, record.created, expected)
                .map_err(|e| malformed(number, &e.to_string()))?;
            Self::extend_projection(
                &mut sizes,
                &mut projected_bytes,
                *hemera::hash(record.text.as_bytes()).as_bytes(),
                record.text.len(),
            )
            .map_err(|e| malformed(number, &e.to_string()))?;
            if let PreparedObservation::Append(proposal) = prepared {
                expected = Some(proposal.head);
            }
        }
        let mut head = None;
        let mut observations = 0;
        let mut expected = start;
        for (i, line) in bytes.split_inclusive(|b| *b == b'\n').enumerate() {
            let number = i as u64 + 1;
            let record = decode(line, number)?;
            let prepared = self.prepare_import_line(
                source,
                number,
                line,
                &record.text,
                record.created,
                expected,
            )?;
            let next = match &prepared {
                PreparedObservation::Append(proposal) => Some(proposal.head),
                PreparedObservation::Existing(_) => expected,
            };
            head = Some(self.publish(prepared).map_err(|error| Error::Rejected(format!(
                "legacy text import {source:02x?}: stopped at line {number} after {observations} resolved lines; retry the exact retained source: {error}"
            )))?);
            expected = next;
            observations += 1;
        }
        Ok(ImportReport {
            source,
            observations,
            bytes: bytes.len() as u64,
            head,
        })
    }
}
