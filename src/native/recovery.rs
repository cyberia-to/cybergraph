use super::*;
use bbg::storage::database::RecordLimits;

pub(super) const DOMAINS: [D; 7] = [
    D::NativeState,
    D::NativeHistory,
    D::NativeRequests,
    D::NativeMetadata,
    D::NativeBalances,
    D::NativeBlocks,
    D::NativeExport,
];

impl NativeNode {
    pub(super) fn open_database(
        db: Database,
        genesis: &[u8],
        importing: bool,
    ) -> Result<Self, Error> {
        let (config, fresh) = configuration(&db, genesis)?;
        let instance = digest(b"cybergraph/native-instance/v1\0", &[&config]);
        let import = db.read_record(D::NativeMetadata, b"import", 128)?;
        if let Some(value) = &import {
            let status = super::import::decode_marker(value)?;
            if !importing && !status.complete {
                return Err(corrupt(
                    "legacy import incomplete; resume explicit import with the same source",
                ));
            }
        }
        let mut node = Self {
            db,
            graph: Cybergraph::new(),
            balances: BTreeMap::new(),
            supply: 0,
            head: Head::default(),
            instance,
            state_metadata: Vec::new(),
        };
        if fresh {
            node.db.transaction::<_, Error>(|tx| {
                tx.put_record(D::NativeMetadata, b"config", &config)?;
                tx.put_record(D::NativeMetadata, b"head", &codec::head(&node.head))?;
                for record in
                    bbg::transition::records(&node.graph.bbg).map_err(|e| corrupt(e.to_string()))?
                {
                    tx.put_record(D::NativeState, &record.key, &record.value)?;
                }
                Ok(())
            })?;
        }
        node.state_metadata = super::format::read(&node.db)?;
        let recovered = (|| {
            let disk_head = node
                .db
                .read_record(D::NativeMetadata, b"head", 256)?
                .ok_or_else(|| corrupt("missing native head"))?;
            let expected_head = codec::decode_head(&disk_head)?;
            let mut after: Option<Vec<u8>> = None;
            loop {
                let rows = node
                    .db
                    .scan_records(D::NativeHistory, after.as_deref(), b"", page())?;
                if rows.is_empty() {
                    break;
                }
                for (key, bytes) in rows {
                    let expected = node
                        .head
                        .position
                        .checked_add(1)
                        .ok_or_else(|| corrupt("history overflow"))?;
                    if key != expected.to_be_bytes() {
                        return Err(corrupt("noncontiguous native history"));
                    }
                    let h = codec::decode_history(&bytes)?;
                    let operation = codec::decode_operation(&h.command)?;
                    node.accept_encoded(
                        Some(h.receipt.request_id),
                        operation,
                        h.command.clone(),
                        h.receipt.timestamp,
                        h.original_wire.clone(),
                        Some(&h),
                    )
                    .map_err(|e| corrupt(format!("request {expected}: {e}")))?;
                    after = Some(key);
                }
            }
            if node.head != expected_head {
                return Err(corrupt("replayed native head differs from disk"));
            }
            node.validate_records()?;
            Ok(())
        })();
        recovered.map_err(|error| super::format::recovery_error(&node.state_metadata, error))?;
        Ok(node)
    }

    pub(super) fn validate_records(&self) -> Result<(), Error> {
        let records =
            bbg::transition::records(&self.graph.bbg).map_err(|e| corrupt(e.to_string()))?;
        compare(&self.db, D::NativeState, records.map(|r| (r.key, r.value)))?;
        compare(
            &self.db,
            D::NativeBalances,
            self.balances
                .iter()
                .map(|(k, v)| (k.to_vec(), v.to_le_bytes().to_vec())),
        )?;
        if count(&self.db, D::NativeRequests)? != self.head.position {
            return Err(corrupt("extra request receipt"));
        }
        if count(&self.db, D::NativeBlocks)? != self.height() {
            return Err(corrupt("extra block record"));
        }
        let mut offset = 0u64;
        let mut after = None;
        loop {
            let rows = self
                .db
                .scan_records(D::NativeExport, after.as_deref(), b"", page())?;
            if rows.is_empty() {
                break;
            }
            for (key, value) in rows {
                if key != offset.to_be_bytes() {
                    return Err(corrupt("noncontiguous compatibility export"));
                }
                offset = offset
                    .checked_add(value.len() as u64)
                    .ok_or_else(|| corrupt("wire offset overflow"))?;
                after = Some(key);
            }
        }
        if offset != self.head.wire_end {
            return Err(corrupt("wire endpoint mismatch"));
        }
        let metadata = self.db.scan_records(D::NativeMetadata, None, b"", page())?;
        if metadata
            .iter()
            .any(|(key, _)| !matches!(key.as_slice(), b"head" | b"config" | b"import"))
        {
            return Err(corrupt("unknown native metadata"));
        }
        super::import::validate_import(&self.db, None)?;
        Ok(())
    }
}

pub(super) fn page() -> RecordLimits {
    RecordLimits {
        max_entries: 256,
        max_bytes: 32 * 1024 * 1024,
    }
}

pub(super) fn configuration(db: &Database, genesis: &[u8]) -> Result<(Vec<u8>, bool), Error> {
    if genesis.is_empty() || genesis.len() > 4096 {
        return Err(invalid("genesis byte limit"));
    }
    let mut config = b"CGCONFIG\x01".to_vec();
    codec::bytes_out(&mut config, PROFILE)?;
    codec::bytes_out(&mut config, genesis)?;
    match db.read_record(D::NativeMetadata, b"config", 8192)? {
        Some(existing) if existing != config => Err(corrupt(
            "genesis or execution profile differs from durable store",
        )),
        Some(_) => Ok((config, false)),
        None => {
            for domain in DOMAINS {
                if !db.scan_records(domain, None, b"", page())?.is_empty() {
                    return Err(corrupt("uninitialized native store contains records"));
                }
            }
            Ok((config, true))
        }
    }
}
fn compare<I: Iterator<Item = (Vec<u8>, Vec<u8>)>>(
    db: &Database,
    domain: D,
    mut expected: I,
) -> Result<(), Error> {
    let mut after = None;
    loop {
        let rows = db.scan_records(domain, after.as_deref(), b"", page())?;
        if rows.is_empty() {
            break;
        }
        for row in rows {
            let matches = expected.next().is_some_and(|(key, value)| {
                key == row.0
                    && if domain == D::NativeState && key == [0] {
                        super::format::equivalent(&row.1, &value)
                    } else {
                        value == row.1
                    }
            });
            if !matches {
                return Err(corrupt(format!("exact state mismatch in {domain:?}")));
            }
            after = Some(row.0);
        }
    }
    if expected.next().is_some() {
        return Err(corrupt(format!("missing state in {domain:?}")));
    }
    Ok(())
}
fn count(db: &Database, domain: D) -> Result<u64, Error> {
    let mut after = None;
    let mut count = 0u64;
    loop {
        let rows = db.scan_records(domain, after.as_deref(), b"", page())?;
        if rows.is_empty() {
            return Ok(count);
        }
        for (key, _) in rows {
            count = count
                .checked_add(1)
                .ok_or_else(|| corrupt("record count overflow"))?;
            after = Some(key);
        }
    }
}
