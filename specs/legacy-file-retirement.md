# Retirement of a legacy file entry point

After a strict importer has durably accepted the source, the local host retires
its ordinary append-file path into a directory containing the original file and
a versioned manifest. Old append/open-as-file calls fail at that path. The source
bytes remain available for inspection; the new application uses its declared
BBG target. A missing target after retirement requires explicit recovery, not a
new empty database masquerading as the old installation.

The supported local profile uses no-replace atomic renames (Apple, Linux/Android),
durable directory/file synchronization and an exclusive source-file lock. The
host must quiesce old writers before migration: advisory locks cannot revoke an
already-open descriptor held by an old program that never acquired a lock.
Retirement does not claim to stop an uncooperative running legacy process.

Preparation creates and synchronizes a complete staging directory before exposing
its deterministic `.retiring-v1` name. Its manifest pins schema, source kind and
byte hash. The source moves inside without overwriting an existing entry; the
complete directory then takes the original path without overwriting a competitor.
Every interruption leaves either the original file or a discoverable staged or
retired source. Rerun validates the manifest/hash and resumes the same operation.
Symlink aliases, malformed markers and conflicting paths fail without replacement.
The importer must resolve its durable acceptance before invoking retirement.
