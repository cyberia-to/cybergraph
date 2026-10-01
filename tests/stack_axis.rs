// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Actual nox axis execution, native TensorMerkle openings and current public
//! execution proofs. Legacy recursive-opening folds explicitly refuse this profile.
mod common;
use common::{default_params, g, result, verify_public_execution, zero_statement};
use lens::{Lens, MultilinearPoly, Transcript, brakedown::Brakedown};
use nebu::Goldilocks;
use nox::{CallProvider, LookProvider, Order, Reduction, VecTrace, reduce};
use zheng::{AxisOpening, CommitError, commit};

struct CommittedObject {
    object: Order,
    commitment: [u8; 32],
}
impl LookProvider for CommittedObject {
    fn look(&self, _: Goldilocks, _: Goldilocks, _: Goldilocks) -> Option<Goldilocks> {
        None
    }
}
impl<const N: usize> CallProvider<N> for CommittedObject {
    fn provide(&self, _: &mut Reduction<N>, _: Goldilocks, _: Order) -> Option<Order> {
        None
    }
    fn axis_commitment(&self, object: u64) -> Option<[u8; 32]> {
        (object == self.object as u64).then_some(self.commitment)
    }
}

/// The polynomial contains the actual fixture's leaves, with explicit zero
/// padding. Execution correctness is separately bound by the public relation.
fn opening(values: &[u64], point: &[u64], value: u64) -> AxisOpening {
    let poly = MultilinearPoly::new(values.iter().map(|&v| g(v)).collect());
    let commitment = Brakedown::commit(&poly);
    let point: Vec<_> = point.iter().map(|&v| g(v)).collect();
    let seed = b"cybergraph/axis-fixture/1".to_vec();
    let proof = Brakedown::open(&poly, &point, &mut Transcript::new(&seed));
    assert!(Brakedown::verify(
        &commitment,
        &point,
        g(value),
        &proof,
        &mut Transcript::new(&seed)
    ));
    assert!(!Brakedown::verify(
        &commitment,
        &point,
        g(value + 1),
        &proof,
        &mut Transcript::new(&seed)
    ));
    AxisOpening {
        commitment,
        point,
        value: g(value),
        opening: proof,
        transcript_seed: seed,
    }
}

fn run<const N: usize>(
    arena: &mut Reduction<N>,
    object: Order,
    address: u64,
    expected: u64,
    openings: [AxisOpening; 2],
) {
    let provider = CommittedObject {
        object,
        commitment: openings[0].commitment.as_bytes().try_into().unwrap(),
    };
    let zero = arena.atom(g(0)).unwrap();
    let address = arena.atom(g(address)).unwrap();
    let formula = arena.pair(zero, address).unwrap();
    let mut trace = VecTrace::default();
    let output = result(reduce(arena, object, formula, 100, &provider, &mut trace));
    assert_eq!(arena.atom_value(output).unwrap().as_u64(), expected);
    assert_eq!(
        result(reduce(arena, object, formula, 99, &provider, &mut trace)),
        output
    );
    assert_eq!(trace.0.len(), 2);
    verify_public_execution(arena, object, formula, output);
    assert!(matches!(
        commit(
            &trace,
            &[],
            &openings,
            &[],
            &zero_statement(),
            &default_params()
        ),
        Err(CommitError::UnsupportedRecursiveOpening)
    ));
}

#[test]
fn axis_identity_public_execution_and_native_opening() {
    let mut arena = Reduction::<1024>::new();
    let object = arena.atom(g(7)).unwrap();
    run(
        &mut arena,
        object,
        1,
        7,
        [opening(&[7, 0], &[0], 7), opening(&[7, 0], &[0], 7)],
    );
}

#[test]
fn axis_nested_pair_public_execution_and_native_opening() {
    let mut arena = Reduction::<1024>::new();
    let a = arena.atom(g(10)).unwrap();
    let b = arena.atom(g(20)).unwrap();
    let c = arena.atom(g(30)).unwrap();
    let ab = arena.pair(a, b).unwrap();
    let object = arena.pair(ab, c).unwrap();
    run(
        &mut arena,
        object,
        4,
        10,
        [
            opening(&[10, 20, 30, 0], &[0, 0], 10),
            opening(&[10, 20, 30, 0], &[0, 0], 10),
        ],
    );
}
