//! The pages: the composer with its draft signal, the signal page, the
//! signals list, the intents. Storage is read through one tick signal:
//! every write bumps it and the views re-read.

use crate::signal::{
    committed_signals, commit_signal, declare_intent, draft, draft_link, find_by_name,
    find_signal, load_intents, load_names, name_of, neuron, particle_of, remember,
    remove_draft_link, resolve, resolve_and_remember, seal_intent, short, verify_intent,
    verify_signal, Leg, Link, Resolved, Signal, DEFAULT_COIN,
};
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::hooks::{use_location, use_navigate, use_params_map};
use leptos_router::path;

/// The path the app is served under, fixed at build time (`STUDIO_BASE`).
const BASE: &str = match option_env!("STUDIO_BASE") {
    Some(b) => b,
    None => "/",
};

fn base() -> &'static str {
    BASE.trim_end_matches('/')
}

fn url(path: &str) -> String {
    format!("{}{}", base(), path)
}

/// Bumped after every storage write; views that read storage track it.
#[derive(Clone, Copy)]
struct Tick(RwSignal<u32>);

fn tick() -> Tick {
    expect_context::<Tick>()
}

impl Tick {
    fn track(&self) {
        self.0.track();
    }
    fn bump(&self) {
        self.0.update(|n| *n = n.wrapping_add(1));
    }
}

fn valence_text(v: i8) -> &'static str {
    match v {
        1 => "+1",
        -1 => "−1",
        _ => "0",
    }
}

fn valence_class(v: i8) -> &'static str {
    match v {
        1 => "val val-pos",
        -1 => "val val-neg",
        _ => "val",
    }
}

fn box_text(total: &[(String, u64)]) -> String {
    if total.is_empty() {
        return "0".into();
    }
    total
        .iter()
        .map(|(c, a)| format!("{a} {c}"))
        .collect::<Vec<_>>()
        .join(" + ")
}

#[component]
pub fn App() -> impl IntoView {
    provide_context(Tick(RwSignal::new(0)));
    // the neuron is a particle too: "me" names it, so a link can start at it
    let me = neuron();
    remember(&me.native_hex, "me", "neuron", 0);
    view! {
        <Router base=base()>
            <Header/>
            <main class="page">
                <Routes fallback=|| view! { <p class="empty">"nothing lives at this address. " <a href=url("/")>"compose a link"</a></p> }>
                    <Route path=path!("/") view=ComposePage/>
                    <Route path=path!("/signal/:id") view=SignalPage/>
                    <Route path=path!("/signals") view=SignalsPage/>
                    <Route path=path!("/intents") view=IntentsPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Header() -> impl IntoView {
    let me = neuron();
    let bech32 = me.bech32.clone();
    let copied = RwSignal::new(false);
    let copy = move |_| {
        if let Some(w) = web_sys::window() {
            let _ = w.navigator().clipboard().write_text(&bech32);
        }
        copied.set(true);
        set_timeout(move || copied.set(false), std::time::Duration::from_millis(1200));
    };
    let path = use_location().pathname;
    let tab = move |href: &'static str, label: &'static str| {
        let full = url(href);
        let full2 = full.clone();
        view! {
            <a href=full
                class=move || {
                    let p = path.get();
                    let on = p.trim_end_matches('/') == full2.trim_end_matches('/')
                        || (href == "/signals" && p.contains("/signal/"));
                    if on { "tab tab-on" } else { "tab" }
                }>{label}</a>
        }
    };
    view! {
        <header class="top">
            <div class="top-row">
                <a class="brand" href=url("/")>"studio"</a>
                <nav class="tabs">
                    {tab("/", "compose")}
                    {tab("/signals", "signals")}
                    {tab("/intents", "intents")}
                </nav>
            </div>
            <button class="neuron" id="neuron" title="your neuron: click to copy" on:click=copy>
                <span class="neuron-lab">{move || if copied.get() { "copied" } else { "neuron" }}</span>
                <span class="neuron-id">{me.bech32.clone()}</span>
            </button>
        </header>
    }
}

// ─── the composer ─────────────────────────────────────────────────────

/// Hash a chosen or dropped file in the page and put its name in the field.
fn take_file(file: web_sys::File, value: RwSignal<String>, busy: RwSignal<bool>) {
    busy.set(true);
    let t = tick();
    leptos::task::spawn_local(async move {
        let name = file.name();
        if let Ok(buf) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
            let bytes = js_sys::Uint8Array::new(&buf).to_vec();
            let particle = particle_of(&bytes);
            // a name already taken by other content gets the particle beside it
            let name = match find_by_name(&name) {
                Some(n) if n.particle != particle => format!("{name} · {}", short(&particle)),
                _ => name,
            };
            remember(&particle, &name, "file", bytes.len() as u64);
            let shown = load_names()
                .into_iter()
                .find(|n| n.particle == particle)
                .map(|n| n.name)
                .unwrap_or(particle);
            value.set(shown);
            t.bump();
        }
        busy.set(false);
    });
}

#[component]
fn Endpoint(
    label: &'static str,
    id: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] offer_me: bool,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let over = RwSignal::new(false);
    let t = tick();
    let hint = move || {
        t.track();
        if busy.get() {
            return "computing the particle…".to_string();
        }
        match resolve(&value.get()) {
            Resolved::Empty => "type a name, paste a particle, or drop a file".to_string(),
            Resolved::Raw(p) => format!("particle {}", short(&p)),
            Resolved::Known(n) => match n.kind.as_str() {
                "file" => format!("file, {} bytes · particle {}", n.size, short(&n.particle)),
                "neuron" => format!("your neuron · particle {}", short(&n.particle)),
                _ => format!("known text · particle {}", short(&n.particle)),
            },
            Resolved::NewText(p) => format!("new text · particle {}", short(&p)),
        }
    };
    let file_id = format!("{id}-file");
    view! {
        <div
            class=move || if over.get() { "field field-over" } else { "field" }
            on:dragover=move |ev| { ev.prevent_default(); over.set(true); }
            on:dragleave=move |_| over.set(false)
            on:drop=move |ev| {
                ev.prevent_default();
                over.set(false);
                if let Some(f) = ev.data_transfer().and_then(|d| d.files()).and_then(|l| l.get(0)) {
                    take_file(f, value, busy);
                }
            }
        >
            <label class="lab" for=id>{label}</label>
            <div class="field-row">
                <input class="inp" id=id type="text" list="known" autocomplete="off" spellcheck="false"
                    placeholder="name, particle or file"
                    prop:value=move || value.get()
                    on:input=move |ev| value.set(event_target_value(&ev)) />
                {offer_me.then(|| view! {
                    <button class="btn-s" type="button" title="start the link at your neuron"
                        on:click=move |_| value.set("me".into())>"me"</button>
                })}
                <label class="btn-s" for=file_id.clone() title="choose a file: its particle is computed in this page">"file"</label>
                <input class="hidden" id=file_id type="file"
                    on:change=move |ev| {
                        let input: web_sys::HtmlInputElement = event_target(&ev);
                        if let Some(f) = input.files().and_then(|l| l.get(0)) {
                            take_file(f, value, busy);
                        }
                        input.set_value("");
                    } />
            </div>
            <div class="hint" id=format!("{id}-hint")>{hint}</div>
        </div>
    }
}

#[component]
fn ComposePage() -> impl IntoView {
    let t = tick();
    let from = RwSignal::new(String::new());
    let to = RwSignal::new(String::new());
    let coin = RwSignal::new(DEFAULT_COIN.to_string());
    let amount = RwSignal::new("0".to_string());
    let valence = RwSignal::new(0i8);
    let err = RwSignal::new(None::<String>);

    let add = move |_| {
        let amt = amount.get_untracked();
        let amt = amt.trim();
        let Ok(a) = (if amt.is_empty() { Ok(0) } else { amt.parse::<u64>() }) else {
            err.set(Some("amount is a whole number, zero or more".into()));
            return;
        };
        let c = coin.get_untracked().trim().to_uppercase();
        if c.is_empty() || c.contains('|') {
            err.set(Some("name the coin of the box, like CYB".into()));
            return;
        }
        let (Some(f), Some(to_p)) = (
            resolve_and_remember(&from.get_untracked()),
            resolve_and_remember(&to.get_untracked()),
        ) else {
            err.set(Some("a link joins two particles: fill from and to".into()));
            return;
        };
        draft_link(Link {
            from: f,
            to: to_p,
            coin: c,
            amount: a,
            valence: valence.get_untracked(),
        });
        err.set(None);
        from.set(String::new());
        to.set(String::new());
        amount.set("0".into());
        valence.set(0);
        t.bump();
    };

    let val_btn = move |v: i8, what: &'static str| {
        view! {
            <button type="button" id=format!("val{}", match v { 1 => "p", -1 => "n", _ => "0" })
                class=move || if valence.get() == v { "seg seg-on" } else { "seg" }
                on:click=move |_| valence.set(v)>
                <span class="seg-v">{valence_text(v)}</span>
                <span class="seg-w">{what}</span>
            </button>
        }
    };

    view! {
        <datalist id="known">
            {move || {
                t.track();
                load_names().into_iter().map(|n| view! {
                    <option value=n.name.clone() label=format!("{} · {}", n.kind, short(&n.particle))></option>
                }).collect_view()
            }}
        </datalist>
        <div class="cols">
            <section class="panel" id="composer">
                <h1 class="h">"cyberlink"</h1>
                <p class="lead">"link two particles, move a box between them, say what you predict."</p>
                <Endpoint label="from" id="from" value=from offer_me=true/>
                <Endpoint label="to" id="to" value=to/>
                <div class="field">
                    <span class="lab">"box"</span>
                    <div class="field-row">
                        <input class="inp inp-amount" id="amount" type="text" inputmode="numeric" autocomplete="off"
                            aria-label="amount"
                            prop:value=move || amount.get()
                            on:input=move |ev| amount.set(event_target_value(&ev)) />
                        <input class="inp inp-coin" id="coin" type="text" autocomplete="off" spellcheck="false"
                            aria-label="coin"
                            prop:value=move || coin.get()
                            on:input=move |ev| coin.set(event_target_value(&ev)) />
                    </div>
                    <div class="hint">"the conviction moved: amount and coin. zero is a bare link."</div>
                </div>
                <div class="field">
                    <span class="lab">"valence"</span>
                    <div class="segs">
                        {val_btn(-1, "it will be rejected")}
                        {val_btn(0, "no prediction")}
                        {val_btn(1, "it will be confirmed")}
                    </div>
                </div>
                {move || err.get().map(|e| view! { <div class="msg msg-err" id="compose-err">{e}</div> })}
                <button class="btn btn-main" id="add" type="button" on:click=add>"add to the signal"</button>
            </section>
            <DraftPanel/>
        </div>
    }
}

fn leg_view(leg: Leg, me: &str, remove: Option<usize>) -> impl IntoView {
    let t = tick();
    let who = if leg.subject == me {
        "me".to_string()
    } else {
        short(&leg.subject)
    };
    let l = leg.link;
    view! {
        <div class="leg">
            <div class="leg-body">
                <div class="leg-link">
                    <span class="pt" title=l.from.clone()>{name_of(&l.from)}</span>
                    <span class="arrow">"→"</span>
                    <span class="pt" title=l.to.clone()>{name_of(&l.to)}</span>
                </div>
                <div class="leg-meta">
                    <span>{format!("box {} {}", l.amount, l.coin)}</span>
                    <span class=valence_class(l.valence)>{format!("valence {}", valence_text(l.valence))}</span>
                    <span title=leg.subject.clone()>{format!("subject {who}")}</span>
                </div>
            </div>
            {remove.map(|i| view! {
                <button class="btn-x" type="button" title="remove this link" aria-label="remove this link"
                    on:click=move |_| { remove_draft_link(i); t.bump(); }>"×"</button>
            })}
        </div>
    }
}

#[component]
fn DraftPanel() -> impl IntoView {
    let t = tick();
    let err = RwSignal::new(None::<String>);
    let me = neuron().bech32;
    let navigate = use_navigate();
    view! {
        <section class="panel" id="draft">
            {move || {
                t.track();
                let d = draft().filter(|d| !d.legs.is_empty());
                let Some(d) = d else {
                    return view! {
                        <h2 class="h">"draft signal"</h2>
                        <p class="empty">"no links yet. add one and it waits here until you commit."</p>
                    }.into_any();
                };
                let n = d.legs.len();
                let id = d.id.clone();
                let total = box_text(&d.total_box());
                let navigate = navigate.clone();
                let commit = move |_| match commit_signal(&id) {
                    Ok(()) => {
                        t.bump();
                        navigate(&format!("/signal/{id}"), Default::default());
                    }
                    Err(e) => err.set(Some(e)),
                };
                view! {
                    <h2 class="h">"draft signal " <span class="dim">{d.id.clone()}</span></h2>
                    <p class="local">"local: kept in this browser, submitted to no chain"</p>
                    <div class="legs">
                        {d.legs.into_iter().enumerate().map(|(i, leg)| leg_view(leg, &me, Some(i))).collect_view()}
                    </div>
                    <div class="sum">
                        <span>{format!("{n} {}", if n == 1 { "link" } else { "links" })}</span>
                        <span id="total-box">{format!("total box {total}")}</span>
                    </div>
                    <div class="cosign">
                        <span>"subjects: me"</span>
                        <button class="btn-s" type="button" disabled=true>"add co-signer"</button>
                        <span class="dim">"needs a second neuron — next change"</span>
                    </div>
                    {move || err.get().map(|e| view! { <div class="msg msg-err">{e}</div> })}
                    <button class="btn btn-main" id="commit" type="button" on:click=commit>"commit: sign with my neuron"</button>
                }.into_any()
            }}
        </section>
    }
}

// ─── signals ──────────────────────────────────────────────────────────

fn signal_row(s: &Signal) -> impl IntoView {
    let n = s.legs.len();
    let first = s
        .legs
        .first()
        .map(|l| format!("{} → {}", name_of(&l.link.from), name_of(&l.link.to)))
        .unwrap_or_default();
    view! {
        <a class="row" href=url(&format!("/signal/{}", s.id))>
            <span class="row-id">{s.id.clone()}</span>
            <span class="row-main">{if n > 1 { format!("{first} and {} more", n - 1) } else { first }}</span>
            <span class="row-side">{format!("box {}", box_text(&s.total_box()))}</span>
        </a>
    }
}

#[component]
fn SignalsPage() -> impl IntoView {
    let t = tick();
    view! {
        <section class="panel">
            <h1 class="h">"signals"</h1>
            <p class="local">"local: signed in this browser, submitted to no chain"</p>
            {move || {
                t.track();
                let list = committed_signals();
                if list.is_empty() {
                    return view! { <p class="empty">"no committed signals yet. " <a href=url("/")>"compose a link"</a></p> }.into_any();
                }
                view! { <div class="rows">{list.iter().map(signal_row).collect_view()}</div> }.into_any()
            }}
        </section>
    }
}

#[component]
fn SignalPage() -> impl IntoView {
    let params = use_params_map();
    let verdict = RwSignal::new(None::<Result<(), String>>);
    let me = neuron().bech32;
    view! {
        {move || {
            let id = params.get().get("id").unwrap_or_default();
            let Some(s) = find_signal(&id) else {
                return view! {
                    <section class="panel">
                        <h1 class="h">"signal not found"</h1>
                        <p class="empty">"this browser holds no signal " {id} ". " <a href=url("/signals")>"all signals"</a></p>
                    </section>
                }.into_any();
            };
            if s.state != "committed" {
                return view! {
                    <section class="panel">
                        <h1 class="h">"signal " {s.id.clone()}</h1>
                        <p class="empty">"still a draft. " <a href=url("/")>"open the composer to commit it"</a></p>
                    </section>
                }.into_any();
            }
            let subjects = s.subjects();
            let total = box_text(&s.total_box());
            let id2 = s.id.clone();
            view! {
                <section class="panel" id="signal">
                    <h1 class="h">"signal " <span class="dim" id="signal-id">{s.id.clone()}</span></h1>
                    <p class="local">"local: signed in this browser, submitted to no chain"</p>
                    <div class="legs">
                        {s.legs.iter().cloned().map(|leg| leg_view(leg, &me, None)).collect_view()}
                    </div>
                    <div class="sum">
                        <span>{format!("{} {}", s.legs.len(), if s.legs.len() == 1 { "link" } else { "links" })}</span>
                        <span>{format!("total box {total}")}</span>
                    </div>
                    <dl class="facts">
                        <dt>{if subjects.len() == 1 { "subject" } else { "subjects" }}</dt>
                        <dd>{subjects.join(", ")}</dd>
                        <dt>"body particle"</dt>
                        <dd id="body-particle">{s.body_particle.clone()}</dd>
                        {s.seals.iter().map(|seal| view! {
                            <dt>"public key"</dt>
                            <dd>{seal.pubkey_hex.clone()}</dd>
                            <dt>"signature"</dt>
                            <dd>{seal.sig_hex.clone()}</dd>
                        }).collect_view()}
                        <dt>"proof"</dt>
                        <dd class="dim">"none: a zheng proof is made when a chain seals the signal"</dd>
                        <dt>"receipt"</dt>
                        <dd class="dim">"none: no chain has accepted it"</dd>
                    </dl>
                    <div class="actions">
                        <button class="btn" id="verify" type="button"
                            on:click=move |_| verdict.set(Some(verify_signal(&id2)))>"re-verify"</button>
                        {move || verdict.get().map(|v| match v {
                            Ok(()) => view! { <span class="msg msg-ok" id="verdict">"valid: the body hashes to its particle and the signature is the subject's"</span> }.into_any(),
                            Err(e) => view! { <span class="msg msg-err" id="verdict">{format!("invalid: {e}")}</span> }.into_any(),
                        })}
                    </div>
                    <div class="actions">
                        <a class="btn-s" href=url("/")>"compose another"</a>
                        <a class="btn-s" href=url("/signals")>"all signals"</a>
                    </div>
                </section>
            }.into_any()
        }}
    }
}

// ─── intents ──────────────────────────────────────────────────────────

#[component]
fn IntentsPage() -> impl IntoView {
    let t = tick();
    let scope = RwSignal::new(String::new());
    let sealer = RwSignal::new("me".to_string());
    let coin = RwSignal::new(DEFAULT_COIN.to_string());
    let amount = RwSignal::new("0".to_string());
    let err = RwSignal::new(None::<String>);

    let declare = move |_| {
        let amt = amount.get_untracked();
        let amt = amt.trim();
        let Ok(a) = (if amt.is_empty() { Ok(0) } else { amt.parse::<u64>() }) else {
            err.set(Some("the offer is a whole number, zero or more".into()));
            return;
        };
        let c = coin.get_untracked().trim().to_uppercase();
        let c = if c.is_empty() { DEFAULT_COIN.to_string() } else { c };
        match declare_intent(&scope.get_untracked(), &sealer.get_untracked(), &c, a) {
            Ok(_) => {
                err.set(None);
                scope.set(String::new());
                amount.set("0".into());
                t.bump();
            }
            Err(e) => err.set(Some(e)),
        }
    };

    let sealer_btn = move |v: &'static str, label: &'static str, what: &'static str| {
        view! {
            <button type="button" id=format!("sealer-{v}")
                class=move || if sealer.get() == v { "seg seg-on" } else { "seg" }
                on:click=move |_| sealer.set(v.into())>
                <span class="seg-v">{label}</span>
                <span class="seg-w">{what}</span>
            </button>
        }
    };

    view! {
        <div class="cols">
            <section class="panel" id="intent-form">
                <h1 class="h">"intent"</h1>
                <p class="lead">"declare an action before it is done: a scope, who may seal it, what you offer."</p>
                <div class="field">
                    <label class="lab" for="scope">"scope"</label>
                    <input class="inp" id="scope" type="text" autocomplete="off"
                        placeholder="what is to be done"
                        prop:value=move || scope.get()
                        on:input=move |ev| scope.set(event_target_value(&ev)) />
                </div>
                <div class="field">
                    <span class="lab">"sealer"</span>
                    <div class="segs segs-2">
                        {sealer_btn("me", "me", "a promise I seal")}
                        {sealer_btn("anyone", "anyone", "a demand any neuron fills")}
                    </div>
                </div>
                <div class="field">
                    <span class="lab">"offer"</span>
                    <div class="field-row">
                        <input class="inp inp-amount" id="offer" type="text" inputmode="numeric" autocomplete="off"
                            aria-label="offer amount"
                            prop:value=move || amount.get()
                            on:input=move |ev| amount.set(event_target_value(&ev)) />
                        <input class="inp inp-coin" id="offer-coin" type="text" autocomplete="off" spellcheck="false"
                            aria-label="offer coin"
                            prop:value=move || coin.get()
                            on:input=move |ev| coin.set(event_target_value(&ev)) />
                    </div>
                    <div class="hint">"the box the filler receives on sealing. zero is no offer."</div>
                </div>
                {move || err.get().map(|e| view! { <div class="msg msg-err">{e}</div> })}
                <button class="btn btn-main" id="declare" type="button" on:click=declare>"declare: sign with my neuron"</button>
            </section>
            <section class="panel" id="intents">
                <h2 class="h">"intents"</h2>
                <p class="local">"local: signed in this browser, declared on no chain"</p>
                {move || {
                    t.track();
                    let mut list = load_intents();
                    list.reverse();
                    if list.is_empty() {
                        return view! { <p class="empty">"no intents yet."</p> }.into_any();
                    }
                    let signals = committed_signals();
                    view! {
                        <div class="legs">
                            {list.into_iter().map(|i| intent_view(i, &signals)).collect_view()}
                        </div>
                    }.into_any()
                }}
            </section>
        </div>
    }
}

fn intent_view(i: crate::signal::Intent, signals: &[Signal]) -> impl IntoView {
    let t = tick();
    let open = i.state == "open";
    let proof = match verify_intent(&i) {
        Ok(()) => "identity proof valid".to_string(),
        Err(e) => format!("identity proof invalid: {e}"),
    };
    let offer = if i.offer_amount == 0 {
        "no offer".to_string()
    } else {
        format!("offer {} {}", i.offer_amount, i.offer_coin)
    };
    let chosen = RwSignal::new(signals.first().map(|s| s.id.clone()).unwrap_or_default());
    let err = RwSignal::new(None::<String>);
    let iid = i.id.clone();
    let seal = move |_| match seal_intent(&iid, &chosen.get_untracked()) {
        Ok(()) => t.bump(),
        Err(e) => err.set(Some(e)),
    };
    let options = signals
        .iter()
        .map(|s| view! { <option value=s.id.clone()>{s.id.clone()}</option> })
        .collect_view();
    let has_signals = !signals.is_empty();
    view! {
        <div class="intent">
            <div class="leg-link">
                <span class="dim">{i.id.clone()}</span>
                <span class=if open { "state state-open" } else { "state" }>{i.state.clone()}</span>
            </div>
            <div class="intent-scope">{i.scope.clone()}</div>
            <div class="leg-meta">
                <span>{format!("sealer {}", i.sealer)}</span>
                <span>{offer}</span>
                <span title=i.scope_particle.clone()>{format!("scope {}", short(&i.scope_particle))}</span>
                <span>{proof}</span>
            </div>
            {if open {
                if has_signals {
                    view! {
                        <div class="seal">
                            <select class="inp inp-sel" aria-label="signal that seals it"
                                on:change=move |ev| chosen.set(event_target_value(&ev))>{options}</select>
                            <button class="btn-s" type="button" on:click=seal>"mark sealed"</button>
                        </div>
                        <div class="hint">"a local mark: no proof binds the signal to the scope yet."</div>
                        {move || err.get().map(|e| view! { <div class="msg msg-err">{e}</div> })}
                    }.into_any()
                } else {
                    view! { <div class="hint">"open. commit a signal to have something that seals it."</div> }.into_any()
                }
            } else {
                view! {
                    <div class="hint">"sealed by " <a href=url(&format!("/signal/{}", i.sealed_by))>{i.sealed_by.clone()}</a> " — a local mark."</div>
                }.into_any()
            }}
        </div>
    }
}
