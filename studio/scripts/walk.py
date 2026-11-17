#!/usr/bin/env python3
"""Walk the studio in a headless browser: two links, commit, re-verify,
reload, a tampered signal, the lists, an intent, a file's particle.

    python3 scripts/walk.py URL OUTDIR [WxH] [mobile]

    python3 scripts/walk.py http://127.0.0.1:8791/pr/studio/ /tmp/studio-check
    python3 scripts/walk.py http://127.0.0.1:8791/pr/studio/ /tmp/studio-check 390x844 mobile

Screenshots land in OUTDIR. The browser driver is cyber/scripts/cdp.py, found
beside this repository or through $CDP_PY_DIR. Exit status 0 is green.
"""
import base64, json, os, sys, urllib.request
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.environ.get("CDP_PY_DIR") or os.path.join(HERE, "..", "..", "..", "cyber", "scripts"))
import cdp

url, out = sys.argv[1], sys.argv[2]
size = sys.argv[3] if len(sys.argv) > 3 else "1440x900"
mobile = len(sys.argv) > 4
w, h = (int(v) for v in size.split("x"))
tag = "phone" if mobile else "desk"
proc, profile, port = cdp.launch(cdp.find_browser(), w, h)
fails = []
os.makedirs(out, exist_ok=True)

def check(name, cond, detail=""):
    print(("ok   " if cond else "FAIL ") + name + (f"  [{detail}]" if detail else ""))
    if not cond:
        fails.append(name)

try:
    req = urllib.request.Request(f"http://127.0.0.1:{port}/json/new?about:blank", method="PUT")
    page = cdp.Page(json.load(urllib.request.urlopen(req, timeout=10))["webSocketDebuggerUrl"])
    for d in ("Page", "Runtime", "Log"):
        page.call(f"{d}.enable")
    page.call("Emulation.setDeviceMetricsOverride", {"width": w, "height": h, "deviceScaleFactor": 2 if mobile else 1, "mobile": mobile})
    if mobile:
        page.call("Emulation.setTouchEmulationEnabled", {"enabled": True, "maxTouchPoints": 5})

    def load(u=None):
        page.events.clear()
        if u:
            page.call("Page.navigate", {"url": u})
        else:
            page.call("Page.reload")
        page.pump(30.0, until="Page.loadEventFired")
        page.pump(1.5)

    def shot(name, full=True):
        r = page.call("Page.captureScreenshot", {"format": "png", "captureBeyondViewport": full})
        open(f"{out}/{tag}-{name}.png", "wb").write(base64.b64decode(r["data"]))

    PRE = """const sleep = ms => new Promise(r => setTimeout(r, ms));
const set = (id, v) => { const e = document.getElementById(id); e.value = v; e.dispatchEvent(new Event('input', {bubbles: true})); };
const click = id => document.getElementById(id).click();
const noscroll = () => document.documentElement.scrollWidth <= innerWidth;
"""
    def ev(js):
        v = page.evaluate(PRE + js)
        if isinstance(v, dict) and "error" in v:
            fails.append("js: " + str(v["error"])[:300]); print("FAIL js", v["error"])
            return {}
        return json.loads(v)

    load(url)
    r = ev("return JSON.stringify({neuron: document.querySelector('.neuron-id').textContent, tabs: document.querySelectorAll('.tab').length, ns: noscroll(), empty: document.querySelector('#draft .empty')?.textContent})")
    check("header shows the neuron", r.get("neuron", "").startswith("cyber1"), r.get("neuron"))
    check("compose: no horizontal scroll", r.get("ns"))
    shot("1-compose-empty")

    r = ev("""document.querySelector('#composer .btn-s').click(); await sleep(50);
set('to', 'the cybergraph'); set('amount', '5'); click('valp'); await sleep(50);
const fromHint = document.getElementById('from-hint').textContent, toHint = document.getElementById('to-hint').textContent;
return JSON.stringify({fromHint, toHint})""")
    check("from = me resolves to the neuron", "your neuron" in r.get("fromHint", ""), r.get("fromHint"))
    check("to shows the particle of new text", "new text" in r.get("toHint", ""), r.get("toHint"))
    shot("2-compose-filled")
    r = ev("""click('add'); await sleep(150);
set('from', 'the cybergraph'); set('to', 'a'.repeat(64)); click('valn'); await sleep(50); click('add'); await sleep(150);
return JSON.stringify({legs: document.querySelectorAll('#draft .leg').length, total: document.getElementById('total-box').textContent,
 cos: document.querySelector('#draft .cosign').textContent, dis: document.querySelector('#draft .cosign button').disabled,
 local: document.querySelector('#draft .local').textContent, ns: noscroll(), legtext: [...document.querySelectorAll('#draft .leg')].map(e => e.textContent)})""")
    check("two links in the draft", r.get("legs") == 2, str(r.get("legtext")))
    check("total box", r.get("total") == "total box 5 CYB", r.get("total"))
    check("co-signer is disabled and says why", r.get("dis") and "needs a second neuron — next change" in r.get("cos", ""), r.get("cos"))
    check("draft says it is local", "local" in r.get("local", ""), r.get("local"))
    check("draft: no horizontal scroll", r.get("ns"))
    shot("3-draft")

    # remove and re-add proves remove works
    r = ev("""document.querySelectorAll('#draft .btn-x')[1].click(); await sleep(150);
const after = document.querySelectorAll('#draft .leg').length;
set('from', 'the cybergraph'); set('to', 'a'.repeat(64)); await sleep(50); click('add'); await sleep(150);
return JSON.stringify({after, again: document.querySelectorAll('#draft .leg').length})""")
    check("remove a link", r.get("after") == 1 and r.get("again") == 2, str(r))

    r = ev("""click('commit'); await sleep(600); click('verify'); await sleep(300);
return JSON.stringify({path: location.pathname, id: document.getElementById('signal-id')?.textContent, legs: document.querySelectorAll('#signal .leg').length,
 body: document.getElementById('body-particle')?.textContent, verdict: document.getElementById('verdict')?.textContent, ns: noscroll(),
 local: document.querySelector('#signal .local')?.textContent})""")
    sig_path = r.get("path", "")
    check("commit lands on the signal page", sig_path.endswith("/signal/sg-1"), sig_path)
    check("signal keeps both links", r.get("legs") == 2)
    check("body particle is 64 hex", len(r.get("body") or "") == 64, r.get("body"))
    check("signal re-verifies", (r.get("verdict") or "").startswith("valid"), r.get("verdict"))
    check("signal says it is local", "no chain" in (r.get("local") or ""), r.get("local"))
    check("signal: no horizontal scroll", r.get("ns"))
    shot("4-signal")

    load()
    r = ev("""click('verify'); await sleep(300);
const out = {path: location.pathname, legs: document.querySelectorAll('#signal .leg').length, verdict: document.getElementById('verdict')?.textContent};
const k = 'cg.studio.signals'; const s = JSON.parse(localStorage.getItem(k)); s[0].legs[0].amount = 6; localStorage.setItem(k, JSON.stringify(s));
click('verify'); await sleep(300); out.tampered = document.getElementById('verdict')?.textContent;
s[0].legs[0].amount = 5; localStorage.setItem(k, JSON.stringify(s));
out.keys = Object.keys(localStorage).sort();
return JSON.stringify(out)""")
    check("reload keeps the signal", r.get("path") == sig_path and r.get("legs") == 2, str(r.get("path")))
    check("reloaded signal re-verifies", (r.get("verdict") or "").startswith("valid"), r.get("verdict"))
    check("a tampered amount fails verification", (r.get("tampered") or "").startswith("invalid"), r.get("tampered"))
    check("storage keys are namespaced", all(k.startswith("cg.studio.") for k in r.get("keys", ["x"])), str(r.get("keys")))

    r = ev("""document.querySelector('a.tab[href$="/signals"]').click(); await sleep(400);
return JSON.stringify({path: location.pathname, rows: document.querySelectorAll('.row').length, row: document.querySelector('.row')?.textContent, ns: noscroll()})""")
    check("signals list shows it", r.get("rows") == 1 and r.get("path", "").endswith("/signals"), str(r))
    check("signals: no horizontal scroll", r.get("ns"))
    shot("5-signals")

    r = ev("""document.querySelector('a.tab[href$="/intents"]').click(); await sleep(400);
set('scope', 'link every photo of the valley to its place'); click('sealer-anyone'); set('offer', '10'); await sleep(50); click('declare'); await sleep(300);
const first = document.querySelector('#intents .intent')?.textContent;
document.querySelector('#intents .seal button').click(); await sleep(300);
return JSON.stringify({path: location.pathname, first, sealed: document.querySelector('#intents .intent')?.textContent, ns: noscroll()})""")
    check("intent declared open with sealer and offer", all(x in (r.get("first") or "") for x in ("open", "sealer anyone", "offer 10 CYB", "identity proof valid")), r.get("first"))
    check("intent marked sealed by the signal", "sealed by sg-1" in (r.get("sealed") or ""), r.get("sealed"))
    check("intents: no horizontal scroll", r.get("ns"))
    shot("6-intents")

    load()
    r = ev("""return JSON.stringify({path: location.pathname, intents: document.querySelectorAll('#intents .intent').length})""")
    check("reload on /intents keeps route and intent", r.get("path", "").endswith("/intents") and r.get("intents") == 1, str(r))

    # back to the composer through the router: the draft is empty again, names are remembered
    r = ev("""document.querySelector('a.brand').click(); await sleep(400);
set('from', 'the cybergraph'); await sleep(50);
return JSON.stringify({path: location.pathname, empty: !!document.querySelector('#draft .empty'), hint: document.getElementById('from-hint').textContent})""")
    check("composer again: empty draft, name remembered", r.get("empty") and "known text" in r.get("hint", ""), str(r))

    r = ev("""const f = new File(['hello particle'], 'note.txt', {type: 'text/plain'}); const dt = new DataTransfer(); dt.items.add(f);
const inp = document.getElementById('to-file'); inp.files = dt.files; inp.dispatchEvent(new Event('change', {bubbles: true})); await sleep(400);
const viaFile = document.getElementById('to-hint').textContent, name = document.getElementById('to').value;
set('to', 'hello particle'); await sleep(50);
return JSON.stringify({viaFile, name, viaText: document.getElementById('to-hint').textContent})""")
    check("a chosen file gets its particle in the page", r.get("name") == "note.txt" and "file, 14 bytes" in r.get("viaFile", ""), str(r))
    check("the same bytes typed give the same particle", r.get("viaFile", "x").split("particle")[-1] == r.get("viaText", "y").split("particle")[-1], str(r))
    shot("7-file")

    page.pump(0.2)
    check("no page errors", not page.errors, "; ".join(page.errors)[:600])
finally:
    proc.terminate()
print("RESULT", "green" if not fails else f"red: {fails}")
sys.exit(1 if fails else 0)
