# Data model and portability

## 1. Objectives

Store must be crash-safe, locally inspectable, upgradeable, recoverable, performant at 10,000+ items, and portable without preserving machine-specific absolute paths. User owns content; app owns invariants.

## 2. SQLite configuration

- One writer actor and bounded read connections.
- `PRAGMA journal_mode=WAL`, `foreign_keys=ON`, tested `synchronous` policy, busy timeout.
- Database and backup directory permissions restricted to user.
- Stable application container chosen by ADR-009; document exact path in About.
- One app instance; secondary activation routes to primary.
- Run lightweight integrity check on clean startup cadence and full check before/after restore.

Never expose SQL plugin directly to WebView.

## 3. Logical schema

Illustrative; migrations own exact SQL.

```text
workspaces
  id TEXT PK
  name TEXT NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL

sections
  id TEXT PK
  workspace_id TEXT FK
  title TEXT NOT NULL
  rank TEXT NOT NULL
  state TEXT CHECK(active|archived|trashed)
  color_token TEXT NULL
  revision INTEGER NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL
  deleted_at_ms INTEGER NULL

items
  id TEXT PK
  section_id TEXT FK
  kind TEXT CHECK(context|prompt|note|task|snippet)
  body TEXT NOT NULL
  title TEXT NULL
  content_language TEXT NOT NULL DEFAULT 'und'
  status TEXT CHECK(queued|copied|active|done|skipped|trashed)
  rank TEXT NOT NULL
  source_id TEXT NULL FK
  revision INTEGER NOT NULL
  created_at_ms INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL
  completed_at_ms INTEGER NULL
  deleted_at_ms INTEGER NULL

sources
  id TEXT PK
  bundle_id TEXT NULL
  app_name TEXT NULL
  safe_title TEXT NULL
  url TEXT NULL
  captured_at_ms INTEGER NOT NULL
  policy_version INTEGER NOT NULL

item_revisions
  item_id TEXT FK
  revision INTEGER
  body TEXT
  content_language TEXT
  status TEXT
  changed_at_ms INTEGER
  change_kind TEXT
  PRIMARY KEY(item_id, revision)

output_profiles
  id TEXT PK
  builtin_key TEXT NULL
  name TEXT NULL
  format TEXT NOT NULL
  format_options_json TEXT NOT NULL
  source_policy TEXT NOT NULL
  post_copy_action TEXT NOT NULL
  advance_policy TEXT NOT NULL
  revision INTEGER NOT NULL

shortcuts
  action TEXT PK
  trigger_json TEXT NOT NULL
  enabled INTEGER NOT NULL
  revision INTEGER NOT NULL
  updated_at_ms INTEGER NOT NULL

settings
  key TEXT PK
  schema_version INTEGER
  json_value TEXT
  updated_at_ms INTEGER

undo_log
  id TEXT PK
  command_type TEXT
  inverse_json TEXT
  created_at_ms INTEGER
  expires_at_ms INTEGER

command_receipts
  id TEXT PK
  command_type TEXT NOT NULL
  result_code TEXT NOT NULL
  created_at_ms INTEGER NOT NULL
  expires_at_ms INTEGER NOT NULL

command_receipt_entities
  command_id TEXT FK
  ordinal INTEGER NOT NULL
  entity_id TEXT NOT NULL
  result_revision INTEGER NULL
  PRIMARY KEY(command_id, ordinal)

schema_migrations
  version INTEGER PK
  name TEXT
  checksum TEXT
  applied_at_ms INTEGER

diagnostic_events
  id TEXT PK
  occurred_at_ms INTEGER
  request_id TEXT NULL
  stage TEXT
  result_code TEXT
  duration_ms INTEGER NULL
  trigger_kind TEXT NULL
  provider_kind TEXT NULL
  permission_state TEXT NULL
  source_bundle_id TEXT NULL
  app_schema_version INTEGER NOT NULL
  queue_depth INTEGER NULL
  overflow_count INTEGER NULL
  tap_health TEXT NULL
  store_result_code TEXT NULL
  build_id TEXT NOT NULL
```

No arbitrary diagnostic JSON or raw framework error string is persisted. Closed enums/typed nullable fields are the schema boundary. Indexes: section rank/state, item section/rank/status, updated times, source bundle. FTS5 external-content table covers item body and allowed source labels; triggers or explicit store layer keep it synchronized.

`output_profiles.post_copy_action` is closed to `unchanged|copied|active|done`; `advance_policy` is `keep|nextQueued`. Output profile is sole lifecycle/advance authority after copy.

`format_options_json` validates against versioned discriminated union owned by Rust: plain separator; Markdown bullet marker/continuation or numbered start; prompt-block literal headings/delimiters and provenance labels. No field is interpreted as code, interpolation, HTML, or command; syntax-like user text remains literal. String/byte/newline limits and exact-output preview apply. UI-locale changes never mutate stored output literals.

`builtin_key` is closed stable identifier for reset/localized presentation. Built-in row may have null `name` to use current catalog label; user rename stores literal override. Custom row requires non-empty `name` and null `builtin_key`. Reset clears override and restores versioned built-in options without changing profile ID.

Mutation transaction writes content-free command receipt plus ordered entity/revision references. Duplicate ID inside bounded retry window reconstructs prior result; timestamped ID older than window returns `idempotency_expired` and never re-executes. Receipts expire after proposed 7 days; they contain no item body or inverse payload.

## 4. IDs and order

- Use UUIDv7 or ULID generated in Rust; never database row ID in external format.
- IDs are opaque lowercase canonical strings.
- Order key uses proven fractional/gapped strategy and locale-independent byte collation.
- Rebalance a section transactionally when keys approach limit; identity and revisions remain unchanged.
- Unique constraint on `(section_id, rank)` plus retry handles concurrent command edge cases.

## 5. Content fidelity

- Store UTF-8 exactly as captured/authored, including newlines and edge whitespace.
- Do not silently NFC/NFKC normalize user text. Search may maintain derived normalized index without changing source.
- Count limits by UTF-8 bytes plus grapheme-aware UI feedback.
- Proposed P0 item cap: 1 MiB, configurable only internally. Larger selection requires explicit cancel/copy-to-file path; never truncate.
- `items.title` is nullable TEXT (migration `items_title`). Persist, composer add, and body edit write `compact_title` first (40-character word-boundary clamp, no ellipsis glyph). A local allow-listed GGUF refine (`general.titleModel`) may replace that string after persist; `clean_title` strips a leaked `Title:` prefix; failure keeps `compact_title` (ADR-019 Proposed). Existing rows keep their stored string until recapture or edit.
- Capture may store constrained markdown in `items.body` (bold/italic plus exact whitespace). Inbox render is sanitized: createElement/createTextNode only, including line-start `ul`/`ol`/`li`; raw HTML and remote images disabled. Copy writes that same dialect as escaped HTML on the pasteboard. Composer text is stored as authored.
- Source URL is optional, allowlisted by scheme, and excluded by default from output.
- `content_language` is canonical BCP 47 or `und`; application validation rejects invalid tags. New captures/manual notes default to `und` unless user assigns language. Bronze never silently detects language.
- Render item text with its own `lang` value and `dir=auto`; do not let UI-locale `lang` incorrectly label unknown or different-language content.

## 6. Migrations

Each migration has monotonic integer, stable name, source checksum, forward SQL/code, test fixture, and rollback/recovery strategy. Procedure:

1. Acquire exclusive migration lock.
2. Create online SQLite backup.
3. Verify current schema version/checksums.
4. Apply pending migration in transaction where SQLite permits.
5. Validate foreign keys, expected schema, and integrity.
6. Commit and record checksum.
7. On failure, keep original/backup, enter read-only recovery, never ignore `ALTER TABLE` errors.

CI upgrades fixtures from every supported schema version and injects crash/failure between phases.

## 7. Backup

- Use SQLite online backup API, not file copy of live WAL database.
- Automatic rotation: proposed daily 7, weekly 4, pre-migration 3; user-configurable retention within safe bounds.
- Backup manifest includes app version, schema version, timestamp, DB checksum, asset list/checksum.
- Restore operates on copy, validates/migrates, backs up current state, then atomic swaps.
- Recovery UI can browse metadata without opening untrusted DB in primary process if practical.

## 8. Export format

Canonical deterministic archive:

```text
bronze-export-YYYYMMDD-HHMMSS/
  manifest.json
  data.json
  notes/
    <section-slug>-<section-id>.md
  assets/                   # P2, content-addressed
  checksums.sha256
```

`data.json` example:

```json
{
  "format": "bronze-export",
  "version": 1,
  "exportedAt": "2026-08-27T12:00:00Z",
  "workspace": { "id": "...", "name": "Bronze" },
  "sections": [],
  "items": [],
  "sources": [],
  "outputProfiles": [],
  "shortcuts": [],
  "settings": {}
}
```

Rules:

- UTF-8, LF, sorted keys where defined, arrays in semantic order, RFC 3339 UTC. Every exported item preserves `contentLanguage` as canonical BCP 47 or `und`.
- No machine-specific absolute path, PID, permission database entry/token, app credential, diagnostic payload, or internal secret. Export necessarily contains selected/user-authored item content and enabled provenance, which may themselves contain user secrets; preview and warning state this before write.
- Settings export includes safe user preferences; excludes install identity and ephemeral state.
- Settings file export is a distinct document from this queue archive. `format` is `bronze-settings` (not `bronze-export`). The Settings window writes/reads that file through rust-owned save/open panels (`export_settings_file` / `import_settings_file`). Library archive commands stay archive-only. A settings file that claims `bronze-export`, includes credentials, permission tokens, diagnostics, or machine paths, or arrives as a WebView path is rejected (SET-001, SEC-003). Preview of a pending settings write lists included categories and leftover user-entered literals (excluded apps, app policies, custom shortcuts, profile literals) without sending raw payload or filesystem paths to the WebView.
- Markdown filenames collision-proof by ID; links relative; front matter schema versioned.
- Write staging directory, fsync/verify checksums, then atomic rename. Existing path requires explicit Replace or new name.

## 9. Import

1. Native file picker yields operation-scoped token/path.
2. Rust checks canonical path, archive size, entry count, traversal/symlink, compression ratio, and type.
3. Parse against explicit versioned schema with unknown-field policy.
4. Validate IDs, enums, canonical BCP 47/`und` content-language values, timestamps, lengths, checksums, references, rank uniqueness.
5. Preview counts/conflicts and provenance impact.
6. Because v1 has one workspace, user chooses merge into current workspace or replace current workspace after automatic backup. “Create new workspace” is out of scope.
7. Apply to staging DB in transaction, run integrity/FTS rebuild, then commit.

No imported HTML/script executes. Markdown preview sanitizes. URLs never auto-open or fetch.

## 10. Deletion and undo

- Delete changes state to trashed/tombstone, preserving descendants and revision.
- Purge query selects only records whose entire dependency set is eligible; never rely on direct-child check.
- Default trash period 30 days; user can keep forever or shorten with clear warning.
- Item revisions retain at most 20 revisions and 30 days by default; undo entries and command receipts expire after 7 days. User may shorten within safe UI. Purging item cascades its revisions, undo payloads, command-receipt entity references, source record, and FTS row, then verifies index rebuild.
- Ordinary Empty Trash lists counts and creates immediate pre-purge backup; UI states purged content remains until automatic backup rotation.
- Separate “Purge from Bronze-controlled storage” deletes eligible primary rows, revisions, undo, command-receipt references, FTS, and Bronze automatic backups without creating a safety backup after explicit typed confirmation. It cannot erase Time Machine, manually exported files, clipboard history, nearby devices, or third-party copies.
- Undo command log stores minimum inverse data; content remains subject to same local protections and explicit retention.

## 11. Search and locale semantics

FTS5 configuration is not assumed globally correct. ADR-018/Spike SEARCH-01 must choose and document tokenizer, case folding, diacritic behavior, word boundaries, prefix/substring semantics, and fallback for Latin, Arabic, Hebrew, CJK, Indic, combining marks, emoji, and mixed code before QUE-007/G-06 are complete. Golden corpus specifies query → expected ordered results per locale. Raw item body remains unchanged; derived index can be rebuilt/purged.

## 12. Performance

- Paginate/query sections; never refetch entire graph after each mutation.
- Domain event includes changed IDs and revisions; React cache updates targeted records.
- Search query budget p95 <100 ms at 10k medium items on minimum supported hardware.
- DB mutation p95 <50 ms excluding fsync variance; capture feedback may precede UI repaint only after durable transaction result.
- Vacuum/FTS optimize occurs during idle with cancellation and power awareness; never blocks capture.

## 13. Data test corpus

Include empty/1 MiB, CRLF/LF, leading spaces/tabs, combining marks, ZWJ emoji, bidi isolates, Arabic/Hebrew, Japanese, Indic, canonical/script/variant and invalid BCP 47 tags, `und`, invalid import UTF-8, duplicate IDs, deep/cyclic references, rank collisions, old schemas, partial WAL, disk-full, permission denied, corrupt page, traversal archive, checksum mismatch, and malicious Markdown.
