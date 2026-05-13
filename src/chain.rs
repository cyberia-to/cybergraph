// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! Signal chain: neuron-signed cyberlink batches with equivocation detection.
//!
//! Implements layer 2 ordering from structural-sync: hash chain, step counter,
//! and equivocation detection. Signal lifecycle owned by cybergraph, not bbg.

use std::collections::BTreeMap;

use hemera::hash as hemera_hash;

use bbg::{Cid, NeuronId};

/// A single cyberlink inside a signal.
pub struct CyberlinkRecord {
    pub neuron: NeuronId,
    pub from: Cid,
    pub to: Cid,
    pub token: Cid,
    pub amount: u64,
    pub valence: i8,
    pub height: u64,
}

/// A batch of cyberlinks signed by a neuron at a given step.
pub struct Signal {
    pub neuron: NeuronId,
    pub links: Vec<CyberlinkRecord>,
    pub delta_pi: Vec<(Cid, u64)>,
    /// Hash of the previous signal in this neuron's chain.
    pub prev: Cid,
    pub step: u64,
    /// Block height (0 until finalized).
    pub height: u64,
    pub proof: Option<zheng::Proof>,
}

impl Signal {
    /// Canonical hash of this signal: hemera over (neuron ‖ step ‖ prev).
    pub fn hash(&self) -> Cid {
        let mut buf = [0u8; 72]; // 32 + 8 + 32
        buf[..32].copy_from_slice(&self.neuron);
        buf[32..40].copy_from_slice(&self.step.to_le_bytes());
        buf[40..72].copy_from_slice(&self.prev);
        let h = hemera_hash(&buf);
        *h.as_bytes().first_chunk::<32>().unwrap_or(&[0u8; 32])
    }
}

/// Per-neuron ordered sequence of signals with equivocation detection.
pub struct SignalChain {
    pub entries: BTreeMap<u64, Signal>,
}

/// Errors produced by `SignalChain::append`.
#[derive(Debug)]
pub enum ChainError {
    StepNotSequential,
    PrevMismatch,
    Equivocation,
}

impl SignalChain {
    pub fn new() -> Self {
        Self { entries: BTreeMap::new() }
    }

    /// Append a signal. Returns `Err` on equivocation, wrong step, or prev mismatch.
    pub fn append(&mut self, signal: Signal) -> Result<(), ChainError> {
        if self.check_equivocation(&signal).is_some() {
            return Err(ChainError::Equivocation);
        }

        let expected_step = self.entries.len() as u64;
        if signal.step != expected_step {
            return Err(ChainError::StepNotSequential);
        }

        let expected_prev: Cid = if expected_step == 0 {
            [0u8; 32]
        } else {
            let last = self.entries.get(&(expected_step - 1)).ok_or(ChainError::PrevMismatch)?;
            last.hash()
        };

        if signal.prev != expected_prev {
            return Err(ChainError::PrevMismatch);
        }

        self.entries.insert(signal.step, signal);
        Ok(())
    }

    /// Return the existing signal if `s` would equivocate (same neuron, step, prev).
    pub fn check_equivocation(&self, s: &Signal) -> Option<&Signal> {
        self.entries.get(&s.step).and_then(|existing| {
            if existing.neuron == s.neuron && existing.prev == s.prev {
                Some(existing)
            } else {
                None
            }
        })
    }
}

impl Default for SignalChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn neuron(seed: u8) -> NeuronId {
        [seed; 32]
    }

    fn cid(seed: u8) -> Cid {
        [seed; 32]
    }

    fn signal(neuron: NeuronId, step: u64, prev: Cid) -> Signal {
        Signal {
            neuron,
            links: vec![],
            delta_pi: vec![],
            prev,
            step,
            height: 0,
            proof: None,
        }
    }

    #[test]
    fn first_signal_has_zero_prev() {
        let mut chain = SignalChain::new();
        let s = signal(neuron(1), 0, [0u8; 32]);
        chain.append(s).unwrap();
        assert_eq!(chain.entries.len(), 1);
    }

    #[test]
    fn sequential_chain_links_correctly() {
        let mut chain = SignalChain::new();
        let n = neuron(1);
        let s0 = signal(n, 0, [0u8; 32]);
        let prev1 = s0.hash();
        chain.append(s0).unwrap();

        let s1 = signal(n, 1, prev1);
        let prev2 = s1.hash();
        chain.append(s1).unwrap();

        let s2 = signal(n, 2, prev2);
        chain.append(s2).unwrap();

        assert_eq!(chain.entries.len(), 3);
    }

    #[test]
    fn wrong_step_is_rejected() {
        let mut chain = SignalChain::new();
        let s = signal(neuron(1), 5, [0u8; 32]);
        assert!(matches!(chain.append(s), Err(ChainError::StepNotSequential)));
    }

    #[test]
    fn wrong_prev_is_rejected() {
        let mut chain = SignalChain::new();
        chain.append(signal(neuron(1), 0, [0u8; 32])).unwrap();
        let s1 = signal(neuron(1), 1, cid(99));
        assert!(matches!(chain.append(s1), Err(ChainError::PrevMismatch)));
    }

    #[test]
    fn equivocation_is_detected() {
        let mut chain = SignalChain::new();
        let n = neuron(1);
        chain.append(signal(n, 0, [0u8; 32])).unwrap();
        let dupe = signal(n, 0, [0u8; 32]);
        assert!(matches!(chain.append(dupe), Err(ChainError::Equivocation)));
    }
}
