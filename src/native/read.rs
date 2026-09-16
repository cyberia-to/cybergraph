use super::*;
use bbg::storage::database::{MAX_BYTES_VALUE, RecordLimits};

impl NativeNode {
    /// Resolve an exact command retry without preparing or applying it.
    pub fn resolve(&self, request: Particle, operation: &Operation) -> Result<Option<Receipt>, Error> {
        let command = codec::operation(operation)?;
        let fingerprint = digest(b"cybergraph/native-command/v1\0", &[&self.instance, &command]);
        self.db.read_record(D::NativeRequests, &request, 256)?
            .map(|bytes| super::commit::check_retry(&bytes, &fingerprint, &request)).transpose()
    }
    pub fn block(&self, height: u64) -> Result<Option<Block>, Error> {
        let Some(bytes) =
            self.db
                .read_record(D::NativeBlocks, &height.to_be_bytes(), MAX_BYTES_VALUE)?
        else {
            if height > 0 && height <= self.height() {
                return Err(corrupt("missing committed block"));
            }
            return Ok(None);
        };
        let block = codec::decode_block(&bytes)?;
        if block.height != height {
            return Err(corrupt("block height differs from key"));
        }
        Ok(Some(block))
    }

    /// Newest first; `before` is an exclusive height cursor.
    pub fn blocks(&self, before: Option<u64>, limit: usize) -> Result<Vec<BlockSummary>, Error> {
        if limit == 0 || limit > 500 {
            return Err(invalid("block page limit must be 1..500"));
        }
        let mut height = before
            .unwrap_or(self.height().saturating_add(1))
            .saturating_sub(1)
            .min(self.height());
        let mut result = Vec::new();
        while height > 0 && result.len() < limit {
            let block = self
                .block(height)?
                .ok_or_else(|| corrupt("missing committed block"))?;
            result.push(BlockSummary {
                height: block.height,
                timestamp: block.timestamp,
                supply: block.supply,
                weight: block.weight,
                root: block.root,
            });
            height -= 1;
        }
        Ok(result)
    }

    /// Compatibility tape bytes. Cursors must identify complete frame boundaries.
    pub fn wire_log(&self, from: usize, limit: usize) -> Result<Vec<u8>, Error> {
        if limit == 0 || limit > 16 * 1024 * 1024 {
            return Err(invalid("wire page byte limit"));
        }
        let from = u64::try_from(from).map_err(|_| invalid("wire cursor"))?;
        if from > self.head.wire_end {
            return Err(invalid("wire cursor exceeds committed log"));
        }
        if from == self.head.wire_end {
            if self.head.wire_blocked {
                return Err(Error::Unsupported(
                    "legacy wire cannot represent retained network/proof; use native history"
                        .into(),
                ));
            }
            // Even an empty read observes a poisoned owner.
            self.db.read_record(D::NativeMetadata, b"head", 256)?;
            return Ok(vec![]);
        }
        let first = self
            .db
            .read_record(D::NativeExport, &from.to_be_bytes(), MAX_BYTES_VALUE)?
            .ok_or_else(|| invalid("wire cursor is inside a frame"))?;
        if first.len() > limit {
            return Err(invalid("wire page cannot fit one complete frame"));
        }
        let mut out = first;
        let mut after = from.to_be_bytes().to_vec();
        while out.len() < limit {
            let rows = self.db.scan_records(
                D::NativeExport,
                Some(&after),
                b"",
                RecordLimits {
                    max_entries: 64,
                    max_bytes: 32 * 1024 * 1024,
                },
            )?;
            if rows.is_empty() {
                break;
            }
            for (key, bytes) in rows {
                if key != ((from + out.len() as u64).to_be_bytes()) {
                    return Err(corrupt("wire gap"));
                }
                if bytes.len() > limit - out.len() {
                    return Ok(out);
                }
                out.extend(bytes);
                after = key;
            }
        }
        Ok(out)
    }

    /// Full canonical operations, ordered by request position, for durable export.
    pub fn history(
        &self,
        after: Option<u64>,
        limit: usize,
    ) -> Result<Vec<(Receipt, Vec<u8>)>, Error> {
        if limit == 0 || limit > 64 {
            return Err(invalid("history page limit must be 1..64"));
        }
        let cursor = after.map(u64::to_be_bytes);
        let rows = self.db.scan_records(
            D::NativeHistory,
            cursor.as_ref().map(|v| v.as_slice()),
            b"",
            RecordLimits {
                max_entries: limit,
                max_bytes: 16 * 1024 * 1024,
            },
        )?;
        let mut expected = after.unwrap_or(0).saturating_add(1);
        if rows.is_empty() && expected <= self.head.position {
            return Err(corrupt("missing committed history page"));
        }
        rows.into_iter()
            .map(|(key, bytes)| {
                let h = codec::decode_history(&bytes)?;
                if key != h.receipt.position.to_be_bytes() || h.receipt.position != expected {
                    return Err(corrupt("history position differs from key"));
                }
                expected = expected.saturating_add(1);
                Ok((h.receipt, h.command))
            })
            .collect()
    }
}
