# Local text archive in the shared graph

The text archive stores verified content and its local observation metadata
through ApplicationGraph on the existing Database. A namespace identifies the
archive projection; it has no key, neuron identity or network publication rights.
Text keeps its existing particle `hemera::hash(UTF-8 bytes)`. A signal can refer
to this particle independently of whether peers have permission to fetch it.

Each immutable observation records a request key, previous observation, text
particle, optional local creation time and optional legacy provenance. The
canonical versioned record and exact text bytes commit together before success.
The archive retains all observations; a last-observation projection supplies the
legacy particle→text/created view. Unknown legacy dates remain absent. Retries
with identical request and fields resolve the original receipt; changed fields
conflict. Local content observation is distinct from signed subject activity.

Legacy `particles.jsonl` import validates every complete source line, its content
hash and metadata. It preserves the original bytes and line ordering, including
duplicates; malformed/truncated/mismatched lines are reported with positions and
cannot silently disappear. The old hand-written format can contain literal tab
or carriage-return characters inside JSON strings; the compatibility decoder may
escape precisely those bytes for JSON parsing while retaining the raw source.
Missing files are distinct from unreadable files. Imported source records carry
the source digest and line position so import resumes without duplicate history.

Before publishing a new source observation, import validates the complete bounded
source, every canonical observation transaction and the resulting projection and
history count. The publication sequence uses the expected starting head and
successive heads from preflight; a concurrent append must conflict instead of
consuming capacity that the preflight reserved only logically. Syntax, hash,
request conflicts and capacity rejection therefore publish no new source prefix.
Storage failure or concurrent publication can interrupt the later sequence after
a durable prefix; the operation returns an error and the exact retained source
is used to resume through original line request keys. An ImportReport denotes
completion of the complete source, including previously resolved lines.

The host quiesces old sidecar writers before cutover. After successful import,
the old ordinary file entry point is retired and retained evidence remains
readable. A retained original is rechecked before migration is declared complete.
New shell and soma callers share the Database handle; neither appends JSONL or
creates a second authoritative text store. Storage failures propagate to the
operation that required the content. Graph history is authoritative; UI Log and
memory lists are projections. No implicit network disclosure accompanies import.

## Local codec and bounds

Observation content is a Blob containing canonical compact UTF-8 JSON, with
fields in this order: `schema`, `previous`, `request`, `text`, `created`,
`provenance`. Schema is `cybergraph/text-observation/1`; references are 32-byte
JSON arrays, previous/created/provenance are explicitly null when absent.
Provenance fields are `source`, `line`, `raw`; line is one-based and raw references
the exact source line including LF. Unknown fields or noncanonical encodings fail.
Text/raw line size is at most 8 MiB and source import at most 256 MiB. Each
observation must also fit ApplicationGraph's 16 MiB atomic content budget:
decoded text, exact raw source line when present, canonical observation JSON and
every content codec tag, after content deduplication within that proposal. Raw
JSON overhead and escaping count toward the raw-line limit. All limits apply
together; a raw line inside 8 MiB can still exceed the atomic transaction budget.

The full projection contains at most 64 MiB of distinct UTF-8 text bytes and the
history at most one million observations. Repeated text counts once in the
projection but each new observation counts toward history. Rust map/metadata
allocation and any separate serialized view have their own overhead; 64 MiB is
not a promise about a JSON export size. New writes must preserve these reader
bounds. Limits produce an explicit error, never an empty successful projection.
Larger archival tooling may use another declared profile; it must preserve the
same content identities and original provenance.
