// ---
// tags: cybergraph, rust, test
// crystal-type: source
// crystal-domain: cyber
// ---
//! Shared builders for cybergraph integration tests.
//!
//! Every test binary that does `mod common;` gets a copy of these helpers.
//! `#![allow(dead_code)]` suppresses warnings for helpers unused in a given binary.

#![allow(dead_code)]

use nebu::Goldilocks;
use nox::{OrderId, Order, Tag};

/// Construct a Goldilocks field element.
pub fn g(v: u64) -> Goldilocks {
    Goldilocks::new(v)
}

/// Statement with all-zero hashes and no focus bound — suitable for basic roundtrip tests.
pub fn zero_statement() -> zheng::Statement {
    zheng::Statement {
        program_hash: [0u8; 32],
        input_hash:   [0u8; 32],
        output_hash:  [0u8; 32],
        focus_bound:  0,
    }
}

/// Default proof parameters.
pub fn default_params() -> zheng::ProofParams {
    zheng::ProofParams::default()
}

/// Build the 4-limb BBG root object used by nox's look pattern (tag 17).
///
/// Layout: `[[l0 | [l1 | [l2 | l3]]] | rest]`
/// Limb axes: l0=4, l1=10, l2=22, l3=23 — matching `BBG_ROOT_LIMB_AXES` in nox.
pub fn make_bbg_object<const N: usize>(order: &mut Order<N>, limbs: [Goldilocks; 4]) -> OrderId {
    let al0 = order.atom(limbs[0], Tag::Field).unwrap();
    let al1 = order.atom(limbs[1], Tag::Field).unwrap();
    let al2 = order.atom(limbs[2], Tag::Field).unwrap();
    let al3 = order.atom(limbs[3], Tag::Field).unwrap();
    let inner     = order.pair(al2, al3).unwrap();
    let mid       = order.pair(al1, inner).unwrap();
    let root_pair = order.pair(al0, mid).unwrap();
    let rest      = order.atom(Goldilocks::ZERO, Tag::Field).unwrap();
    order.pair(root_pair, rest).unwrap()
}

/// Build the 4-limb BBG object from a live `BbgState` root.
pub fn bbg_object_from_state<const N: usize>(
    order: &mut Order<N>,
    state: &bbg::BbgState,
) -> OrderId {
    make_bbg_object(order, bbg::dim::goldilocks_from_bytes32(&state.root))
}

/// Build `[17 [[1 ns] [1 key]]]` — look formula with quoted ns and key.
pub fn make_look_formula<const N: usize>(order: &mut Order<N>, ns: u64, key: u64) -> OrderId {
    let t17  = order.atom(g(17), Tag::Field).unwrap();
    let t1   = order.atom(g(1),  Tag::Field).unwrap();
    let vns  = order.atom(g(ns),  Tag::Field).unwrap();
    let vkey = order.atom(g(key), Tag::Field).unwrap();
    let ns_f  = order.pair(t1, vns).unwrap();
    let key_f = order.pair(t1, vkey).unwrap();
    let body  = order.pair(ns_f, key_f).unwrap();
    order.pair(t17, body).unwrap()
}

/// Build `[tag [[1 a] [1 b]]]` — binary field operation formula (tags 5–10).
/// Operands use `Tag::Field` (Goldilocks elements).
pub fn make_field_binop<const N: usize>(order: &mut Order<N>, tag: u64, a: u64, b: u64) -> OrderId {
    let t  = order.atom(g(tag), Tag::Field).unwrap();
    let t1 = order.atom(g(1),   Tag::Field).unwrap();
    let va = order.atom(g(a),   Tag::Field).unwrap();
    let vb = order.atom(g(b),   Tag::Field).unwrap();
    let qa   = order.pair(t1, va).unwrap();
    let qb   = order.pair(t1, vb).unwrap();
    let body = order.pair(qa, qb).unwrap();
    order.pair(t, body).unwrap()
}

/// Build `[tag [[1 a] [1 b]]]` — binary bitwise operation formula (tags 11–14).
/// Operands use `Tag::Word` (machine words, not field elements).
pub fn make_word_binop<const N: usize>(order: &mut Order<N>, tag: u64, a: u64, b: u64) -> OrderId {
    let t  = order.atom(g(tag), Tag::Field).unwrap();
    let t1 = order.atom(g(1),   Tag::Field).unwrap();
    let va = order.atom(g(a),   Tag::Word).unwrap();
    let vb = order.atom(g(b),   Tag::Word).unwrap();
    let qa   = order.pair(t1, va).unwrap();
    let qb   = order.pair(t1, vb).unwrap();
    let body = order.pair(qa, qb).unwrap();
    order.pair(t, body).unwrap()
}

/// Build `[16 [[1 call_tag] [1 0]]]` — call formula where the check always returns 0 (accepted).
/// `call_tag` identifies which witness the CallProvider should supply.
pub fn make_call_formula<const N: usize>(order: &mut Order<N>, call_tag: u64) -> OrderId {
    let t16    = order.atom(g(16),       Tag::Field).unwrap();
    let t1     = order.atom(g(1),        Tag::Field).unwrap();
    let ctag   = order.atom(g(call_tag), Tag::Field).unwrap();
    let zero   = order.atom(g(0),        Tag::Field).unwrap();
    let tag_f   = order.pair(t1, ctag).unwrap();   // [1 call_tag]
    let check_f = order.pair(t1, zero).unwrap();   // [1 0] — always accepted
    let body    = order.pair(tag_f, check_f).unwrap();
    order.pair(t16, body).unwrap()
}

/// `BbgState` seeded with a neuron, one cyberlink (p2→p3), and a Time snapshot at height 0.
pub fn seeded_bbg_state() -> bbg::BbgState {
    let p1: bbg::Particle = [1u8; 32];
    let p2: bbg::Particle = [2u8; 32];
    let p3: bbg::Particle = [3u8; 32];
    let mut s = bbg::BbgState::new();
    s.neurons.insert(p1, bbg::types::NeuronRecord { focus: 50_000, karma: 0, stake: 0 });
    s.insert(&bbg::Signal {
        neuron:    p1,
        links:     vec![bbg::Cyberlink { from: p2, to: p3, token: [0u8; 32], amount: 100, valence: 1 }],
        box_moves: vec![],
        height:    0,
    }).unwrap();
    s.time.insert(0, [99u8; 32]);
    s
}
