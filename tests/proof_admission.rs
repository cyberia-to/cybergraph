// ---
// tags: cybergraph, rust, test, proof
// crystal-type: source
// crystal-domain: cyber
// ---
//! Admission verifies a signal's proof (foculus pay proof, zheng public
//! certificate v3) before anything is applied — in the in-memory API and in
//! the durable native node. Signals without a proof behave as before.

use cybergraph::{
    ApiError, Cybergraph, CyberlinkRecord, NeuronId, PayProofError, Proof, ProofError, Signal,
};

fn neuron() -> NeuronId {
    [1; 32]
}

fn pay_signal(network: [u8; 32], amounts: &[u64]) -> Signal {
    Signal {
        neuron: neuron(),
        network,
        links: amounts
            .iter()
            .enumerate()
            .map(|(i, &amount)| CyberlinkRecord {
                neuron: neuron(),
                from: [71; 32],
                to: [72 + i as u8; 32],
                token: [0; 32],
                amount,
                valence: 1,
                height: 0,
            })
            .collect(),
        delta_pi: vec![],
        box_moves: vec![],
        prev: [0; 32],
        step: 0,
        height: 0,
        proof: None,
    }
}

fn proved(network: [u8; 32], amounts: &[u64]) -> Signal {
    let mut s = pay_signal(network, amounts);
    s.proof = Some(foculus::prove_pay(&s).unwrap());
    s
}

/// The valid proof with one certificate value changed.
fn forged(network: [u8; 32], amounts: &[u64]) -> Signal {
    let mut s = proved(network, amounts);
    let Some(Proof::Public(body)) = s.proof.as_mut() else {
        unreachable!("prove_pay returns a proof")
    };
    body.certificate.free[0] = (body.certificate.free[0] + 1) % nebu::field::P;
    s
}

/// A valid proof of another signal (other amounts, same leg count).
fn transplanted(network: [u8; 32]) -> Signal {
    let mut s = pay_signal(network, &[100, 50]);
    s.proof = proved(network, &[100, 51]).proof;
    s
}

#[test]
fn api_admits_a_valid_proved_signal() {
    let mut g = Cybergraph::new();
    g.link(proved(foculus::SELF_NETWORK, &[100, 50])).unwrap();
    assert_eq!(g.chains[&neuron()].entries.len(), 1);
    assert_eq!(g.bbg.state.signals.len(), 1);
}

#[test]
fn api_rejects_forged_and_transplanted_proofs_without_applying() {
    let mut g = Cybergraph::new();
    let root = g.bbg.state.root();
    let e = g
        .link(forged(foculus::SELF_NETWORK, &[100, 50]))
        .unwrap_err();
    assert!(
        matches!(e, ApiError::ProofRejected(PayProofError::Proof(_))),
        "{e:?}"
    );
    let e = g.link(transplanted(foculus::SELF_NETWORK)).unwrap_err();
    assert!(
        matches!(
            e,
            ApiError::ProofRejected(PayProofError::Proof(ProofError::WrongStatement))
        ),
        "{e:?}"
    );
    assert!(g.chains.is_empty());
    assert_eq!(g.bbg.state.root(), root);
    // the neuron's chain is untouched: the honest signal at step 0 still fits
    g.link(pay_signal(foculus::SELF_NETWORK, &[1])).unwrap();
}

#[cfg(feature = "local-storage")]
mod native {
    use super::*;
    use cybergraph::native::{Error, Event, NativeNode, Operation, decode_operation};
    use foculus::signal_codec::{LEGACY_VERSION, MAGIC, encode_signal};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);
    struct Temp(PathBuf);
    impl Temp {
        fn new() -> Self {
            let p = std::env::temp_dir().join(format!(
                "cybergraph-proof-admission-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir(&p).unwrap();
            Self(p)
        }
        fn open(&self) -> NativeNode {
            NativeNode::open(&self.0.join("bbg"), b"proof admission genesis").unwrap()
        }
    }
    impl Drop for Temp {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn events(s: Signal) -> Operation {
        Operation::Events(vec![Event::Signal(s)])
    }

    #[test]
    fn native_admits_a_valid_proved_signal_and_keeps_its_proof() {
        let t = Temp::new();
        let s = proved([88; 32], &[100, 50]);
        let proof_hash = foculus::signal_codec::proof_hash(&s).unwrap();
        let mut node = t.open();
        let receipt = node.accept(Some([1; 32]), events(s), 1).unwrap();
        assert_eq!(
            (receipt.height, receipt.signals, receipt.applied),
            (1, 1, 1)
        );
        assert_eq!(node.graph().bbg.state.signals[&0].proof_hash, proof_hash);
        drop(node);
        let node = t.open();
        let block = node.block(1).unwrap().unwrap();
        assert!(foculus::verify_pay(
            block.signal.proof.as_ref().unwrap(),
            &block.signal
        ));
    }

    #[test]
    fn native_rejects_forged_and_transplanted_proofs_without_applying() {
        let t = Temp::new();
        let mut node = t.open();
        let (root, height) = (node.root(), node.height());
        for (label, s) in [
            ("forged", forged([88; 32], &[100, 50])),
            ("transplanted", transplanted([88; 32])),
            ("self network forged", forged(foculus::SELF_NETWORK, &[3])),
        ] {
            match node.accept(None, events(s), 1) {
                Err(Error::Invalid(m)) => assert!(
                    m.starts_with("signal proof: pay proof rejected"),
                    "{label}: {m}"
                ),
                other => panic!("{label}: {other:?}"),
            }
            assert_eq!((node.root(), node.height()), (root, height), "{label}");
        }
        // a proof-less signal is admitted as before
        node.accept(None, events(pay_signal(foculus::SELF_NETWORK, &[1])), 2)
            .unwrap();
        assert_eq!(node.height(), 1);
    }

    #[test]
    fn native_rejects_a_proof_over_a_signal_without_links() {
        let t = Temp::new();
        let mut node = t.open();
        let mut s = pay_signal([88; 32], &[]);
        s.proof = proved([88; 32], &[1]).proof;
        let e = node.accept(None, events(s), 1).unwrap_err();
        assert!(
            e.to_string()
                .contains("pay proof over a signal without links"),
            "{e}"
        );
    }

    /// A version-1 signal carrying a legacy (HyperNova) proof body cannot
    /// enter: the operation decoder refuses it before admission.
    #[test]
    fn legacy_proof_signal_is_rejected_at_decode() {
        let s = pay_signal([88; 32], &[100]);
        let mut v1 = encode_signal(&s).unwrap();
        v1[MAGIC.len()] = LEGACY_VERSION;
        let flag = v1.len() - 1;
        v1[flag] = 1;
        v1.extend_from_slice(&[0u8; 64]);
        let mut op = b"CGOP\x01\x02".to_vec();
        op.extend(1u32.to_le_bytes());
        op.push(0);
        op.extend((v1.len() as u32).to_le_bytes());
        op.extend(&v1);
        match decode_operation(&op) {
            Err(Error::Invalid(m)) => assert!(m.contains("LegacyProof"), "{m}"),
            other => panic!("legacy proof decoded: {}", other.is_ok()),
        }
        // the same bytes without the legacy body decode (v1, no proof)
        let mut v1 = encode_signal(&s).unwrap();
        v1[MAGIC.len()] = LEGACY_VERSION;
        let mut op = b"CGOP\x01\x02".to_vec();
        op.extend(1u32.to_le_bytes());
        op.push(0);
        op.extend((v1.len() as u32).to_le_bytes());
        op.extend(&v1);
        assert!(decode_operation(&op).is_ok());
    }
}
