// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! Cybergraph public API — five verbs over a local cyberlink processor.
//!
//! The lifecycle verbs are discrete and ordered:
//!
//!   intend(scope)        → declare an intent: signed scope, no STARK yet.
//!                          Persisted in bbg's intents dimension so abandonment
//!                          stays on the record.
//!   seal(key, signal)    → finalize a previously-declared intent into a signal
//!                          with a STARK proof. Validates the chain link from
//!                          intent → signal via sync's order_and_chain.
//!   link(signal)         → atomic one-shot submit. Used when the process is
//!                          a discrete local statement that does not need a
//!                          separate intent phase.
//!
//! The interaction verbs are read/observe:
//!
//!   subscribe(filter)    → register an event handler over a filter predicate.
//!   query(inf_script)    → run an inf (CozoScript datalog) query over the
//!                          relations cybergraph exposes — schema in specs/query.md.

mod batch;
#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, btree_map::Entry};

use bbg::{Bbg, IntentRecord, NeuronId, Particle};
use foculus::{ChainError, Signal, SignalChain};
use inf_eval::{Ctx, Output, eval};
use inf_parse::parse;
use inf_plan::plan;

use crate::source::BbgSource;

/// Structured description of an intended action.
///
/// Encoding is dialect-specific; cybergraph stores its scope_hash (Hemera).
#[derive(Clone)]
pub struct Scope {
    pub target: Particle,
    pub predicate: Vec<u8>,
    pub deadline: Option<u64>,
    pub constraints: Vec<u8>,
}

impl Scope {
    /// Canonical hash of the scope = H(target ‖ predicate ‖ deadline ‖ constraints).
    pub fn hash(&self) -> Particle {
        let mut buf: Vec<u8> =
            Vec::with_capacity(32 + self.predicate.len() + 8 + self.constraints.len());
        buf.extend_from_slice(&self.target);
        buf.extend_from_slice(&self.predicate);
        buf.extend_from_slice(&self.deadline.unwrap_or(0).to_le_bytes());
        buf.extend_from_slice(&self.constraints);
        let h = hemera::hash(&buf);
        let b = h.as_bytes();
        let mut out = [0u8; 32];
        out[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
        out
    }
}

/// A declared intent: neuron + scope + identity proof.
/// At declare time there is no STARK — only the signature over (ν ‖ h0 ‖ scope_hash).
pub struct Intent {
    pub neuron: NeuronId,
    pub h0: u64,
    pub scope: Scope,
    pub signature: [u8; 64],
}

/// Subscription filter — which events the caller wants to observe.
#[derive(Clone)]
pub enum Filter {
    All,
    ByNeuron(NeuronId),
    ByTargetParticle(Particle),
}

/// Events emitted by cybergraph.
#[derive(Clone)]
pub enum Event {
    IntentDeclared {
        key: Particle,
        neuron: NeuronId,
        h0: u64,
    },
    SignalSealed {
        intent_key: Particle,
        neuron: NeuronId,
        step: u64,
    },
    Linked {
        neuron: NeuronId,
        step: u64,
    },
}

/// Errors from the public API.
#[derive(Debug)]
pub enum ApiError {
    /// Sync rejected the signal envelope (chain, VDF, or equivocation).
    SyncRejected(ChainError),
    /// Bbg rejected the cyberlinks (e.g., DoubleSpend).
    BbgRejected(bbg::InsertError),
    /// Applying the ordered link batch would overflow a public u64 balance.
    AmountOverflow,
    /// Intent key not found (seal called for unknown intent).
    UnknownIntent(Particle),
    /// Signal's resolved destination network does not match the network this
    /// node serves.
    WrongNetwork { expected: Particle, got: Particle },
}

/// A neuron's private network — the default destination for its signals.
///
/// Derived deterministically as a card id from the neuron identity, so every
/// neuron has a sovereign private network with no minting step.
pub fn private_network(neuron: &NeuronId) -> Particle {
    let mut buf = Vec::with_capacity(8 + 32);
    buf.extend_from_slice(b"network:");
    buf.extend_from_slice(neuron);
    let h = hemera::hash(&buf);
    let b = h.as_bytes();
    let mut out = [0u8; 32];
    out[..b.len().min(32)].copy_from_slice(&b[..b.len().min(32)]);
    out
}

/// Errors from `query` — one variant per inf pipeline stage.
#[derive(Debug)]
pub enum QueryError {
    Parse(String),
    Plan(String),
    Eval(String),
}

/// The cybergraph runtime: bbg state + per-neuron chains + event bus.
///
/// Scope is local-first — a cybergraph instance processes whichever cyberlinks
/// it sees, at whatever scope it is configured for. Sync fans out distribution
/// to peers when present.
pub struct Cybergraph {
    pub bbg: Bbg,
    pub chains: BTreeMap<NeuronId, SignalChain>,
    /// The network this node serves. `None` accepts any network (local-first
    /// dev default); `Some(n)` enforces that each signal's resolved network is `n`.
    pub network: Option<Particle>,
    subscribers: Vec<(Filter, Box<dyn Fn(&Event) + Send + Sync>)>,
}

impl Cybergraph {
    pub fn new() -> Self {
        Self {
            bbg: Bbg::new(),
            chains: BTreeMap::new(),
            network: None,
            subscribers: Vec::new(),
        }
    }

    /// A node serving exactly one network — rejects any signal whose resolved
    /// destination network differs from `network`.
    pub fn serving(network: Particle) -> Self {
        Self {
            network: Some(network),
            ..Self::new()
        }
    }

    /// intend — declare an unsealed intent. Persists the record and emits
    /// `IntentDeclared`. Sync layer would broadcast in a networked deployment.
    pub fn intend(&mut self, intent: Intent) -> Result<Particle, ApiError> {
        let record = IntentRecord {
            neuron: intent.neuron,
            h0: intent.h0,
            scope_hash: intent.scope.hash(),
            signature: intent.signature,
        };
        let key = self.bbg.apply_intent(&record);
        self.emit(Event::IntentDeclared {
            key,
            neuron: intent.neuron,
            h0: intent.h0,
        });
        Ok(key)
    }

    /// seal — finalize an intent into a complete signal carrying a STARK.
    /// The signal's chain ordering is validated by sync; cyberlinks land in bbg.
    pub fn seal(&mut self, intent_key: Particle, signal: Signal) -> Result<(), ApiError> {
        if !self.bbg.state.intents.contains_key(&intent_key) {
            return Err(ApiError::UnknownIntent(intent_key));
        }
        let (neuron, step) = self.commit_signal(signal)?;
        self.emit(Event::SignalSealed {
            intent_key,
            neuron,
            step,
        });
        Ok(())
    }

    /// link — atomic, one-shot submit. No prior intent required. Used for
    /// discrete local statements where the process does not need phasing.
    pub fn link(&mut self, signal: Signal) -> Result<(), ApiError> {
        let (neuron, step) = self.commit_signal(signal)?;
        self.emit(Event::Linked { neuron, step });
        Ok(())
    }

    /// subscribe — register a handler over an event filter.
    pub fn subscribe<F>(&mut self, filter: Filter, handler: F)
    where
        F: Fn(&Event) + Send + Sync + 'static,
    {
        self.subscribers.push((filter, Box::new(handler)));
    }

    /// query — run an inf query over local bbg state.
    ///
    /// Pipeline: parse → plan → eval against a `BbgSource` (the bbg aggregate
    /// relations: particles, neurons, axons, focus, karma, signals). Returns the
    /// inf `Output` (columns + rows, or a derived mutation batch).
    ///
    /// Release 0: reads run over the local snapshot and are not yet provable
    /// (`BbgSource::provable` is false) — Lens openings over the committed root
    /// arrive in a later release. The query language itself lives in [[inf]].
    pub fn query(&self, inf_script: &str) -> Result<Output, QueryError> {
        let prog = parse(inf_script)
            .map_err(|e| QueryError::Parse(format!("[{}:{}] {}", e.line, e.col, e.msg)))?;
        let ir = plan(&prog).map_err(|e| QueryError::Plan(e.msg))?;
        let src = BbgSource::new(&self.bbg.state);
        eval(&ir, &src, &Ctx::default()).map_err(|e| QueryError::Eval(e.msg))
    }

    // ── internals ────────────────────────────────────────────────────────────

    /// Order, apply, and record a signal. Shared by `link` and `seal`.
    ///
    /// Three steps, in order:
    ///   1. chain ordering — equivocation / step / prev (gate: sync)
    ///   2. state application — cyberlinks land in bbg via `insert` (gate: double-spend)
    ///   3. header record — the signal header enters the signals dimension
    ///
    /// Preflight same-batch nullifier uniqueness and arithmetic before mutation.
    /// BBG's only returned insertion error is structural DoubleSpend, checked
    /// before it changes state. If that owner rejects, remove the just-appended
    /// chain entry (and a newly created chain) before returning. Exclusive &mut
    /// access keeps the temporary append unobservable; events follow success.
    /// This in-memory boundary carries no disk durability or consensus claim.
    fn commit_signal(&mut self, signal: Signal) -> Result<(NeuronId, u64), ApiError> {
        let neuron = signal.neuron;
        let step = signal.step;
        let link_count = signal.links.len() as u32;
        let height = signal.height;

        // Resolve the destination network: the SELF_NETWORK sentinel routes the
        // signal to the sender's private network.
        let network = if signal.network == foculus::SELF_NETWORK {
            private_network(&neuron)
        } else {
            signal.network
        };
        // Routing gate: a node serving a specific network rejects foreign signals.
        if let Some(serving) = self.network {
            if network != serving {
                return Err(ApiError::WrongNetwork {
                    expected: serving,
                    got: network,
                });
            }
        }

        let bbg_signal = bridge_to_bbg(&signal);
        self.validate_bbg_batch(&bbg_signal)?;

        // 1. ordering gate
        let new_chain = !self.chains.contains_key(&neuron);
        self.chain_append(signal)?;
        // 2. apply cyberlinks to authenticated state
        if let Err(error) = self.bbg.insert(&bbg_signal) {
            if new_chain {
                self.chains.remove(&neuron);
            } else {
                self.chains
                    .get_mut(&neuron)
                    .expect("appended chain exists")
                    .entries
                    .remove(&step);
            }
            return Err(ApiError::BbgRejected(error));
        }
        // 3. record the signal header at bbg::signal_key(neuron, step)
        self.bbg.apply_signal_record(bbg::SignalRecord {
            neuron,
            network,
            link_count,
            block_height: height,
            proof_hash: [0u8; 32],
            step,
        });

        Ok((neuron, step))
    }

    fn chain_append(&mut self, signal: Signal) -> Result<(), ApiError> {
        match self.chains.entry(signal.neuron) {
            Entry::Occupied(mut entry) => entry
                .get_mut()
                .append(signal)
                .map_err(ApiError::SyncRejected),
            Entry::Vacant(entry) => {
                let mut chain = SignalChain::new();
                chain.append(signal).map_err(ApiError::SyncRejected)?;
                entry.insert(chain);
                Ok(())
            }
        }
    }

    fn emit(&self, event: Event) {
        for (filter, handler) in &self.subscribers {
            if Self::matches(filter, &event) {
                handler(&event);
            }
        }
    }

    fn matches(filter: &Filter, event: &Event) -> bool {
        match (filter, event) {
            (Filter::All, _) => true,
            (Filter::ByNeuron(n), Event::IntentDeclared { neuron, .. })
            | (Filter::ByNeuron(n), Event::SignalSealed { neuron, .. })
            | (Filter::ByNeuron(n), Event::Linked { neuron, .. }) => n == neuron,
            (Filter::ByTargetParticle(_), _) => false, // target lookup needs scope; see Phase 5
        }
    }
}

/// Convert a sync signal into a bbg signal for state application.
///
/// Maps each `CyberlinkRecord` to a `bbg::Cyberlink` and each
/// `BoxMoveRecord` to a `bbg::BoxMove` (nullifier double-spend gate).
fn bridge_to_bbg(signal: &Signal) -> bbg::Signal {
    bbg::Signal {
        neuron: signal.neuron,
        links: signal
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
        box_moves: signal
            .box_moves
            .iter()
            .map(|m| bbg::BoxMove {
                nullifier: m.nullifier,
                commitment: m.commitment,
            })
            .collect(),
        height: signal.height,
    }
}

impl Default for Cybergraph {
    fn default() -> Self {
        Self::new()
    }
}
