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
use nox::{Order, Reduction};

/// Construct a Goldilocks field element.
pub fn g(v: u64) -> Goldilocks {
    Goldilocks::new(v)
}

/// Legacy raw-fold fixture statement. This checks the folding primitive only;
/// verifier-derived public/state execution proofs below bind actual semantics.
pub fn zero_statement() -> zheng::Statement {
    zheng::Statement {
        program_hash: [0u8; 32],
        input_hash: [0u8; 32],
        output_hash: [0u8; 32],
        focus_bound: 0,
        bbg_root: [0; 32],
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
pub fn make_bbg_object<const N: usize>(order: &mut Reduction<N>, limbs: [Goldilocks; 4]) -> Order {
    let al0 = order.atom(limbs[0]).unwrap();
    let al1 = order.atom(limbs[1]).unwrap();
    let al2 = order.atom(limbs[2]).unwrap();
    let al3 = order.atom(limbs[3]).unwrap();
    let inner = order.pair(al2, al3).unwrap();
    let mid = order.pair(al1, inner).unwrap();
    let root_pair = order.pair(al0, mid).unwrap();
    let rest = order.atom(Goldilocks::ZERO).unwrap();
    order.pair(root_pair, rest).unwrap()
}

/// Build the 4-limb BBG object from a live `BbgState` root.
pub fn bbg_object_from_state<const N: usize>(
    order: &mut Reduction<N>,
    state: &bbg::BbgState,
) -> Order {
    make_bbg_object(order, bbg::dim::goldilocks_from_bytes32(&state.root()))
}

/// Build `[17 [[1 ns] [1 key]]]` — look formula with quoted ns and key.
pub fn make_look_formula<const N: usize>(order: &mut Reduction<N>, ns: u64, key: u64) -> Order {
    let t17 = order.atom(g(17)).unwrap();
    let t1 = order.atom(g(1)).unwrap();
    let vns = order.atom(g(ns)).unwrap();
    let vkey = order.atom(g(key)).unwrap();
    let ns_f = order.pair(t1, vns).unwrap();
    let key_f = order.pair(t1, vkey).unwrap();
    let body = order.pair(ns_f, key_f).unwrap();
    order.pair(t17, body).unwrap()
}

/// Build `[tag [[1 a] [1 b]]]` — binary field formula (5, 6, 7, 9 or 10).
/// Operands are Goldilocks atoms in the current tag-free nox model.
pub fn make_field_binop<const N: usize>(
    order: &mut Reduction<N>,
    tag: u64,
    a: u64,
    b: u64,
) -> Order {
    let t = order.atom(g(tag)).unwrap();
    let t1 = order.atom(g(1)).unwrap();
    let va = order.atom(g(a)).unwrap();
    let vb = order.atom(g(b)).unwrap();
    let qa = order.pair(t1, va).unwrap();
    let qb = order.pair(t1, vb).unwrap();
    let body = order.pair(qa, qb).unwrap();
    order.pair(t, body).unwrap()
}

/// Build `[tag [[1 a] [1 b]]]` — binary word formula (11, 12 or 14).
/// Word operations refine the same atoms to the 32-bit range.
pub fn make_word_binop<const N: usize>(
    order: &mut Reduction<N>,
    tag: u64,
    a: u64,
    b: u64,
) -> Order {
    let t = order.atom(g(tag)).unwrap();
    let t1 = order.atom(g(1)).unwrap();
    let va = order.atom(g(a)).unwrap();
    let vb = order.atom(g(b)).unwrap();
    let qa = order.pair(t1, va).unwrap();
    let qb = order.pair(t1, vb).unwrap();
    let body = order.pair(qa, qb).unwrap();
    order.pair(t, body).unwrap()
}

/// Build `[16 [[1 call_tag] [1 0]]]` — call formula where the check always returns 0 (accepted).
/// `call_tag` identifies which witness the CallProvider should supply.
pub fn make_call_formula<const N: usize>(order: &mut Reduction<N>, call_tag: u64) -> Order {
    let t16 = order.atom(g(16)).unwrap();
    let t1 = order.atom(g(1)).unwrap();
    let ctag = order.atom(g(call_tag)).unwrap();
    let zero = order.atom(g(0)).unwrap();
    let tag_f = order.pair(t1, ctag).unwrap(); // [1 call_tag]
    let check_f = order.pair(t1, zero).unwrap(); // [1 0] — always accepted
    let body = order.pair(tag_f, check_f).unwrap();
    order.pair(t16, body).unwrap()
}

/// `BbgState` seeded with a neuron, one cyberlink (p2→p3), and a Time snapshot at height 0.
pub fn seeded_bbg_state() -> bbg::BbgState {
    let p1: bbg::Particle = [1u8; 32];
    let p2: bbg::Particle = [2u8; 32];
    let p3: bbg::Particle = [3u8; 32];
    let mut s = bbg::BbgState::new();
    s.neurons.insert(
        p1,
        bbg::types::NeuronRecord {
            focus: 50_000,
            karma: 0,
            stake: 0,
        },
    );
    s.insert(&bbg::Signal {
        neuron: p1,
        links: vec![bbg::Cyberlink {
            from: p2,
            to: p3,
            token: [0u8; 32],
            amount: 100,
            valence: 1,
        }],
        box_moves: vec![],
        height: 0,
    })
    .unwrap();
    s.time.insert(0, [99u8; 32]);
    s.refresh_root();
    s
}

/// Reify this bounded test arena's actual formula/object for the current
/// verifier-derived execution relation. No prover-supplied CCS is accepted.
pub fn execution_noun<const N: usize>(
    arena: &Reduction<N>,
    id: Order,
) -> zheng::execution::ExecutionNoun {
    use nox::data::Data;
    use zheng::execution::ExecutionNoun::{Atom, Pair};
    match arena.get(id).expect("test noun exists").inner {
        Data::Atom { value } => Atom(value.as_u64()),
        Data::Pair { left, right } => Pair(
            Box::new(execution_noun(arena, left)),
            Box::new(execution_noun(arena, right)),
        ),
    }
}

pub fn noun_leaves<const N: usize>(arena: &Reduction<N>, id: Order) -> Vec<u64> {
    use nox::data::Data;
    match arena.get(id).expect("test result exists").inner {
        Data::Atom { value } => vec![value.as_u64()],
        Data::Pair { left, right } => {
            let mut out = noun_leaves(arena, left);
            out.extend(noun_leaves(arena, right));
            out
        }
    }
}

pub fn result(outcome: nox::Outcome) -> Order {
    match outcome {
        nox::Outcome::Ok(id, _) => id,
        other => panic!("execution must succeed: {other:?}"),
    }
}

/// Bind the actual public object and formula, independently of the legacy raw
/// trace fold. This public development proof discloses its witness columns.
pub fn verify_public_execution<const N: usize>(
    arena: &Reduction<N>,
    object: Order,
    formula: Order,
    output: Order,
) {
    use zheng::execution::{ExecutionNoun, prove_execution, verify_execution};
    let atom = ExecutionNoun::Atom;
    let pair = |a, b| ExecutionNoun::Pair(Box::new(a), Box::new(b));
    let quote = |v| pair(atom(1), v);
    let program = pair(
        atom(2),
        pair(
            quote(execution_noun(arena, object)),
            quote(execution_noun(arena, formula)),
        ),
    );
    let (statement, proof) =
        prove_execution(&program, &[], 10_000).expect("current public execution proof");
    assert_eq!(statement.public_output, noun_leaves(arena, output));
    verify_execution(&statement, &proof).expect("verifier-derived public execution relation");
    let mut wrong = statement.clone();
    wrong.public_output[0] = (wrong.public_output[0] + 1) % nebu::field::P;
    assert!(
        verify_execution(&wrong, &proof).is_err(),
        "wrong actual result must fail"
    );
}

/// Authenticate a flat public BBG coordinate against the caller's exact root.
/// QueryProof includes the current complete-table context; a sampled legacy
/// LookOpening by itself deliberately fails the owner's verification API.
pub fn authenticated_value(
    state: &bbg::BbgState,
    root: &[u8; 32],
    namespace: u64,
    index: u64,
) -> Option<u64> {
    let dim = bbg::Dim::from_u64(namespace)?;
    let query = bbg::proof::open_cell(state, dim, index.try_into().ok()?)?;
    if !bbg::verify_query_at(&query, root, namespace, index) {
        return None;
    }
    Some(u64::from_le_bytes(
        query.value_bytes.as_slice().try_into().ok()?,
    ))
}

pub fn verify_state_execution<const N: usize>(
    arena: &Reduction<N>,
    formula: Order,
    state: &bbg::BbgState,
    expected: &[u64],
) -> (
    zheng::execution::state::StateStatement,
    zheng::execution::DirectProof,
) {
    let root = state.root();
    let limbs = bbg::dim::goldilocks_from_bytes32(&root).map(|v| v.as_u64());
    let mut lookup = |ns, index| authenticated_value(state, &root, ns, index);
    let (statement, proof) = zheng::execution::state::prove_state_execution(
        &execution_noun(arena, formula),
        &[],
        10_000,
        limbs,
        true,
        *hemera::hash(b"cybergraph/stack-state-test/1").as_bytes(),
        &mut lookup,
    )
    .expect("public state execution with authenticated BBG coordinates");
    assert_eq!(statement.execution.public_output, expected);
    statement
        .verify(&proof, &mut lookup)
        .expect("public state execution verifies");
    for i in 0..4 {
        let mut wrong = statement.clone();
        wrong.state_root[i] = (wrong.state_root[i] + 1) % nebu::field::P;
        assert!(
            wrong.verify(&proof, &mut lookup).is_err(),
            "every state-root limb is bound"
        );
    }
    let mut wrong = statement.clone();
    wrong.context[0] ^= 1;
    assert!(
        wrong.verify(&proof, &mut lookup).is_err(),
        "execution context is bound"
    );
    assert!(
        statement.verify(&proof, &mut |_, _| None).is_err(),
        "missing authentication must fail"
    );
    (statement, proof)
}

pub fn verify_look_openings(
    state: &bbg::BbgState,
    trace: &nox::VecTrace,
    openings: &[zheng::LookOpening],
) {
    let rows: Vec<_> = trace.0.iter().filter(|r| r.r()[0] == 17).collect();
    assert_eq!(
        openings.len(),
        rows.len(),
        "one complete opening per successful look"
    );
    let root = state.root();
    let limbs = bbg::dim::goldilocks_from_bytes32(&root).map(|v| v.as_u64());
    for (row, opening) in rows.iter().zip(openings) {
        let ns = row.r()[5];
        let index = row.r()[6];
        assert_eq!(opening.namespace.as_u64(), ns);
        assert_eq!(opening.value.as_u64(), row.r()[7]);
        assert_eq!([row.r()[4], row.r()[11], row.r()[12], row.r()[13]], limbs);
        let query =
            bbg::proof::open_cell(state, bbg::Dim::from_u64(ns).unwrap(), index as usize).unwrap();
        assert!(bbg::verify_opening_with_context(
            opening, &query, &root, index
        ));
        assert!(
            !bbg::verify_opening(opening),
            "context-free sampled proof has no authority"
        );
        let mut wrong = query.clone();
        wrong.value_bytes[0] ^= 1;
        assert!(!bbg::verify_opening_with_context(
            opening, &wrong, &root, index
        ));
        let mut wrong_root = root;
        wrong_root[0] ^= 1;
        assert!(!bbg::verify_opening_with_context(
            opening,
            &query,
            &wrong_root,
            index
        ));
        assert!(!bbg::verify_opening_with_context(
            opening,
            &query,
            &root,
            index + 1
        ));
    }
}

pub fn refuse_recursive_look(
    trace: &nox::VecTrace,
    state: &bbg::BbgState,
    openings: &[zheng::LookOpening],
) {
    let mut statement = zero_statement();
    statement.bbg_root = state.root();
    assert!(
        matches!(
            zheng::commit(trace, &[], &[], openings, &statement, &default_params()),
            Err(zheng::CommitError::UnsupportedRecursiveOpening)
        ),
        "TensorMerkle recursion is explicitly unsupported"
    );
}
