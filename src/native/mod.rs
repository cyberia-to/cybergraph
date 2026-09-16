//! Durable local native execution, backed by one shared BBG database.
mod codec;
mod commit;
mod economics;
mod import;
mod read;
mod recovery;
mod view;
pub use view::HistoryView;
#[cfg(test)]
mod tests;

use crate::{Cybergraph, CyberlinkRecord, IntentRecord, NeuronId, Particle, SELF_NETWORK, Signal};
use bbg::storage::{
    StorageError,
    database::{Backend, Database, RecordDomain as D},
};
pub use import::ImportReport;
use std::{collections::BTreeMap, fmt, path::Path};

pub const MAX_EVENTS: usize = 64;
pub const MAX_OPERATION_BYTES: usize = 4 * 1024 * 1024;
pub const PROFILE: &[u8] = b"cybergraph/local-chaosnet/v1";

/// Canonical command bytes used by request fingerprints and history export.
pub fn encode_operation(operation: &Operation) -> Result<Vec<u8>, Error> {
    codec::operation(operation)
}
/// Decode one bounded command, rejecting unknown versions and trailing bytes.
pub fn decode_operation(bytes: &[u8]) -> Result<Operation, Error> {
    codec::decode_operation(bytes)
}

// Batches contain at most 64 events. Keeping signals inline avoids a separate
// allocation per signal inside that already bounded vector.
#[allow(clippy::large_enum_variant)]
pub enum Event {
    Signal(Signal),
    Intent(IntentRecord),
    /// Trusted local host adjustment; never decoded from peer signal tape.
    LocalCredit { neuron: NeuronId, token: Particle, amount: u64, focus: u64, reason: Particle },
}
pub enum Operation {
    Link {
        neuron: NeuronId,
        from: Particle,
        to: Particle,
        token: Particle,
        amount: u64,
        valence: i8,
    },
    Pay {
        from: NeuronId,
        to: NeuronId,
        amount: u64,
    },
    Events(Vec<Event>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Receipt {
    pub request_id: Particle,
    pub position: u64,
    pub height: u64,
    pub root: Particle,
    pub signals: u64,
    pub balance: Option<u64>,
    pub supply: u64,
    pub weight: u64,
    pub timestamp: u64,
    pub applied: u64,
}

pub struct Block {
    pub height: u64,
    pub timestamp: u64,
    pub supply: u64,
    pub weight: u64,
    pub root: Particle,
    pub signal: Signal,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockSummary {
    pub height: u64,
    pub timestamp: u64,
    pub supply: u64,
    pub weight: u64,
    pub root: Particle,
}

#[derive(Debug)]
pub enum Error {
    Conflict,
    Invalid(String),
    Limit(String),
    Unsupported(String),
    Storage(StorageError),
    Corrupt(String),
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Conflict => f.write_str("request identity already used for different content"),
            Self::Invalid(s) => write!(f, "native operation rejected: {s}"),
            Self::Limit(s) => write!(f, "native operation limit: {s}"),
            Self::Unsupported(s) => write!(f, "unsupported native profile: {s}"),
            Self::Storage(e) => write!(f, "native storage: {e}"),
            Self::Corrupt(s) => write!(f, "native store inconsistent: {s}"),
        }
    }
}
impl std::error::Error for Error {}
impl From<StorageError> for Error {
    fn from(e: StorageError) -> Self {
        Self::Storage(e)
    }
}
impl From<foculus::signal_codec::CodecError> for Error {
    fn from(e: foculus::signal_codec::CodecError) -> Self {
        Self::Invalid(e.to_string())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct Head {
    position: u64,
    signals: u64,
    wire_end: u64,
    wire_blocked: bool,
    history_hash: Particle,
}

pub struct NativeNode {
    db: Database,
    graph: Cybergraph,
    balances: BTreeMap<NeuronId, u64>,
    supply: u64,
    head: Head,
    instance: Particle,
}

impl NativeNode {
    pub fn from_database(db: Database, genesis: &[u8]) -> Result<Self, Error> {
        Self::open_database(db, genesis, false)
    }

    pub fn database(&self) -> Database { self.db.clone() }

    pub fn open(path: &Path, genesis: &[u8]) -> Result<Self, Error> {
        let db = Database::open(path, Backend::Ssd)?;
        Self::open_database(db, genesis, false)
    }

    pub fn graph(&self) -> &Cybergraph {
        &self.graph
    }
    pub fn height(&self) -> u64 {
        self.graph.bbg.state.height
    }
    pub fn root(&self) -> Particle {
        self.graph.bbg.state.root()
    }
    pub fn balance(&self, neuron: &NeuronId) -> u64 {
        self.balances.get(neuron).copied().unwrap_or(0)
    }
    pub fn supply(&self) -> u64 {
        self.supply
    }
    pub fn healthy(&self) -> bool {
        !self.db.is_poisoned()
    }

    pub fn accept(
        &mut self,
        request_id: Option<Particle>,
        operation: Operation,
        timestamp: u64,
    ) -> Result<Receipt, Error> {
        let command = codec::operation(&operation)?;
        self.accept_encoded(request_id, operation, command, timestamp, None, None)
    }

    fn events(&self, operation: Operation) -> Result<(Vec<Event>, Option<NeuronId>), Error> {
        match operation {
            Operation::Events(events) => {
                if events.is_empty() || events.len() > MAX_EVENTS {
                    return Err(Error::Invalid("event count must be 1..64".into()));
                }
                Ok((events, None))
            }
            Operation::Link {
                neuron,
                from,
                to,
                token,
                amount,
                valence,
            } => {
                let mut signal = self.signal(neuron);
                signal.links.push(CyberlinkRecord {
                    neuron,
                    from,
                    to,
                    token,
                    amount,
                    valence,
                    height: 0,
                });
                Ok((vec![Event::Signal(signal)], None))
            }
            Operation::Pay { from, to, amount } => {
                if amount == 0 || self.balance(&from) < amount {
                    return Err(Error::Invalid("zero payment or insufficient funds".into()));
                }
                let mut signal = self.signal(from);
                signal.delta_pi.push((to, amount));
                Ok((vec![Event::Signal(signal)], Some(from)))
            }
        }
    }

    fn signal(&self, neuron: NeuronId) -> Signal {
        let (step, prev) = self.next_position(&neuron);
        Signal {
            neuron,
            network: SELF_NETWORK,
            links: vec![],
            delta_pi: vec![],
            box_moves: vec![],
            step,
            prev,
            height: 0,
            proof: None,
        }
    }

    fn next_position(&self, neuron: &NeuronId) -> (u64, Particle) {
        self.graph
            .chains
            .get(neuron)
            .and_then(|c| c.entries.last_key_value())
            .map_or((0, [0; 32]), |(step, s)| (step.saturating_add(1), s.hash()))
    }
}

fn digest(domain: &[u8], parts: &[&[u8]]) -> Particle {
    let mut hash = hemera::Hasher::new();
    hash.update(domain);
    for part in parts {
        hash.update(&(part.len() as u64).to_le_bytes());
        hash.update(part);
    }
    *hash.finalize().as_bytes()
}
fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}
fn corrupt(s: impl Into<String>) -> Error {
    Error::Corrupt(s.into())
}
