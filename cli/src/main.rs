// ---
// tags: cybergraph, cli, rust
// crystal-type: source
// crystal-domain: cyber
// ---
//! `cybergraph` — the cyberlink processor, on the command line.
//!
//! One layer above the `bbg` state store: it runs the lifecycle verbs
//! (intend / seal / link), keeps the per-neuron signal chains, and answers
//! [[inf]] queries. `link` auto-fills the chain fields (step / prev / network)
//! so the operator supplies only cyberlink content. The store is a tape-frame
//! envelope-signal log, replayed through the processor. See specs/cli.md.

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use foculus::{decode_events, encode_intent_frame, encode_signal_frame, CyberFrame};
use cybergraph::{
    Cybergraph, CyberlinkRecord, Intent, IntentRecord, NeuronId, Particle, Scope, Signal,
};
use inf_value::Value;

const SELF_NETWORK: Particle = [0u8; 32]; // sentinel → the neuron's private network

#[derive(Parser)]
#[command(name = "cybergraph", about = "the cyberlink processor, on the command line")]
struct Cli {
    /// store directory (a tape-frame signal log). absent → ephemeral, in-memory.
    #[arg(long, global = true)]
    store: Option<PathBuf>,
    /// machine-readable output.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// declare an unsealed intent → prints its key.
    Intend {
        #[arg(long)] neuron: String,
        #[arg(long)] target: String,
        #[arg(long, default_value = "")] predicate: String,
        #[arg(long)] deadline: Option<u64>,
        #[arg(long, default_value_t = 0)] h0: u64,
    },
    /// atomic one-shot submit (step/prev/network auto-filled).
    Link {
        #[arg(long)] neuron: String,
        #[arg(long)] from: String,
        #[arg(long)] to: String,
        #[arg(long, default_value = "0")] token: String,
        #[arg(long, default_value_t = 1)] amount: u64,
        #[arg(long, default_value_t = 0)] valence: i8,
    },
    /// seal a declared intent into a signal.
    Seal {
        #[arg(long)] intent: String,
        #[arg(long)] neuron: String,
        #[arg(long)] from: String,
        #[arg(long)] to: String,
        #[arg(long, default_value = "0")] token: String,
        #[arg(long, default_value_t = 1)] amount: u64,
        #[arg(long, default_value_t = 0)] valence: i8,
    },
    /// run an inf datalog query over the graph.
    Query { script: String },
    /// show a neuron's signal chain (step, prev, link count).
    Chain { neuron: String },
    /// list declared (unsealed) intents.
    Intents,
    /// finalize a block: advance root + height.
    Finalize,
    /// print BBG_root (hex).
    Root,
    /// print graph statistics.
    Stats,
}

fn main() {
    let cli = Cli::parse();
    std::process::exit(run(cli));
}

fn run(cli: Cli) -> i32 {
    let mut cg = open_store(cli.store.as_ref());

    match cli.command {
        Command::Intend { neuron, target, predicate, deadline, h0 } => {
            let (n, t) = match (h32(&neuron), h32(&target)) {
                (Some(n), Some(t)) => (n, t),
                _ => return bad("bad hex key"),
            };
            let scope = Scope { target: t, predicate: predicate.into_bytes(), deadline, constraints: vec![] };
            let scope_hash = scope.hash();
            let intent = Intent { neuron: n, h0, scope, signature: [0u8; 64] };
            match cg.intend(intent) {
                Ok(key) => {
                    append(cli.store.as_ref(), &encode_intent_frame(&IntentRecord { neuron: n, h0, scope_hash, signature: [0u8; 64] }));
                    println!("{}", hex(&key));
                }
                Err(e) => return rejected(e),
            }
        }

        Command::Link { neuron, from, to, token, amount, valence } => {
            let Some(signal) = build_signal(&cg, &neuron, &from, &to, &token, amount, valence) else {
                return bad("bad hex key");
            };
            match cg.link(signal.clone()) {
                Ok(()) => {
                    append(cli.store.as_ref(), &encode_signal_frame(&signal));
                    println!("{}", hex(&cg.bbg.state.root()));
                }
                Err(e) => return rejected(e),
            }
        }

        Command::Seal { intent, neuron, from, to, token, amount, valence } => {
            let Some(key) = h32(&intent) else { return bad("bad intent key") };
            let Some(signal) = build_signal(&cg, &neuron, &from, &to, &token, amount, valence) else {
                return bad("bad hex key");
            };
            match cg.seal(key, signal.clone()) {
                Ok(()) => {
                    append(cli.store.as_ref(), &encode_signal_frame(&signal));
                    println!("{}", hex(&cg.bbg.state.root()));
                }
                Err(e) => return rejected(e),
            }
        }

        Command::Query { script } => return cmd_query(&cg, &script, cli.json),
        Command::Chain { neuron } => return cmd_chain(&cg, &neuron),
        Command::Intents => cmd_intents(&cg),

        Command::Finalize => {
            cg.bbg.finalize_block();
            bump_blocks(cli.store.as_ref());
            println!("height {}  root {}", cg.bbg.state.height, hex(&cg.bbg.state.root()));
        }

        Command::Root => println!("{}", hex(&cg.bbg.state.root())),
        Command::Stats => {
            let s = cg.bbg.statistics();
            println!("node_count      {}", s.node_count);
            println!("max_degree      {}", s.max_degree);
            println!("diameter_bound  {}", s.diameter_bound);
            println!("relation_sizes  {:?}", s.relation_sizes);
        }
    }
    0
}

// ── build a signal, auto-filling the chain fields ────────────────────────────

fn build_signal(cg: &Cybergraph, neuron: &str, from: &str, to: &str, token: &str, amount: u64, valence: i8) -> Option<Signal> {
    let (n, f, t, tok) = (h32(neuron)?, h32(from)?, h32(to)?, h32(token)?);
    let (step, prev) = next_pos(cg, &n);
    Some(Signal {
        neuron: n,
        network: SELF_NETWORK,
        links: vec![CyberlinkRecord { neuron: n, from: f, to: t, token: tok, amount, valence, height: 0 }],
        delta_pi: vec![],
        prev,
        step,
        height: 0,
        proof: None,
    })
}

/// The next chain position for a neuron: step = chain length, prev = head hash.
fn next_pos(cg: &Cybergraph, neuron: &NeuronId) -> (u64, Particle) {
    match cg.chains.get(neuron) {
        Some(chain) if !chain.entries.is_empty() => {
            let step = chain.entries.len() as u64;
            let prev = chain.entries[&(step - 1)].hash();
            (step, prev)
        }
        _ => (0, [0u8; 32]),
    }
}

// ── store: a tape-frame log replayed through the processor ───────────────────

fn open_store(store: Option<&PathBuf>) -> Cybergraph {
    let mut cg = Cybergraph::new();
    let Some(dir) = store else { return cg };
    if let Ok(bytes) = std::fs::read(dir.join("log")) {
        for frame in decode_events(&bytes) {
            match frame {
                CyberFrame::Signal(s) => { let _ = cg.link(s); }
                CyberFrame::Intent(i) => { cg.bbg.apply_intent(&i); }
            }
        }
    }
    // finalize count is tracked separately (see specs/cli.md: signals then blocks).
    if let Ok(text) = std::fs::read_to_string(dir.join("blocks")) {
        if let Ok(n) = text.trim().parse::<u64>() {
            for _ in 0..n { cg.bbg.finalize_block(); }
        }
    }
    cg
}

fn append(store: Option<&PathBuf>, frame: &[u8]) {
    let Some(dir) = store else { return };
    if let Err(e) = std::fs::create_dir_all(dir) { eprintln!("store write error: {e}"); return; }
    use std::io::Write;
    match std::fs::OpenOptions::new().create(true).append(true).open(dir.join("log")) {
        Ok(mut f) => { let _ = f.write_all(frame); }
        Err(e) => eprintln!("store write error: {e}"),
    }
}

fn bump_blocks(store: Option<&PathBuf>) {
    let Some(dir) = store else { return };
    let path = dir.join("blocks");
    let n = std::fs::read_to_string(&path).ok().and_then(|t| t.trim().parse::<u64>().ok()).unwrap_or(0);
    let _ = std::fs::write(path, (n + 1).to_string());
}

// ── query ────────────────────────────────────────────────────────────────────

fn cmd_query(cg: &Cybergraph, script: &str, json: bool) -> i32 {
    let out = match cg.query(script) {
        Ok(o) => o,
        Err(e) => { eprintln!("query error: {e:?}"); return 3; }
    };
    if json {
        println!("{{\"columns\":{:?},\"rows\":{}}}", out.columns, out.rows.len());
    } else if out.columns.is_empty() && out.rows.is_empty() {
        println!("(no rows)");
    } else {
        println!("{}", out.columns.join("\t"));
        for row in &out.rows {
            let cells: Vec<String> = row.iter().map(fmt_value).collect();
            println!("{}", cells.join("\t"));
        }
    }
    0
}

fn fmt_value(v: &Value) -> String {
    match v {
        Value::Null => "null".into(),
        Value::Bool(b) => b.to_string(),
        Value::Int(i) => i.to_string(),
        Value::Field(f) => format!("{f:?}"),
        Value::Word(w) => w.to_string(),
        Value::Hash(h) => hex(h),
        Value::Bytes(b) => hex(b),
        Value::List(l) => format!("[{}]", l.iter().map(fmt_value).collect::<Vec<_>>().join(",")),
    }
}

// ── chain / intents ──────────────────────────────────────────────────────────

fn cmd_chain(cg: &Cybergraph, neuron: &str) -> i32 {
    let Some(n) = h32(neuron) else { return bad("bad hex key") };
    let Some(chain) = cg.chains.get(&n) else { eprintln!("no chain for neuron"); return 1 };
    for (step, s) in &chain.entries {
        println!("step {}  prev {}  links {}", step, hex(&s.prev), s.links.len());
    }
    0
}

fn cmd_intents(cg: &Cybergraph) {
    for (key, r) in &cg.bbg.state.intents {
        println!("intent {}  neuron {}  h0 {}", hex(key), hex(&r.neuron), r.h0);
    }
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn bad(msg: &str) -> i32 { eprintln!("{msg}"); 3 }
fn rejected<E: std::fmt::Debug>(e: E) -> i32 { eprintln!("rejected: {e:?}"); 2 }

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Parse a hex key into 32 bytes. Optional `0x`, left-padded with zeros. Pure hex.
fn h32(s: &str) -> Option<[u8; 32]> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    if s.is_empty() || s.len() > 64 || !s.chars().all(|c| c.is_ascii_hexdigit()) { return None; }
    let padded = format!("{s:0>64}");
    let mut out = [0u8; 32];
    for i in 0..32 {
        out[i] = u8::from_str_radix(&padded[i * 2..i * 2 + 2], 16).ok()?;
    }
    Some(out)
}
