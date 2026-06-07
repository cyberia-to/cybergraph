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

use std::collections::BTreeMap;

use bbg::{Bbg, IntentRecord, NeuronId, Particle};
use cyber_sync::{ChainError, Signal, SignalChain};

/// Structured description of an intended action.
///
/// Encoding is dialect-specific; cybergraph stores its scope_hash (Hemera).
#[derive(Clone)]
pub struct Scope {
    pub target:      Particle,
    pub predicate:   Vec<u8>,
    pub deadline:    Option<u64>,
    pub constraints: Vec<u8>,
}

impl Scope {
    /// Canonical hash of the scope = H(target ‖ predicate ‖ deadline ‖ constraints).
    pub fn hash(&self) -> Particle {
        let mut buf: Vec<u8> = Vec::with_capacity(32 + self.predicate.len() + 8 + self.constraints.len());
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
    pub neuron:    NeuronId,
    pub h0:        u64,
    pub scope:     Scope,
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
    IntentDeclared { key: Particle, neuron: NeuronId, h0: u64 },
    SignalSealed   { intent_key: Particle, neuron: NeuronId, step: u64 },
    Linked         { neuron: NeuronId, step: u64 },
}

/// Errors from the public API.
#[derive(Debug)]
pub enum ApiError {
    /// Sync rejected the signal envelope (chain, VDF, or equivocation).
    SyncRejected(ChainError),
    /// Bbg rejected the cyberlinks (e.g., DoubleSpend).
    BbgRejected(bbg::InsertError),
    /// Intent key not found (seal called for unknown intent).
    UnknownIntent(Particle),
}

/// The cybergraph runtime: bbg state + per-neuron chains + event bus.
///
/// Scope is local-first — a cybergraph instance processes whichever cyberlinks
/// it sees, at whatever scope it is configured for. Sync fans out distribution
/// to peers when present.
pub struct Cybergraph {
    pub bbg:         Bbg,
    pub chains:      BTreeMap<NeuronId, SignalChain>,
    subscribers:     Vec<(Filter, Box<dyn Fn(&Event) + Send + Sync>)>,
}

impl Cybergraph {
    pub fn new() -> Self {
        Self {
            bbg:         Bbg::new(),
            chains:      BTreeMap::new(),
            subscribers: Vec::new(),
        }
    }

    /// intend — declare an unsealed intent. Persists the record and emits
    /// `IntentDeclared`. Sync layer would broadcast in a networked deployment.
    pub fn intend(&mut self, intent: Intent) -> Result<Particle, ApiError> {
        let record = IntentRecord {
            neuron:     intent.neuron,
            h0:         intent.h0,
            scope_hash: intent.scope.hash(),
            signature:  intent.signature,
        };
        let key = self.bbg.apply_intent(&record);
        self.emit(Event::IntentDeclared { key, neuron: intent.neuron, h0: intent.h0 });
        Ok(key)
    }

    /// seal — finalize an intent into a complete signal carrying a STARK.
    /// The signal's chain ordering is validated by sync; cyberlinks land in bbg.
    pub fn seal(&mut self, intent_key: Particle, signal: Signal) -> Result<(), ApiError> {
        if !self.bbg.state.intents.contains_key(&intent_key) {
            return Err(ApiError::UnknownIntent(intent_key));
        }
        let neuron = signal.neuron;
        let step = signal.step;
        self.chain_append(signal.clone())?;
        // The signal carries CyberlinkRecord but bbg.insert expects bbg::Signal —
        // adapt: write a header-only record and skip cyberlink application for
        // now (full bridge lives in Phase 5+).
        self.bbg.apply_signal_record(step, bbg::SignalRecord {
            neuron,
            link_count: signal.links.len() as u32,
            block_height: signal.height,
            proof_hash: [0u8; 32],
        });
        self.emit(Event::SignalSealed { intent_key, neuron, step });
        Ok(())
    }

    /// link — atomic, one-shot submit. No prior intent required. Used for
    /// discrete local statements where the process does not need phasing.
    pub fn link(&mut self, signal: Signal) -> Result<(), ApiError> {
        let neuron = signal.neuron;
        let step = signal.step;
        self.chain_append(signal.clone())?;
        self.bbg.apply_signal_record(step, bbg::SignalRecord {
            neuron,
            link_count: signal.links.len() as u32,
            block_height: signal.height,
            proof_hash: [0u8; 32],
        });
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

    /// query — run an inf (CozoScript) query. The actual execution is
    /// delegated to inf; the relation schema lives in specs/query.md.
    /// This stub returns the raw script for the caller to feed into inf.
    pub fn query(&self, inf_script: &str) -> String {
        // Real implementation routes to a CozoDB instance fed by bbg state.
        inf_script.to_string()
    }

    // ── internals ────────────────────────────────────────────────────────────

    fn chain_append(&mut self, signal: Signal) -> Result<(), ApiError> {
        let chain = self.chains.entry(signal.neuron).or_default();
        chain.append(signal).map_err(ApiError::SyncRejected)
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

impl Default for Cybergraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    fn n(seed: u8) -> NeuronId { [seed; 32] }
    fn p(seed: u8) -> Particle { [seed; 32] }

    fn scope(target: Particle) -> Scope {
        Scope { target, predicate: b"do-thing".to_vec(), deadline: Some(100), constraints: vec![] }
    }

    fn intent(neuron: NeuronId, h0: u64, target: Particle) -> Intent {
        Intent { neuron, h0, scope: scope(target), signature: [0u8; 64] }
    }

    fn empty_signal(neuron: NeuronId, step: u64, prev: Particle) -> Signal {
        Signal { neuron, links: vec![], delta_pi: vec![], prev, step, height: 0, proof: None }
    }

    #[test]
    fn intend_persists_and_returns_key() {
        let mut g = Cybergraph::new();
        let key = g.intend(intent(n(1), 0, p(2))).unwrap();
        assert!(g.bbg.state.intents.contains_key(&key));
    }

    #[test]
    fn seal_unknown_intent_fails() {
        let mut g = Cybergraph::new();
        let s = empty_signal(n(1), 0, [0u8; 32]);
        let err = g.seal(p(99), s);
        assert!(matches!(err, Err(ApiError::UnknownIntent(_))));
    }

    #[test]
    fn intend_then_seal_succeeds() {
        let mut g = Cybergraph::new();
        let key = g.intend(intent(n(1), 0, p(2))).unwrap();
        let s = empty_signal(n(1), 0, [0u8; 32]);
        g.seal(key, s).unwrap();
        assert!(g.bbg.state.signals.contains_key(&0));
    }

    #[test]
    fn link_atomic_path() {
        let mut g = Cybergraph::new();
        let s = empty_signal(n(1), 0, [0u8; 32]);
        g.link(s).unwrap();
        assert!(g.bbg.state.signals.contains_key(&0));
    }

    #[test]
    fn subscribe_receives_matching_events() {
        let mut g = Cybergraph::new();
        let log: Arc<Mutex<Vec<&'static str>>> = Arc::new(Mutex::new(Vec::new()));
        let log2 = log.clone();
        g.subscribe(Filter::All, move |e| {
            let tag = match e {
                Event::IntentDeclared { .. } => "intent",
                Event::SignalSealed { .. }   => "seal",
                Event::Linked { .. }         => "link",
            };
            log2.lock().unwrap().push(tag);
        });
        g.intend(intent(n(1), 0, p(2))).unwrap();
        let s = empty_signal(n(1), 0, [0u8; 32]);
        g.link(s).unwrap();
        assert_eq!(*log.lock().unwrap(), vec!["intent", "link"]);
    }

    #[test]
    fn subscribe_by_neuron_filters() {
        let mut g = Cybergraph::new();
        let counter: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        let c2 = counter.clone();
        g.subscribe(Filter::ByNeuron(n(1)), move |_| { *c2.lock().unwrap() += 1; });
        g.link(empty_signal(n(1), 0, [0u8; 32])).unwrap();
        g.link(empty_signal(n(2), 0, [0u8; 32])).unwrap();
        assert_eq!(*counter.lock().unwrap(), 1);
    }

    #[test]
    fn query_returns_script_stub() {
        let g = Cybergraph::new();
        let q = g.query("?[x] := *cyberlinks{}");
        assert_eq!(q, "?[x] := *cyberlinks{}");
    }
}
