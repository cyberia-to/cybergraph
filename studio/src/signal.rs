//! The signal kernel: local-first, real primitives, the spec's shapes.
//!
//! - a particle is the hemera hash of a file's bytes (typed text is a file)
//! - the neuron is a mudra domain-scoped secp256k1 key
//! - a cyberlink is the five stored fields of `specs/cyberlink.md`:
//!   from, to, coin, amount, valence
//! - a signal carries each link under its author (`specs/signal.md`); commit
//!   hashes the canonical body and signs it ADR-036 style; verify re-checks
//!   the body particle, the neuron and the signature
//! - an intent (`specs/intent.md`) is a signed scope with a sealer and an offer
//!
//! Everything lives in localStorage under `cg.studio.*`. Nothing here talks
//! to a chain: a committed signal is signed and local.

use serde::{Deserialize, Serialize};

pub const SEED_KEY: &str = "cg.studio.seed";
pub const NAMES_KEY: &str = "cg.studio.particles";
pub const SIGNALS_KEY: &str = "cg.studio.signals";
pub const SIGNAL_SEQ_KEY: &str = "cg.studio.signal_seq";
pub const INTENTS_KEY: &str = "cg.studio.intents";
pub const INTENT_SEQ_KEY: &str = "cg.studio.intent_seq";

pub const DOMAIN: &str = "cybergraph.studio";
pub const HRP: &str = "cyber";
pub const DEFAULT_COIN: &str = "CYB";

// ─── storage ──────────────────────────────────────────────────────────

fn ls_get(key: &str) -> Option<String> {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|ls| ls.get_item(key).ok().flatten())
}

fn ls_set(key: &str, raw: &str) {
    if let Some(ls) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = ls.set_item(key, raw);
    }
}

fn load_json<T: for<'de> Deserialize<'de>>(key: &str) -> Vec<T> {
    ls_get(key)
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

fn save_json<T: Serialize>(key: &str, list: &[T]) {
    if let Ok(raw) = serde_json::to_string(list) {
        ls_set(key, &raw);
    }
}

fn next_id(seq_key: &str, prefix: &str) -> String {
    let n: u64 = ls_get(seq_key).and_then(|s| s.parse().ok()).unwrap_or(0) + 1;
    ls_set(seq_key, &n.to_string());
    format!("{prefix}-{n}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.is_ascii() {
        return None;
    }
    (0..s.len() / 2)
        .map(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).ok())
        .collect()
}

pub fn is_particle(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| c.is_ascii_hexdigit())
}

// ─── neuron: the signing identity ─────────────────────────────────────

/// The local neuron: bech32 wire id, compressed pubkey, hemera native id.
#[derive(Clone, Debug, PartialEq)]
pub struct NeuronInfo {
    pub bech32: String,
    pub pubkey_hex: String,
    pub native_hex: String,
}

fn load_or_create_entropy() -> [u8; 32] {
    if let Some(saved) = ls_get(SEED_KEY).and_then(|s| unhex(&s)) {
        if let Ok(e) = <[u8; 32]>::try_from(saved) {
            return e;
        }
    }
    let mut e = [0u8; 32];
    getrandom::getrandom(&mut e).expect("os entropy");
    ls_set(SEED_KEY, &hex(&e));
    e
}

fn domain_key() -> mudra::domain::DomainKey {
    let entropy = load_or_create_entropy();
    mudra::domain::DomainKey::derive(&entropy, DOMAIN, HRP).expect("domain key derivation")
}

pub fn neuron() -> NeuronInfo {
    let key = domain_key();
    NeuronInfo {
        bech32: key.bech32.clone(),
        pubkey_hex: hex(&key.pubkey),
        native_hex: hex(&key.native),
    }
}

// ─── particles: content addresses and the names this page knows ───────

/// The particle of some bytes: their hemera hash, hex.
pub fn particle_of(bytes: &[u8]) -> String {
    hex(hemera::hash(bytes).as_bytes())
}

/// A particle this page can name: typed text, a dropped file, the neuron.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Named {
    pub particle: String,
    pub name: String,
    /// text | file | neuron
    pub kind: String,
    /// byte length of the hashed content (0 for the neuron)
    #[serde(default)]
    pub size: u64,
}

pub fn load_names() -> Vec<Named> {
    load_json(NAMES_KEY)
}

/// Remember a name for a particle. The first name a particle got stays.
pub fn remember(particle: &str, name: &str, kind: &str, size: u64) {
    let mut names = load_names();
    if !names.iter().any(|n| n.particle == particle) {
        names.push(Named {
            particle: particle.to_string(),
            name: name.to_string(),
            kind: kind.to_string(),
            size,
        });
        save_json(NAMES_KEY, &names);
    }
}

pub fn find_by_name(name: &str) -> Option<Named> {
    let lower = name.trim().to_lowercase();
    load_names().into_iter().find(|n| n.name.to_lowercase() == lower)
}

pub fn short(p: &str) -> String {
    if p.len() > 14 && p.is_ascii() {
        format!("{}…{}", &p[..6], &p[p.len() - 4..])
    } else {
        p.to_string()
    }
}

/// Display name for a particle: the name this page knows, or a short hex.
pub fn name_of(particle: &str) -> String {
    load_names()
        .into_iter()
        .find(|n| n.particle == particle)
        .map(|n| n.name)
        .unwrap_or_else(|| short(particle))
}

/// What a typed endpoint resolves to, without storing anything.
#[derive(Clone, Debug, PartialEq)]
pub enum Resolved {
    Empty,
    /// a pasted 64-hex particle
    Raw(String),
    /// a name this page already knows
    Known(Named),
    /// new text: its particle is the hash of the text
    NewText(String),
}

pub fn resolve(text: &str) -> Resolved {
    let t = text.trim();
    if t.is_empty() {
        return Resolved::Empty;
    }
    if is_particle(t) {
        return Resolved::Raw(t.to_lowercase());
    }
    if let Some(n) = find_by_name(t) {
        return Resolved::Known(n);
    }
    Resolved::NewText(particle_of(t.as_bytes()))
}

/// Resolve an endpoint to its particle, remembering new text by name.
pub fn resolve_and_remember(text: &str) -> Option<String> {
    match resolve(text) {
        Resolved::Empty => None,
        Resolved::Raw(p) => Some(p),
        Resolved::Known(n) => Some(n.particle),
        Resolved::NewText(p) => {
            let t = text.trim();
            remember(&p, t, "text", t.len() as u64);
            Some(p)
        }
    }
}

// ─── cyberlink + signal ───────────────────────────────────────────────

/// The five stored fields of a cyberlink.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// from particle (hex)
    pub from: String,
    /// to particle (hex)
    pub to: String,
    /// the box's denomination
    pub coin: String,
    /// the box's quantity, in whole units of the coin; zero is a bare link
    pub amount: u64,
    /// -1, 0, +1
    pub valence: i8,
}

/// A link carried under its author: one leg of a signal.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leg {
    /// bech32 neuron that authors this link
    pub subject: String,
    #[serde(flatten)]
    pub link: Link,
}

/// One subject's signature over the canonical body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seal {
    pub subject: String,
    pub pubkey_hex: String,
    pub sig_hex: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    pub id: String,
    /// draft | committed
    pub state: String,
    pub legs: Vec<Leg>,
    /// hemera hash of the canonical body (hex), set on commit
    #[serde(default)]
    pub body_particle: String,
    /// one signature per subject, set on commit
    #[serde(default)]
    pub seals: Vec<Seal>,
}

impl Signal {
    /// The subjects: the set of authors of the legs, in first-seen order.
    pub fn subjects(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for l in &self.legs {
            if !out.contains(&l.subject) {
                out.push(l.subject.clone());
            }
        }
        out
    }

    /// The total box: amount summed per coin, in first-seen order.
    pub fn total_box(&self) -> Vec<(String, u64)> {
        let mut out: Vec<(String, u64)> = Vec::new();
        for l in &self.legs {
            match out.iter_mut().find(|(c, _)| *c == l.link.coin) {
                Some((_, a)) => *a = a.saturating_add(l.link.amount),
                None => out.push((l.link.coin.clone(), l.link.amount)),
            }
        }
        out
    }
}

pub fn load_signals() -> Vec<Signal> {
    load_json(SIGNALS_KEY)
}

fn save_signals(list: &[Signal]) {
    save_json(SIGNALS_KEY, list);
}

pub fn find_signal(id: &str) -> Option<Signal> {
    load_signals().into_iter().find(|s| s.id == id)
}

pub fn committed_signals() -> Vec<Signal> {
    let mut v: Vec<Signal> = load_signals()
        .into_iter()
        .filter(|s| s.state == "committed")
        .collect();
    v.reverse();
    v
}

/// The one open draft, if any.
pub fn draft() -> Option<Signal> {
    load_signals().into_iter().find(|s| s.state == "draft")
}

/// Add a link to the open draft under the local neuron, opening a draft
/// when there is none. Returns the draft's id.
pub fn draft_link(link: Link) -> String {
    let mut signals = load_signals();
    let leg = Leg {
        subject: neuron().bech32,
        link,
    };
    let id = match signals.iter_mut().find(|s| s.state == "draft") {
        Some(s) => {
            s.legs.push(leg);
            s.id.clone()
        }
        None => {
            let id = next_id(SIGNAL_SEQ_KEY, "sg");
            signals.push(Signal {
                id: id.clone(),
                state: "draft".into(),
                legs: vec![leg],
                body_particle: String::new(),
                seals: Vec::new(),
            });
            id
        }
    };
    save_signals(&signals);
    id
}

pub fn remove_draft_link(index: usize) {
    let mut signals = load_signals();
    if let Some(s) = signals.iter_mut().find(|s| s.state == "draft") {
        if index < s.legs.len() {
            s.legs.remove(index);
        }
    }
    save_signals(&signals);
}

/// Canonical body: one leg per line, `subject|from|to|coin|amount|valence`.
/// This is what gets hashed and signed; it is stable across serde details.
pub fn canonical_body(legs: &[Leg]) -> String {
    legs.iter()
        .map(|l| {
            format!(
                "{}|{}|{}|{}|{}|{}",
                l.subject, l.link.from, l.link.to, l.link.coin, l.link.amount, l.link.valence
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Commit: hash the canonical body (hemera) and sign it (ADR-036) with the
/// local neuron. A draft with a leg authored by another neuron refuses: this
/// page holds one key, and a signal lands for every subject or for none.
pub fn commit_signal(id: &str) -> Result<(), String> {
    let mut signals = load_signals();
    let Some(s) = signals.iter_mut().find(|s| s.id == id) else {
        return Err("signal not found".into());
    };
    if s.state != "draft" {
        return Err("already committed".into());
    }
    if s.legs.is_empty() {
        return Err("empty signal: add at least one link".into());
    }
    let key = domain_key();
    if s.legs.iter().any(|l| l.subject != key.bech32) {
        return Err("a link is authored by another neuron: its signature is missing".into());
    }
    let body = canonical_body(&s.legs);
    s.body_particle = particle_of(body.as_bytes());
    s.seals = vec![Seal {
        subject: key.bech32.clone(),
        pubkey_hex: hex(&key.pubkey),
        sig_hex: hex(&mudra::claim::sign_arbitrary(
            key.signing_key(),
            &key.bech32,
            body.as_bytes(),
        )),
    }];
    s.state = "committed".into();
    save_signals(&signals);
    Ok(())
}

pub fn verify_signal(id: &str) -> Result<(), String> {
    let Some(s) = find_signal(id) else {
        return Err("signal not found".into());
    };
    verify_stored_signal(&s)
}

fn verify_seal(subject: &str, pubkey_hex: &str, sig_hex: &str, body: &[u8]) -> Result<(), String> {
    let pubkey: [u8; 33] = unhex(pubkey_hex)
        .and_then(|v| v.try_into().ok())
        .ok_or("bad pubkey encoding")?;
    match mudra::cosmos::address(&pubkey, HRP) {
        Ok(addr) if addr == subject => {}
        Ok(_) => return Err("neuron does not match pubkey".into()),
        Err(e) => return Err(format!("address: {e}")),
    }
    let sig: [u8; 64] = unhex(sig_hex)
        .and_then(|v| v.try_into().ok())
        .ok_or("bad signature encoding")?;
    if mudra::claim::verify_arbitrary(&pubkey, subject, body, &sig) {
        Ok(())
    } else {
        Err("signature invalid".into())
    }
}

/// Verify a stored signal from its own fields: the body must re-hash to the
/// body particle, and every subject must have a valid signature over it.
pub fn verify_stored_signal(s: &Signal) -> Result<(), String> {
    if s.state != "committed" {
        return Err("not committed".into());
    }
    let body = canonical_body(&s.legs);
    if particle_of(body.as_bytes()) != s.body_particle {
        return Err("body particle mismatch".into());
    }
    for subject in s.subjects() {
        let Some(seal) = s.seals.iter().find(|x| x.subject == subject) else {
            return Err(format!("no signature from {subject}"));
        };
        verify_seal(&seal.subject, &seal.pubkey_hex, &seal.sig_hex, body.as_bytes())?;
    }
    Ok(())
}

// ─── intent ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Intent {
    pub id: String,
    /// declaring neuron, bech32
    pub neuron: String,
    /// the scope's predicate, as written
    pub scope: String,
    /// me | anyone
    pub sealer: String,
    /// the offer's box; amount zero is no offer
    pub offer_coin: String,
    pub offer_amount: u64,
    /// hemera hash of the canonical scope (hex)
    pub scope_particle: String,
    pub pubkey_hex: String,
    /// identity proof: the neuron's signature over neuron and scope particle
    pub sig_hex: String,
    /// open | sealed
    pub state: String,
    /// id of the signal marked as sealing it
    #[serde(default)]
    pub sealed_by: String,
}

pub fn load_intents() -> Vec<Intent> {
    load_json(INTENTS_KEY)
}

fn canonical_scope(scope: &str, sealer: &str, coin: &str, amount: u64) -> String {
    format!("{scope}|{sealer}|{coin}|{amount}")
}

fn intent_message(neuron: &str, scope_particle: &str) -> String {
    // no chain answers here, so the inception height is absent from the proof
    format!("intent|{neuron}|{scope_particle}")
}

/// Declare an intent: hash the scope and sign (neuron, scope particle).
pub fn declare_intent(scope: &str, sealer: &str, coin: &str, amount: u64) -> Result<String, String> {
    let scope = scope.trim();
    if scope.is_empty() {
        return Err("write the scope: what is to be done".into());
    }
    let key = domain_key();
    let scope_particle = particle_of(canonical_scope(scope, sealer, coin, amount).as_bytes());
    let msg = intent_message(&key.bech32, &scope_particle);
    let id = next_id(INTENT_SEQ_KEY, "in");
    let mut intents = load_intents();
    intents.push(Intent {
        id: id.clone(),
        neuron: key.bech32.clone(),
        scope: scope.to_string(),
        sealer: sealer.to_string(),
        offer_coin: coin.to_string(),
        offer_amount: amount,
        scope_particle,
        pubkey_hex: hex(&key.pubkey),
        sig_hex: hex(&mudra::claim::sign_arbitrary(
            key.signing_key(),
            &key.bech32,
            msg.as_bytes(),
        )),
        state: "open".into(),
        sealed_by: String::new(),
    });
    save_json(INTENTS_KEY, &intents);
    Ok(id)
}

/// Re-check an intent's identity proof from its own fields.
pub fn verify_intent(i: &Intent) -> Result<(), String> {
    let scope = canonical_scope(&i.scope, &i.sealer, &i.offer_coin, i.offer_amount);
    if particle_of(scope.as_bytes()) != i.scope_particle {
        return Err("scope particle mismatch".into());
    }
    let msg = intent_message(&i.neuron, &i.scope_particle);
    verify_seal(&i.neuron, &i.pubkey_hex, &i.sig_hex, msg.as_bytes())
}

/// Mark an open intent sealed by a committed signal. A local mark: no proof
/// binds the signal to the scope here, and the page says so.
pub fn seal_intent(intent_id: &str, signal_id: &str) -> Result<(), String> {
    let Some(signal) = find_signal(signal_id) else {
        return Err("signal not found".into());
    };
    if signal.state != "committed" {
        return Err("only a committed signal seals an intent".into());
    }
    let mut intents = load_intents();
    let Some(i) = intents.iter_mut().find(|i| i.id == intent_id) else {
        return Err("intent not found".into());
    };
    if i.state != "open" {
        return Err("an intent seals once".into());
    }
    if i.sealer == "me" && !signal.subjects().contains(&i.neuron) {
        return Err("this intent admits only its declarer as sealer".into());
    }
    i.state = "sealed".into();
    i.sealed_by = signal_id.to_string();
    save_json(INTENTS_KEY, &intents);
    Ok(())
}
