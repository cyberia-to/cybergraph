// ---
// tags: cybergraph, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! Verifiable Delay Function over the Goldilocks field.
//!
//! Sequential squaring: output = x^(2^T) mod p.
//! Layer 2 ordering primitive — owned by cybergraph per structural-sync spec.
//! Proves minimum wall-clock time elapsed between signals; rate-limits equivocation.

use nebu::Goldilocks;

/// A VDF proof: input → T sequential squarings → output.
#[derive(Clone, Debug, PartialEq)]
pub struct VdfProof {
    /// Input to the VDF (derived from previous signal hash).
    pub input: u64,
    /// Output after T sequential squarings.
    pub output: u64,
    /// Number of sequential squarings (difficulty parameter).
    pub iterations: u64,
}

/// Evaluate the VDF: compute x^(2^T) by T sequential squarings.
///
/// Inherently sequential — cannot be parallelised. Time is proportional
/// to `iterations` regardless of hardware.
pub fn evaluate(input: u64, iterations: u64) -> VdfProof {
    let mut x = Goldilocks::new(input);
    if x.is_zero() {
        x = Goldilocks::ONE;
    }
    for _ in 0..iterations {
        x = x.square();
    }
    VdfProof { input, output: x.as_u64(), iterations }
}

/// Verify a VDF proof by re-evaluation.
///
/// For signal-scale (local sync), full re-evaluation is acceptable.
/// For global scale, Pietrzak/Wesolowski proofs would be needed.
pub fn verify(proof: &VdfProof) -> bool {
    if proof.iterations == 0 {
        return proof.input == proof.output || (proof.input == 0 && proof.output == 1);
    }
    evaluate(proof.input, proof.iterations).output == proof.output
}

/// Derive a deterministic VDF challenge from a signal hash.
pub fn challenge_from_hash(hash: &[u8; 32]) -> u64 {
    let mut val: u64 = 0;
    for (i, &byte) in hash.iter().take(8).enumerate() {
        val |= (byte as u64) << (i * 8);
    }
    if val == 0 { 1 } else { val }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vdf_evaluate_and_verify() {
        let proof = evaluate(42, 1000);
        assert!(verify(&proof));
        assert_ne!(proof.output, 42);
    }

    #[test]
    fn vdf_is_deterministic() {
        assert_eq!(evaluate(42, 1000), evaluate(42, 1000));
    }

    #[test]
    fn vdf_tampered_output_fails() {
        let mut proof = evaluate(42, 1000);
        proof.output ^= 1;
        assert!(!verify(&proof));
    }

    #[test]
    fn vdf_tampered_iterations_fails() {
        let mut proof = evaluate(42, 1000);
        proof.iterations += 1;
        assert!(!verify(&proof));
    }

    #[test]
    fn vdf_sequential_property() {
        let full = evaluate(7, 2000);
        let half = evaluate(evaluate(7, 1000).output, 1000);
        assert_eq!(full.output, half.output);
    }

    #[test]
    fn vdf_zero_iterations() {
        let proof = evaluate(42, 0);
        assert_eq!(proof.input, proof.output);
        assert!(verify(&proof));
    }

    #[test]
    fn challenge_from_hash_is_deterministic() {
        let h = [0xab; 32];
        assert_eq!(challenge_from_hash(&h), challenge_from_hash(&h));
        assert_ne!(challenge_from_hash(&h), 0);
    }
}
