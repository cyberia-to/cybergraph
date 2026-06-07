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
        let (neuron, step) = self.commit_signal(signal)?;
        self.emit(Event::SignalSealed { intent_key, neuron, step });
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

    /// query — run an inf (CozoScript) query. The actual execution is
    /// delegated to inf; the relation schema lives in specs/query.md.
    /// This stub returns the raw script for the caller to feed into inf.
    pub fn query(&self, inf_script: &str) -> String {
        // Real implementation routes to a CozoDB instance fed by bbg state.
        inf_script.to_string()
    }

    // ── internals ────────────────────────────────────────────────────────────

    /// Order, apply, and record a signal. Shared by `link` and `seal`.
    ///
    /// Three steps, in order:
    ///   1. chain ordering — equivocation / step / prev (gate: sync)
    ///   2. state application — cyberlinks land in bbg via `insert` (gate: double-spend)
    ///   3. header record — the signal header enters the signals dimension
    ///
    /// Release 0: `box_moves` is always empty (CyberlinkRecord carries none), so
    /// `insert` is infallible here and the partial-failure window between step 1
    /// and step 2 cannot open. When conviction spends arrive (Release 1), this
    /// must become a validate-then-apply two-phase commit.
    fn commit_signal(&mut self, signal: Signal) -> Result<(NeuronId, u64), ApiError> {
        let neuron = signal.neuron;
        let step = signal.step;
        let link_count = signal.links.len() as u32;
        let height = signal.height;

        let bbg_signal = bridge_to_bbg(&signal);

        // 1. ordering gate
        self.chain_append(signal)?;
        // 2. apply cyberlinks to authenticated state
        self.bbg.insert(&bbg_signal).map_err(ApiError::BbgRejected)?;
        // 3. record the signal header
        self.bbg.apply_signal_record(step, bbg::SignalRecord {
            neuron,
            link_count,
            block_height: height,
            proof_hash: [0u8; 32],
        });

        Ok((neuron, step))
    }

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

/// Convert a sync signal into a bbg signal for state application.
///
/// Maps each `CyberlinkRecord` to a `bbg::Cyberlink` (from, to, token, amount,
/// valence). Release 0 limitation: `CyberlinkRecord` carries no box movements,
/// so `box_moves` is empty — the local path applies cyberlinks only; conviction
/// spends arrive in Release 1.
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
        box_moves: Vec::new(),
        height: signal.height,
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
    use cyber_sync::CyberlinkRecord;
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

    fn one_link_signal(neuron: NeuronId, step: u64, prev: Particle, from: Particle, to: Particle) -> Signal {
        Signal {
            neuron,
            links: vec![CyberlinkRecord { neuron, from, to, token: p(0), amount: 1, valence: 1, height: 0 }],
            delta_pi: vec![],
            prev,
            step,
            height: 0,
            proof: None,
        }
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
    fn link_applies_cyberlinks_to_bbg_state() {
        let mut g = Cybergraph::new();
        let s = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
        g.link(s).unwrap();
        // The cyberlink (p2 → p3, amount 1) must reach bbg state, not just the header.
        let target = g.bbg.state.particles.get(&p(3)).expect("target particle materialized");
        assert_eq!(target.energy, 1, "target energy reflects the staked link");
        assert!(g.bbg.state.axons_out.get(&p(2)).is_some(), "outgoing axon recorded");
        assert!(g.bbg.state.axons_in.get(&p(3)).is_some(), "incoming axon recorded");
    }

    #[test]
    fn link_moves_the_root() {
        let mut g = Cybergraph::new();
        let before = g.bbg.state.root;
        g.link(one_link_signal(n(1), 0, [0u8; 32], p(2), p(3))).unwrap();
        assert_ne!(g.bbg.state.root, before, "applying cyberlinks advances BBG_root");
    }

    #[test]
    fn seal_applies_cyberlinks_to_bbg_state() {
        let mut g = Cybergraph::new();
        let key = g.intend(intent(n(1), 0, p(3))).unwrap();
        let s = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
        g.seal(key, s).unwrap();
        assert_eq!(g.bbg.state.particles.get(&p(3)).unwrap().energy, 1);
    }

    #[test]
    fn link_accumulates_across_signals() {
        let mut g = Cybergraph::new();
        let s0 = one_link_signal(n(1), 0, [0u8; 32], p(2), p(3));
        let prev = s0.hash(); // chain link for the next signal
        g.link(s0).unwrap();
        let s1 = one_link_signal(n(1), 1, prev, p(2), p(3));
        g.link(s1).unwrap();
        // Two staked links onto p3 accumulate energy.
        assert_eq!(g.bbg.state.particles.get(&p(3)).unwrap().energy, 2);
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
