use super::*;
use bbg::storage::database::Transaction;
use bbg::transition::NativeChange;
use codec::History;

impl NativeNode {
    pub(super) fn accept_encoded(
        &mut self,
        request_id: Option<Particle>,
        operation: Operation,
        command: Vec<u8>,
        timestamp: u64,
        original_wire: Option<Vec<Vec<u8>>>,
        replay: Option<&History>,
    ) -> Result<Receipt, Error> {
        let fingerprint = digest(
            b"cybergraph/native-command/v1\0",
            &[&self.instance, &command],
        );
        let position = self
            .head
            .position
            .checked_add(1)
            .ok_or_else(|| invalid("history position overflow"))?;
        let id = request_id.unwrap_or_else(|| {
            digest(
                b"cybergraph/server-request/v1\0",
                &[&self.instance, &position.to_le_bytes(), &fingerprint],
            )
        });
        if replay.is_none()
            && let Some(bytes) = self.db.read_record(D::NativeRequests, &id, 256)?
        {
            return check_retry(&bytes, &fingerprint, &id);
        }
        let (events, payer) = self.events(operation)?;
        self.validate_chains(&events)?;
        let ledger = economics::Ledger::prepare(self, &events)?;
        let balance = payer.map(|n| ledger.balance(self, &n));
        let bridges: Vec<_> = events
            .iter()
            .map(|e| match e {
                Event::Signal(s) => Some(bridge(s)),
                Event::Intent(_) => None,
            })
            .collect();
        let mut signals = self.head.signals;
        let mut headers = Vec::with_capacity(events.len());
        for event in &events {
            headers.push(match event {
                Event::Signal(s) => {
                    let position = signals;
                    signals = signals
                        .checked_add(1)
                        .ok_or_else(|| invalid("signal count overflow"))?;
                    let network = if s.network == SELF_NETWORK {
                        crate::api::private_network(&s.neuron)
                    } else {
                        s.network
                    };
                    Some((
                        position,
                        bbg::SignalRecord {
                            neuron: s.neuron,
                            network,
                            link_count: u32::try_from(s.links.len())
                                .map_err(|_| invalid("link count"))?,
                            block_height: s.height,
                            proof_hash: foculus::signal_codec::proof_hash(s)?,
                        },
                    ))
                }
                Event::Intent(_) => None,
            });
        }
        let changes: Vec<_> = events
            .iter()
            .zip(&bridges)
            .zip(&headers)
            .map(|((event, signal), header)| match event {
                Event::Signal(_) => {
                    let (position, header) = header.as_ref().expect("signal header");
                    NativeChange::Signal {
                        signal: signal.as_ref().expect("signal bridge"),
                        position: *position,
                        header,
                    }
                }
                Event::Intent(i) => NativeChange::Intent(i),
            })
            .collect();
        let (wire, wire_blocked) =
            wire_frames(&events, original_wire.as_ref(), self.head.wire_blocked)?;
        let mut wire_end = self.head.wire_end;
        let mut exports = Vec::with_capacity(wire.len());
        for frame in wire {
            let start = wire_end;
            wire_end = wire_end
                .checked_add(frame.len() as u64)
                .ok_or_else(|| invalid("wire offset overflow"))?;
            exports.push((start.to_be_bytes(), frame));
        }
        let prepared = self
            .graph
            .bbg
            .prepare_native_batch(&changes)
            .map_err(|e| match e {
                bbg::transition::Error::Limit(limit) => Error::Limit(limit.into()),
                error => invalid(error.to_string()),
            })?;
        let weight = ledger
            .observations
            .iter()
            .try_fold(0u64, |n, (_, w)| n.checked_add(*w))
            .ok_or_else(|| invalid("request weight overflow"))?;
        let receipt = Receipt {
            request_id: id,
            position,
            height: prepared.height(),
            root: prepared.root(),
            signals,
            balance,
            supply: ledger.supply,
            weight,
            timestamp,
            applied: events.len() as u64,
        };
        let mut blocks = Vec::new();
        for ((event, snapshot), (supply, weight)) in events
            .iter()
            .zip(prepared.snapshots())
            .zip(&ledger.observations)
        {
            if let Event::Signal(signal) = event {
                let block = Block {
                    height: snapshot.height,
                    root: snapshot.root,
                    timestamp,
                    supply: *supply,
                    weight: *weight,
                    signal: signal.clone(),
                };
                blocks.push((block.height.to_be_bytes(), codec::block(&block)?));
            }
        }
        let history = History {
            fingerprint,
            receipt: receipt.clone(),
            command,
            original_wire,
        };
        let history_bytes = codec::history(&history)?;
        let head = Head {
            position,
            signals,
            wire_end,
            wire_blocked,
            history_hash: digest(
                b"cybergraph/native-history/v1\0",
                &[&self.head.history_hash, &history_bytes],
            ),
        };
        let mut stored_receipt = fingerprint.to_vec();
        stored_receipt.extend(codec::receipt(&receipt));
        if let Some(expected) = replay {
            if codec::history(expected)? != history_bytes {
                return Err(corrupt(format!("history result at request {position}")));
            }
            require_record(&self.db, D::NativeRequests, &id, &stored_receipt)?;
            for (key, value) in &blocks {
                require_record(&self.db, D::NativeBlocks, key, value)?;
            }
            for (key, value) in &exports {
                require_record(&self.db, D::NativeExport, key, value)?;
            }
        } else {
            self.db
                .transaction::<_, Error>(|tx| {
                    if tx.read_record(D::NativeMetadata, b"head", 256)?.as_deref()
                        != Some(codec::head(&self.head).as_slice())
                    {
                        return Err(corrupt("coordinator head changed outside its owner"));
                    }
                    for change in prepared.changes() {
                        match &change.value {
                            Some(value) => tx.put_record(D::NativeState, &change.key, value)?,
                            None => tx.remove_record(D::NativeState, &change.key)?,
                        }
                    }
                    for (key, value) in &ledger.changes {
                        tx.put_record(D::NativeBalances, key, &value.to_le_bytes())?;
                    }
                    for (key, value) in &blocks {
                        insert(tx, D::NativeBlocks, key, value)?;
                    }
                    for (key, value) in &exports {
                        insert(tx, D::NativeExport, key, value)?;
                    }
                    insert(
                        tx,
                        D::NativeHistory,
                        &position.to_be_bytes(),
                        &history_bytes,
                    )?;
                    insert(tx, D::NativeRequests, &id, &stored_receipt)?;
                    tx.put_record(D::NativeMetadata, b"head", &codec::head(&head))?;
                    #[cfg(test)]
                    super::tests::after_staging()?;
                    Ok(())
                })
                .map_err(|error| match error {
                    Error::Storage(StorageError::Limit(limit)) => Error::Limit(limit.into()),
                    error => error,
                })?;
        }
        prepared.publish();
        for event in events {
            if let Event::Signal(signal) = event {
                self.graph
                    .chains
                    .entry(signal.neuron)
                    .or_default()
                    .entries
                    .insert(signal.step, signal);
            }
        }
        self.balances.extend(ledger.changes);
        self.supply = ledger.supply;
        self.head = head;
        Ok(receipt)
    }

    fn validate_chains(&self, events: &[Event]) -> Result<(), Error> {
        let mut positions = BTreeMap::new();
        for event in events {
            if let Event::Signal(signal) = event {
                let (step, prev) = positions
                    .get(&signal.neuron)
                    .copied()
                    .unwrap_or_else(|| self.next_position(&signal.neuron));
                if signal.step != step || signal.prev != prev {
                    return Err(invalid("nonsequential signal step or previous hash"));
                }
                if signal.links.iter().any(|link| link.neuron != signal.neuron) {
                    return Err(invalid("link neuron differs from signal neuron"));
                }
                positions.insert(
                    signal.neuron,
                    (
                        step.checked_add(1)
                            .ok_or_else(|| invalid("neuron step overflow"))?,
                        signal.hash(),
                    ),
                );
            }
        }
        Ok(())
    }
}

fn bridge(s: &Signal) -> bbg::Signal {
    bbg::Signal {
        neuron: s.neuron,
        height: s.height,
        links: s
            .links
            .iter()
            .map(|l| bbg::Cyberlink {
                from: l.from,
                to: l.to,
                token: l.token,
                amount: l.amount,
                valence: l.valence,
            })
            .collect(),
        box_moves: s
            .box_moves
            .iter()
            .map(|m| bbg::BoxMove {
                nullifier: m.nullifier,
                commitment: m.commitment,
            })
            .collect(),
    }
}
fn check_retry(bytes: &[u8], fingerprint: &Particle, id: &Particle) -> Result<Receipt, Error> {
    if bytes.len() < 32 {
        return Err(corrupt("receipt fingerprint"));
    }
    if &bytes[..32] != fingerprint {
        return Err(Error::Conflict);
    }
    let receipt = codec::decode_receipt(&bytes[32..])?;
    if &receipt.request_id != id {
        return Err(corrupt("receipt identity"));
    }
    Ok(receipt)
}
fn insert(tx: &mut Transaction<'_>, domain: D, key: &[u8], value: &[u8]) -> Result<(), Error> {
    if tx
        .read_record(domain, key, bbg::storage::database::MAX_BYTES_VALUE)?
        .is_some()
    {
        return Err(corrupt("immutable native record already exists"));
    }
    tx.put_record(domain, key, value)?;
    Ok(())
}
pub(super) fn require_record(
    db: &Database,
    domain: D,
    key: &[u8],
    value: &[u8],
) -> Result<(), Error> {
    if db
        .read_record(domain, key, bbg::storage::database::MAX_BYTES_VALUE)?
        .as_deref()
        != Some(value)
    {
        return Err(corrupt(format!("record mismatch in {domain:?}")));
    }
    Ok(())
}

fn wire_frames(
    events: &[Event],
    original: Option<&Vec<Vec<u8>>>,
    blocked: bool,
) -> Result<(Vec<Vec<u8>>, bool), Error> {
    if let Some(frames) = original {
        if blocked || frames.len() != events.len() {
            return Err(corrupt("legacy wire span count"));
        }
        for (frame, event) in frames.iter().zip(events) {
            let decoded = foculus::frames::decode_events_strict(frame)?;
            let mapped: Vec<_> = decoded
                .into_iter()
                .map(|e| match e {
                    foculus::CyberFrame::Signal(s) => Event::Signal(s),
                    foculus::CyberFrame::Intent(i) => Event::Intent(i),
                })
                .collect();
            let expected = match event {
                Event::Signal(s) => Operation::Events(vec![Event::Signal(s.clone())]),
                Event::Intent(i) => Operation::Events(vec![Event::Intent(IntentRecord {
                    neuron: i.neuron,
                    h0: i.h0,
                    scope_hash: i.scope_hash,
                    signature: i.signature,
                })]),
            };
            if codec::operation(&Operation::Events(mapped))? != codec::operation(&expected)? {
                return Err(corrupt("legacy bytes differ from operation"));
            }
        }
        return Ok((frames.clone(), false));
    }
    if blocked {
        return Ok((vec![], true));
    }
    let mut frames = Vec::new();
    for event in events {
        match event {
            Event::Signal(s) if s.network != SELF_NETWORK || s.proof.is_some() => {
                return Ok((frames, true));
            }
            Event::Signal(s) => frames.push(foculus::encode_signal_frame(s)),
            Event::Intent(i) => frames.push(foculus::encode_intent_frame(i)),
        }
    }
    Ok((frames, false))
}
