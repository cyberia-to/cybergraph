// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: nox axis pattern (tag 0) → zheng proof.
//!
//! Pipeline under test:
//!   nox::reduce()         — executes axis formula, records axis address in trace
//!   AxisOpening           — Brakedown commitment to the noun polynomial + opening
//!   zheng::commit()       — folds main steps + verifier_steps (opening binding)
//!   zheng::verify()       — checks SuperSpartan proof
//!
//! The AxisOpening uses a synthetic polynomial (not the actual noun tree).
//! This exercises the full commit/fold/verify pipeline for the axis pattern;
//! binding the opening to the actual noun polynomial is enforced at application level.

mod common;

use common::{g, zero_statement, default_params};

use nebu::Goldilocks;
use nox::{reduce, Order, Tag, VecTrace, NullCalls};
use zheng::{commit, verify, AxisOpening};
use lens::brakedown::Brakedown;
use lens::{Lens, MultilinearPoly, Transcript as LensTx};

const ORDER_SIZE: usize = 1024;

/// Build a valid Brakedown commitment + opening for a small 2-variable polynomial.
/// The polynomial is `f(x0,x1) ∈ {1,2,3,4}` evaluated at point `(0,0)` → value 1.
fn make_axis_opening() -> AxisOpening {
    let evals: Vec<Goldilocks> = (1u64..=4).map(Goldilocks::new).collect();
    let poly       = MultilinearPoly::new(evals);
    let commitment = Brakedown::commit(&poly);
    let point      = vec![Goldilocks::ZERO, Goldilocks::ZERO];
    let value      = Goldilocks::new(1);
    let opening = {
        let mut lt = LensTx::new(b"axis-open");
        Brakedown::open(&poly, &point, &mut lt)
    };
    AxisOpening { commitment, point, value, opening }
}

/// Full pipeline: two axis(s, 1) reduces → two AxisOpenings → zheng commit → verify.
///
/// axis(s, 1) is the identity: it returns `s` and records addr=1 in r[5].
/// Two consecutive axis rows give build_ccs_from_trace one main step to fold.
#[test]
fn axis_identity_full_proof_roundtrip() {
    let mut order  = Order::<ORDER_SIZE>::new();
    let s      = order.atom(g(7), Tag::Field).unwrap();
    let tag0   = order.atom(g(0), Tag::Field).unwrap();
    let addr1  = order.atom(g(1), Tag::Field).unwrap();
    let axis_f = order.cell(tag0, addr1).unwrap();

    let mut trace = VecTrace::default();
    reduce(&mut order, s, axis_f, 100, &NullCalls, &mut trace);
    reduce(&mut order, s, axis_f,  99, &NullCalls, &mut trace);
    assert_eq!(trace.0.len(), 2, "two axis reduces → two rows");

    let openings = [make_axis_opening(), make_axis_opening()];
    let stmt     = zero_statement();
    let proof    = commit(&trace, &[], &openings, &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("axis proof must verify");
}

/// Full pipeline: axis into a nested cell at address 4 (left→left) → commit → verify.
///
/// Object: `[[A | B] | C]`; axis 4 navigates left→left → A.
/// Two reduces with the same formula give a pair of axis rows for CCS folding.
#[test]
fn axis_nested_cell_full_proof_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let a   = order.atom(g(10), Tag::Field).unwrap();
    let b   = order.atom(g(20), Tag::Field).unwrap();
    let c   = order.atom(g(30), Tag::Field).unwrap();
    let ab  = order.cell(a, b).unwrap();
    let obj = order.cell(ab, c).unwrap();    // [[10|20]|30]

    let tag0  = order.atom(g(0),  Tag::Field).unwrap();
    let addr4 = order.atom(g(4),  Tag::Field).unwrap();  // axis 4: left→left → A = 10
    let axis_f = order.cell(tag0, addr4).unwrap();

    let mut trace = VecTrace::default();
    reduce(&mut order, obj, axis_f, 100, &NullCalls, &mut trace);
    reduce(&mut order, obj, axis_f,  99, &NullCalls, &mut trace);
    assert_eq!(trace.0.len(), 2);

    let openings = [make_axis_opening(), make_axis_opening()];
    let stmt     = zero_statement();
    let proof    = commit(&trace, &[], &openings, &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("axis-4 proof must verify");
}
