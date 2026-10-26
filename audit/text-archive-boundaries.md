# TextArchive boundary review

Date: 2026-09-13. Scope: P04 neuron/cell convergence, shared text observations
used by cyb and Soma. Source owners are `src/text_archive.rs` and
`src/text_archive/legacy.rs`; storage and content authentication remain in
ApplicationGraph/BBG and the existing Content/Hemera implementation.

## Findings and changes

The old reader validated all JSONL syntax and text hashes before publication,
but discovered some valid-input limits only during later publication:

- A near-8 MiB raw JSON line plus its decoded text and canonical observation can
  exceed the owner's 16 MiB atomic content budget. Earlier source lines could
  already have committed before that rejection.
- New observations could successfully grow the distinct-text projection beyond
  its 64 MiB reader limit, or history beyond its 1,000,000-observation limit,
  leaving subsequent projection reads unable to succeed.

Preparation now validates the exact canonical observation and codec-tagged
content byte total. A complete import simulates its original request keys and
successive heads, checks conflicts, count and resulting projection before its
first publication. `remember` checks the same projection/count bounds before
appending. Existing duplicate requests return their original receipts; repeated
text contributes once to the projection while retaining each observation.

Publication uses the expected heads from that preflight sequence. A concurrent
append causes the owner CAS to reject a conflicting publication, so two writers
cannot both consume the same remaining projection capacity. Readers select one
immutable head. The retained raw-provenance blob must have the expected codec
and bounded size as well as its verified content identity.

## Limits and failure meaning

| Boundary | Meaning |
|---|---|
| 8 MiB text | UTF-8 bytes, including multibyte characters |
| 8 MiB legacy line | Exact raw line bytes including LF; JSON overhead/escaping counts |
| 16 MiB observation transaction | Decoded text + exact raw line when present + canonical observation + each codec tag, after per-proposal content deduplication |
| 256 MiB source | Exact bounded JSONL input; independent of transaction size |
| 64 MiB projection | Sum of distinct retained text bytes; Rust map/metadata allocation is additional bounded overhead, not a serialized JSON-size promise |
| 1,000,000 observations | Retained observation history count, including repeated text |

All these constraints apply together. An input inside the raw-line bound can
still fail the atomic transaction bound; the error names the source line before
any new source observation is published. Several valid observations whose total
source exceeds 16 MiB are supported because each publication uses its own owner
transaction. Capacity-invalid source files remain available for explicit handling.

Whole-source storage atomicity is intentionally a resumable sequence of atomic
observations. A persistence failure or writer conflict can leave a durable prefix
and returns an error, never an ImportReport claiming completion. Publication errors
include source digest, current line and the number of resolved preceding lines.
Retry uses the exact retained source and original line request keys. Cyb's
GraphSession retires a legacy source only after import returns success. This
review changes no source bytes, UTF-8 hashes, observation schema or legacy request
derivation, and does not rewrite already stored history.

Preflight reads the bounded existing projection/history and holds its ID/length
index while checking the source. It has explicit finite limits, but its cost grows
with retained history. The 64 MiB limit is enforced before new publication; an
already over-limit database from an earlier writer still requires explicit
recovery rather than silent truncation.

## Verification

- Existing compatibility suite: three tests passed, including literal tab/CR,
  unknown dates, duplicate source lines, malformed late lines and truncated tail.
  Log: `../../soft3/audit/neuron-cell/implementation-baseline/p04-text-archive-boundaries-existing.log`.
- Cyb GraphSession migration integration: five tests passed, including shared
  database text-sidecar import, reopening and retirement of the old writer path.
  Log: `../../soft3/audit/neuron-cell/implementation-baseline/p04-text-archive-boundaries-cyb.log`.
- Dedicated boundaries: all four tests passed (debug profile, 1485.88 s).
  `tests/text_archive_boundaries.rs` covers actual 8 MiB
  multibyte text/reopen, valid late JSON exceeding 16 MiB/no prefix, a 63 MiB
  multi-transaction source with exact raw provenance, rejection before projection
  overflow, concurrent writers at the 64 MiB boundary, duplicate retry and reopen.
  It also poisons the actual shared BBG transaction lock to verify error propagation,
  preservation of previously committed state and exact retry after reopen.
  Log: `../../soft3/audit/neuron-cell/implementation-baseline/p04-text-archive-boundaries.log`.
- `rustfmt --check` on the three Rust files and whitespace/diff checks passed.

The dedicated fault case is a shared database writer-lock failure, not injected
disk I/O. BBG's backend persistence-fault tests remain the evidence for failures
inside atomic durable publication. No new production fault hook or fake text
storage backend was introduced.
