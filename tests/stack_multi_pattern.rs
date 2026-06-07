// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Integration tests: mixed-pattern nox traces → zheng proof.
//!
//! These tests verify that traces containing rows from multiple pattern types
//! (field-arithmetic, hash, axis, look) can be committed and verified in a
//! single zheng proof. Each test builds a VecTrace from multiple reduce() calls
//! using different formulas, then provides all required auxiliary data:
//!   HashAux       — one per Poseidon2 hash block
//!   AxisOpening   — one per axis row
//!   LookOpening   — one per look row (from ProofLookProvider)

mod common;

use common::{
    g, zero_statement, default_params,
    make_field_binop, make_look_formula, bbg_object_from_state,
    seeded_bbg_state,
};

use nebu::Goldilocks;
use nox::{reduce, Order, Tag, VecTrace, NullCalls, Outcome};
use zheng::{commit, verify, HashAux, AxisOpening};
use lens::brakedown::Brakedown;
use lens::{Lens, MultilinearPoly, Transcript as LensTx};
use bbg::ProofLookProvider;

const ORDER_SIZE: usize = 1024;

/// Synthetic Brakedown opening for a 2-variable polynomial.
/// Evaluated at (0,0) → value=1.  Used as a stand-in when no real noun tree exists.
fn make_axis_opening() -> AxisOpening {
    let evals: Vec<Goldilocks> = (1u64..=4).map(Goldilocks::new).collect();
    let poly       = MultilinearPoly::new(evals);
    let commitment = Brakedown::commit(&poly);
    let point      = vec![Goldilocks::ZERO, Goldilocks::ZERO];
    let value      = Goldilocks::new(1);
    let transcript_seed = b"axis-open".to_vec();
    let opening = { let mut lt = LensTx::new(&transcript_seed); Brakedown::open(&poly, &point, &mut lt) };
    AxisOpening { commitment, point, value, opening, transcript_seed }
}

// ── mixed traces ──────────────────────────────────────────────────────────────

/// One add reduce + one hash reduce + one axis reduce → single commit → verify.
///
/// Trace layout:
///   rows 0–2  : add formula   [quote|quote|add]  — 3 rows
///   rows 3–28 : hash formula  [quote|…24 rounds…|squeeze] — 26 rows
///   row  29   : axis formula  [axis] — 1 row
///
/// Required auxiliaries: 1 HashAux (one hash block), 1 AxisOpening (one axis row).
#[test]
fn mixed_add_hash_axis_trace_roundtrip() {
    let mut order = Order::<ORDER_SIZE>::new();
    let obj = order.atom(Goldilocks::new(0), Tag::Field).unwrap();

    // Add formula
    let add_f = make_field_binop(&mut order, 5, 3, 7);
    let mut trace = VecTrace::default();
    reduce(&mut order, obj, add_f, 1000, &NullCalls, &mut trace);

    // Hash formula: [15 [1 s]] where s = atom(42)
    let s      = order.atom(Goldilocks::new(42), Tag::Field).unwrap();
    let tag1   = order.atom(Goldilocks::new(1),  Tag::Field).unwrap();
    let tag15  = order.atom(Goldilocks::new(15), Tag::Field).unwrap();
    let qf     = order.cell(tag1,  s).unwrap();
    let hash_f = order.cell(tag15, qf).unwrap();
    reduce(&mut order, s, hash_f, 500, &NullCalls, &mut trace);

    let in_digest = *order.digest(s).unwrap();
    let rate = [
        in_digest[0], in_digest[1], in_digest[2], in_digest[3],
        Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO, Goldilocks::ZERO,
    ];
    let hash_aux = HashAux { rate };

    // Axis formula: [0 1] — axis(1) is identity
    let tag0  = order.atom(g(0), Tag::Field).unwrap();
    let addr1 = order.atom(g(1), Tag::Field).unwrap();
    let axis_f = order.cell(tag0, addr1).unwrap();
    reduce(&mut order, obj, axis_f, 100, &NullCalls, &mut trace);

    let hash_rows = trace.0.iter().filter(|r| r.r()[0] == 15).count();
    let axis_rows = trace.0.iter().filter(|r| r.r()[0] == 0).count();
    assert_eq!(hash_rows, 25, "one hash block emits 25 tag-15 rows (24 rounds + squeeze)");
    assert_eq!(axis_rows, 1, "one axis reduce emits 1 row");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[hash_aux], &[make_axis_opening()], &[], &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("mixed add+hash+axis proof must verify");
}

/// One add reduce + one look reduce → single commit → verify.
///
/// The add uses NullCalls (no BBG needed). The look uses ProofLookProvider to
/// produce a real opening against the seeded BbgState.
#[test]
fn mixed_add_look_trace_roundtrip() {
    let state = seeded_bbg_state();
    let prov  = ProofLookProvider::new(&state);

    let mut order = Order::<ORDER_SIZE>::new();
    let obj = order.atom(g(0), Tag::Field).unwrap();

    // Add formula (no BBG needed)
    let add_f = make_field_binop(&mut order, 5, 4, 8);
    let mut trace = VecTrace::default();
    reduce(&mut order, obj, add_f, 1000, &NullCalls, &mut trace);

    // Look formula: look(Time=8, height=0) against the seeded state
    let bbg_obj  = bbg_object_from_state(&mut order, &state);
    let look_f   = make_look_formula(&mut order, 8, 0);  // Dim::Time=8, key=0
    let outcome  = reduce(&mut order, bbg_obj, look_f, 1000, &prov, &mut trace);
    assert!(matches!(outcome, Outcome::Ok(_, _)), "look must succeed on seeded state");

    let look_openings = prov.take_look_openings();
    assert_eq!(look_openings.len(), 1, "one look row → one opening");

    let look_rows = trace.0.iter().filter(|r| r.r()[0] == 17).count();
    assert_eq!(look_rows, 1, "one look reduce → one look row");

    let stmt  = zero_statement();
    let proof = commit(&trace, &[], &[], &look_openings, &stmt, &default_params()).unwrap();
    verify(&proof, &stmt, &default_params()).expect("mixed add+look proof must verify");
}
