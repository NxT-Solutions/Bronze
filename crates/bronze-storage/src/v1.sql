CREATE TABLE workspaces (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE sections (
  id TEXT PRIMARY KEY,
  workspace_id TEXT NOT NULL REFERENCES workspaces(id),
  title TEXT NOT NULL,
  rank TEXT NOT NULL,
  state TEXT NOT NULL CHECK(state IN ('active','archived','trashed')),
  color_token TEXT,
  revision INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  deleted_at_ms INTEGER
);

CREATE TABLE sources (
  id TEXT PRIMARY KEY,
  bundle_id TEXT,
  app_name TEXT,
  safe_title TEXT,
  url TEXT,
  captured_at_ms INTEGER NOT NULL,
  policy_version INTEGER NOT NULL
);

CREATE TABLE items (
  id TEXT PRIMARY KEY,
  section_id TEXT NOT NULL REFERENCES sections(id),
  kind TEXT NOT NULL CHECK(kind IN ('context','prompt','note','task','snippet')),
  body TEXT NOT NULL,
  content_language TEXT NOT NULL DEFAULT 'und',
  status TEXT NOT NULL CHECK(status IN ('queued','copied','active','done','skipped','trashed')),
  rank TEXT NOT NULL,
  source_id TEXT REFERENCES sources(id),
  revision INTEGER NOT NULL,
  created_at_ms INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL,
  completed_at_ms INTEGER,
  deleted_at_ms INTEGER,
  UNIQUE(section_id, rank)
);

CREATE TABLE item_revisions (
  item_id TEXT NOT NULL REFERENCES items(id),
  revision INTEGER NOT NULL,
  body TEXT,
  content_language TEXT,
  status TEXT,
  changed_at_ms INTEGER,
  change_kind TEXT,
  PRIMARY KEY(item_id, revision)
);

CREATE TABLE output_profiles (
  id TEXT PRIMARY KEY,
  builtin_key TEXT,
  name TEXT,
  format TEXT NOT NULL,
  format_options_json TEXT NOT NULL,
  source_policy TEXT NOT NULL,
  post_copy_action TEXT NOT NULL CHECK(post_copy_action IN ('unchanged','copied','active','done')),
  advance_policy TEXT NOT NULL CHECK(advance_policy IN ('keep','nextQueued')),
  revision INTEGER NOT NULL
);

CREATE TABLE shortcuts (
  action TEXT PRIMARY KEY,
  trigger_json TEXT NOT NULL,
  enabled INTEGER NOT NULL,
  revision INTEGER NOT NULL,
  updated_at_ms INTEGER NOT NULL
);

CREATE TABLE settings (
  key TEXT PRIMARY KEY,
  schema_version INTEGER,
  json_value TEXT,
  updated_at_ms INTEGER
);

CREATE TABLE undo_log (
  id TEXT PRIMARY KEY,
  command_type TEXT,
  inverse_json TEXT,
  created_at_ms INTEGER,
  expires_at_ms INTEGER
);

CREATE TABLE command_receipts (
  id TEXT PRIMARY KEY,
  command_type TEXT NOT NULL,
  result_code TEXT NOT NULL,
  created_at_ms INTEGER NOT NULL,
  expires_at_ms INTEGER NOT NULL
);

CREATE TABLE command_receipt_entities (
  command_id TEXT NOT NULL REFERENCES command_receipts(id),
  ordinal INTEGER NOT NULL,
  entity_id TEXT NOT NULL,
  result_revision INTEGER,
  PRIMARY KEY(command_id, ordinal)
);

CREATE TABLE schema_migrations (
  version INTEGER PRIMARY KEY,
  name TEXT,
  checksum TEXT,
  applied_at_ms INTEGER
);

CREATE TABLE diagnostic_events (
  id TEXT PRIMARY KEY,
  occurred_at_ms INTEGER,
  request_id TEXT,
  stage TEXT,
  result_code TEXT,
  duration_ms INTEGER,
  trigger_kind TEXT,
  provider_kind TEXT,
  permission_state TEXT,
  source_bundle_id TEXT,
  app_schema_version INTEGER NOT NULL,
  queue_depth INTEGER,
  overflow_count INTEGER,
  tap_health TEXT,
  store_result_code TEXT,
  build_id TEXT NOT NULL
);

CREATE INDEX idx_sections_rank_state ON sections(rank, state);
CREATE INDEX idx_items_section_rank_status ON items(section_id, rank, status);
CREATE INDEX idx_items_updated_at ON items(updated_at_ms);
CREATE INDEX idx_sections_updated_at ON sections(updated_at_ms);
CREATE INDEX idx_sources_bundle ON sources(bundle_id);
