use super::*;

pub(super) fn operation(op: &Operation) -> Result<Vec<u8>, Error> {
    let mut out = b"CGOP\x01".to_vec();
    match op {
        Operation::Link {
            neuron,
            from,
            to,
            token,
            amount,
            valence,
        } => {
            out.push(0);
            for p in [neuron, from, to, token] {
                out.extend(p);
            }
            u64_out(&mut out, *amount);
            out.push(*valence as u8);
        }
        Operation::Pay { from, to, amount } => {
            out.push(1);
            out.extend(from);
            out.extend(to);
            u64_out(&mut out, *amount);
        }
        Operation::Events(events) => {
            if events.is_empty() || events.len() > MAX_EVENTS {
                return Err(invalid("event count"));
            }
            out.push(2);
            out.extend((events.len() as u32).to_le_bytes());
            for event in events {
                match event {
                    Event::Signal(s) => {
                        out.push(0);
                        let b = foculus::signal_codec::encode_signal(s)?;
                        bytes_out(&mut out, &b)?;
                    }
                    Event::Intent(i) => {
                        out.push(1);
                        out.extend(intent(i));
                    }
                    Event::LocalCredit { neuron, token, amount, focus, reason } => {
                        out.push(2);
                        out.extend(neuron);
                        out.extend(token);
                        u64_out(&mut out, *amount);
                        u64_out(&mut out, *focus);
                        out.extend(reason);
                    }
                }
                if out.len() > MAX_OPERATION_BYTES {
                    return Err(invalid("operation byte limit"));
                }
            }
        }
    }
    Ok(out)
}

pub(super) fn decode_operation(bytes: &[u8]) -> Result<Operation, Error> {
    if bytes.len() > MAX_OPERATION_BYTES {
        return Err(corrupt("operation byte limit"));
    }
    let mut r = Reader::new(bytes);
    r.expect(b"CGOP\x01")?;
    let result = match r.byte()? {
        0 => Operation::Link {
            neuron: r.array()?,
            from: r.array()?,
            to: r.array()?,
            token: r.array()?,
            amount: r.u64()?,
            valence: r.byte()? as i8,
        },
        1 => Operation::Pay {
            from: r.array()?,
            to: r.array()?,
            amount: r.u64()?,
        },
        2 => {
            let n = r.u32()? as usize;
            if n == 0 || n > MAX_EVENTS {
                return Err(corrupt("event count"));
            }
            let mut events = Vec::with_capacity(n);
            for _ in 0..n {
                events.push(match r.byte()? {
                    0 => Event::Signal(foculus::signal_codec::decode_signal(
                        r.bytes(MAX_OPERATION_BYTES)?,
                    )?),
                    1 => Event::Intent(IntentRecord {
                        neuron: r.array()?,
                        h0: r.u64()?,
                        scope_hash: r.array()?,
                        signature: r.array()?,
                    }),
                    2 => Event::LocalCredit { neuron: r.array()?, token: r.array()?,
                        amount: r.u64()?, focus: r.u64()?, reason: r.array()? },
                    _ => return Err(corrupt("event kind")),
                });
            }
            Operation::Events(events)
        }
        _ => return Err(corrupt("operation kind")),
    };
    r.end()?;
    Ok(result)
}

fn intent(i: &IntentRecord) -> Vec<u8> {
    let mut b = Vec::with_capacity(136);
    b.extend(i.neuron);
    u64_out(&mut b, i.h0);
    b.extend(i.scope_hash);
    b.extend(i.signature);
    b
}

pub(super) fn receipt(r: &Receipt) -> Vec<u8> {
    let mut b = b"CGR\x01".to_vec();
    b.extend(r.request_id);
    u64_out(&mut b, r.position);
    u64_out(&mut b, r.height);
    b.extend(r.root);
    u64_out(&mut b, r.signals);
    match r.balance {
        None => b.push(0),
        Some(v) => {
            b.push(1);
            u64_out(&mut b, v);
        }
    }
    for n in [r.supply, r.weight, r.timestamp, r.applied] {
        u64_out(&mut b, n);
    }
    b
}
pub(super) fn decode_receipt(bytes: &[u8]) -> Result<Receipt, Error> {
    let mut r = Reader::new(bytes);
    r.expect(b"CGR\x01")?;
    let value = Receipt {
        request_id: r.array()?,
        position: r.u64()?,
        height: r.u64()?,
        root: r.array()?,
        signals: r.u64()?,
        balance: match r.byte()? {
            0 => None,
            1 => Some(r.u64()?),
            _ => return Err(corrupt("receipt option")),
        },
        supply: r.u64()?,
        weight: r.u64()?,
        timestamp: r.u64()?,
        applied: r.u64()?,
    };
    r.end()?;
    Ok(value)
}

pub(super) fn head(h: &Head) -> Vec<u8> {
    let mut b = b"CGH\x01".to_vec();
    for n in [h.position, h.signals, h.wire_end] {
        u64_out(&mut b, n);
    }
    b.push(u8::from(h.wire_blocked));
    b.extend(h.history_hash);
    b
}
pub(super) fn decode_head(bytes: &[u8]) -> Result<Head, Error> {
    let mut r = Reader::new(bytes);
    r.expect(b"CGH\x01")?;
    let h = Head {
        position: r.u64()?,
        signals: r.u64()?,
        wire_end: r.u64()?,
        wire_blocked: match r.byte()? {
            0 => false,
            1 => true,
            _ => return Err(corrupt("wire flag")),
        },
        history_hash: r.array()?,
    };
    r.end()?;
    Ok(h)
}

pub(super) struct History {
    pub fingerprint: Particle,
    pub receipt: Receipt,
    pub command: Vec<u8>,
    pub original_wire: Option<Vec<Vec<u8>>>,
}
pub(super) fn history(h: &History) -> Result<Vec<u8>, Error> {
    let mut b = b"CGJ\x01".to_vec();
    b.extend(h.fingerprint);
    bytes_out(&mut b, &receipt(&h.receipt))?;
    bytes_out(&mut b, &h.command)?;
    match &h.original_wire {
        None => b.push(0),
        Some(frames) => {
            b.push(1);
            b.extend((frames.len() as u32).to_le_bytes());
            for frame in frames {
                bytes_out(&mut b, frame)?;
            }
        }
    }
    Ok(b)
}
pub(super) fn decode_history(bytes: &[u8]) -> Result<History, Error> {
    let mut r = Reader::new(bytes);
    r.expect(b"CGJ\x01")?;
    let fingerprint = r.array()?;
    let receipt = decode_receipt(r.bytes(256)?)?;
    let command = r.bytes(MAX_OPERATION_BYTES)?.to_vec();
    let original_wire = match r.byte()? {
        0 => None,
        1 => {
            let n = r.u32()? as usize;
            if n > MAX_EVENTS {
                return Err(corrupt("wire event count"));
            }
            let mut frames = Vec::with_capacity(n);
            for _ in 0..n {
                frames.push(r.bytes(MAX_OPERATION_BYTES)?.to_vec());
            }
            Some(frames)
        }
        _ => return Err(corrupt("wire option")),
    };
    r.end()?;
    Ok(History {
        fingerprint,
        receipt,
        command,
        original_wire,
    })
}

pub(super) fn block(b: &Block) -> Result<Vec<u8>, Error> {
    let mut bytes = b"CGB\x01".to_vec();
    for n in [b.height, b.timestamp, b.supply, b.weight] {
        u64_out(&mut bytes, n);
    }
    bytes.extend(b.root);
    bytes_out(
        &mut bytes,
        &foculus::signal_codec::encode_signal(&b.signal)?,
    )?;
    Ok(bytes)
}
pub(super) fn decode_block(bytes: &[u8]) -> Result<Block, Error> {
    let mut r = Reader::new(bytes);
    r.expect(b"CGB\x01")?;
    let b = Block {
        height: r.u64()?,
        timestamp: r.u64()?,
        supply: r.u64()?,
        weight: r.u64()?,
        root: r.array()?,
        signal: foculus::signal_codec::decode_signal(r.bytes(MAX_OPERATION_BYTES)?)?,
    };
    r.end()?;
    Ok(b)
}

pub(super) fn u64_out(out: &mut Vec<u8>, v: u64) {
    out.extend(v.to_le_bytes());
}
pub(super) fn bytes_out(out: &mut Vec<u8>, v: &[u8]) -> Result<(), Error> {
    let n = u32::try_from(v.len()).map_err(|_| invalid("encoding length"))?;
    out.extend(n.to_le_bytes());
    out.extend(v);
    Ok(())
}
pub(super) struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8], Error> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| corrupt("encoding length"))?;
        let b = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| corrupt("truncated native record"))?;
        self.pos = end;
        Ok(b)
    }
    pub fn expect(&mut self, b: &[u8]) -> Result<(), Error> {
        if self.take(b.len())? != b {
            return Err(corrupt("record version"));
        }
        Ok(())
    }
    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        self.take(N)?.try_into().map_err(|_| corrupt("array"))
    }
    pub fn byte(&mut self) -> Result<u8, Error> {
        Ok(self.array::<1>()?[0])
    }
    pub fn u64(&mut self) -> Result<u64, Error> {
        Ok(u64::from_le_bytes(self.array()?))
    }
    pub fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.array()?))
    }
    pub fn bytes(&mut self, max: usize) -> Result<&'a [u8], Error> {
        let n = self.u32()? as usize;
        if n > max {
            return Err(corrupt("record limit"));
        }
        self.take(n)
    }
    pub fn end(&self) -> Result<(), Error> {
        if self.pos != self.bytes.len() {
            return Err(corrupt("trailing native record bytes"));
        }
        Ok(())
    }
}
