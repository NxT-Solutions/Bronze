//! Placeholder item filter (story 4.7, QUE-007, G-06, DG-10).
//! ADR-018 stays Proposed; this hook does not complete locale search.

use crate::migrate::Store;
use bronze_domain::fuzzy_best_score;

pub const ADR_018_STATUS: &str = "Proposed";
pub const QUE_007_COMPLETE: bool = false;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchKind {
    PlaceholderSubstring,
    LocaleAware,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SearchError {
    LocaleSemanticsUnavailable,
    Store,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SearchHit {
    pub item_id: String,
}

pub fn claims_locale_search() -> bool {
    false
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SearchAnnouncement {
    pub count: usize,
}

pub fn announce_search(hits: &[SearchHit]) -> SearchAnnouncement {
    SearchAnnouncement { count: hits.len() }
}

impl Store {
    pub fn search_placeholder(&self, query: &str) -> Result<Vec<SearchHit>, SearchError> {
        self.search(query, SearchKind::PlaceholderSubstring)
    }

    pub fn search_placeholder_in_locale(
        &self,
        query: &str,
        locale: &str,
    ) -> Result<Vec<SearchHit>, SearchError> {
        self.search_for_locale(query, SearchKind::PlaceholderSubstring, locale)
    }

    pub fn search(&self, query: &str, kind: SearchKind) -> Result<Vec<SearchHit>, SearchError> {
        self.search_for_locale(query, kind, "en")
    }

    pub fn search_for_locale(
        &self,
        query: &str,
        kind: SearchKind,
        locale: &str,
    ) -> Result<Vec<SearchHit>, SearchError> {
        if kind == SearchKind::LocaleAware || !matches_placeholder_contract() {
            return Err(SearchError::LocaleSemanticsUnavailable);
        }
        let rows = self.list_items(true).map_err(|_| SearchError::Store)?;
        if query.trim().is_empty() {
            return Ok(rows
                .into_iter()
                .map(|row| SearchHit { item_id: row.id })
                .collect());
        }
        let mut ranked = Vec::new();
        for row in rows {
            let title = row.title.unwrap_or_default();
            let app = row.source_app_name.unwrap_or_default();
            let bundle = row.source_bundle_id.unwrap_or_default();
            let Some(score) = fuzzy_best_score(
                [
                    row.body.as_str(),
                    title.as_str(),
                    app.as_str(),
                    bundle.as_str(),
                ],
                query,
                locale,
            ) else {
                continue;
            };
            if score > 0 {
                ranked.push((score, row.id));
            }
        }
        ranked.sort_by_key(|row| std::cmp::Reverse(row.0));
        Ok(ranked
            .into_iter()
            .map(|(_, item_id)| SearchHit { item_id })
            .collect())
    }
}

fn matches_placeholder_contract() -> bool {
    ADR_018_STATUS == "Proposed" && !QUE_007_COMPLETE && !claims_locale_search()
}

#[cfg(test)]
mod search_placeholder_tests {
    use super::*;
    use crate::migrate::{NoopBackup, PathLocator};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static SEQ: AtomicU64 = AtomicU64::new(1);

    fn open_store() -> Store {
        let n = SEQ.fetch_add(1, Ordering::Relaxed);
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("bronze-search-{nanos}-{n}"));
        std::fs::create_dir_all(&dir).expect("dir");
        let locator = PathLocator {
            path: dir.join("bronze.sqlite"),
        };
        let mut backup = NoopBackup;
        let store = Store::open(&locator, &mut backup).expect("open");
        store
            .conn
            .execute_batch(
                "INSERT INTO workspaces VALUES ('w1','ws',1,1);
                 INSERT INTO sections VALUES ('s1','w1','inbox','a','active',NULL,1,1,1,NULL);
                 INSERT INTO items VALUES ('i1','s1','note','Café token','und','queued','a',NULL,1,1,1,NULL,NULL,NULL);
                 INSERT INTO items VALUES ('i2','s1','note','plain','und','queued','b',NULL,1,1,1,NULL,NULL,NULL);",
            )
            .expect("seed");
        store
    }

    #[test]
    fn search_placeholder_does_not_claim_locale_or_complete_que007() {
        assert_eq!(ADR_018_STATUS, "Proposed");
        assert!(!QUE_007_COMPLETE);
        assert!(!claims_locale_search());
        let store = open_store();
        assert_eq!(
            store.search("cafe", SearchKind::LocaleAware).unwrap_err(),
            SearchError::LocaleSemanticsUnavailable
        );
        let hits = store.search_placeholder("Café").expect("hits");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].item_id, "i1");
        assert!(store.search_placeholder("cafe").expect("ascii").is_empty());
    }

    #[test]
    fn search_placeholder_leaves_raw_body_unchanged() {
        let store = open_store();
        let before: String = store
            .conn
            .query_row("SELECT body FROM items WHERE id='i1'", [], |row| row.get(0))
            .expect("before");
        store.search_placeholder("Café").expect("search");
        let after: String = store
            .conn
            .query_row("SELECT body FROM items WHERE id='i1'", [], |row| row.get(0))
            .expect("after");
        assert_eq!(before, "Café token");
        assert_eq!(before, after);
    }

    #[test]
    fn search_placeholder_query_never_enters_diagnostics() {
        let store = open_store();
        let query = "unique-search-needle-4-7";
        store
            .conn
            .execute(
                "INSERT INTO items VALUES ('i3','s1','note',?1,'und','queued','c',NULL,1,1,1,NULL,NULL,NULL)",
                [query],
            )
            .expect("seed needle");
        store.search_placeholder(query).expect("search");
        let diag_rows: i64 = store
            .conn
            .query_row("SELECT COUNT(*) FROM diagnostic_events", [], |row| {
                row.get(0)
            })
            .expect("diag count");
        assert_eq!(diag_rows, 0);
        let leaked: i64 = store
            .conn
            .query_row(
                "SELECT COUNT(*) FROM diagnostic_events
                 WHERE id LIKE ?1 OR request_id LIKE ?1 OR stage LIKE ?1
                    OR result_code LIKE ?1 OR trigger_kind LIKE ?1
                    OR source_bundle_id LIKE ?1 OR build_id LIKE ?1",
                [format!("%{query}%")],
                |row| row.get(0),
            )
            .expect("leak");
        assert_eq!(leaked, 0);
    }

    #[test]
    fn search_ui_announces_count_without_query_or_que007() {
        assert!(!QUE_007_COMPLETE);
        assert_eq!(ADR_018_STATUS, "Proposed");
        let store = open_store();
        let query = "secret-query-never-in-announce";
        let hits = store.search_placeholder("plain").expect("hits");
        let announced = announce_search(&hits);
        assert_eq!(announced.count, 1);
        let rendered = format!("{announced:?}{announced:?}");
        assert!(!rendered.contains(query));
        assert!(!rendered.contains("plain"));
    }

    #[test]
    fn search_is_fuzzy_case_insensitive_and_scans_the_full_list() {
        use crate::queue::{QueueListFilter, QUEUE_PAGE_SIZE};

        let store = open_store();
        let page = QUEUE_PAGE_SIZE as i64;
        for index in 0..page {
            store
                .conn
                .execute(
                    "INSERT INTO items VALUES (?1,'s1','note','filler','und','queued',?2,NULL,1,1,1,NULL,NULL,NULL)",
                    rusqlite::params![format!("fill-{index}"), format!("m{index:02}")],
                )
                .expect("filler");
        }
        store
            .conn
            .execute(
                "INSERT INTO sources VALUES ('src-safari','com.apple.Safari','Safari',NULL,NULL,1,1)",
                [],
            )
            .expect("source");
        store
            .conn
            .execute(
                "INSERT INTO items VALUES ('late','s1','note','unrelated words','und','queued','z','src-safari',1,1,1,NULL,NULL,'Bronze')",
                [],
            )
            .expect("late");

        let first = store
            .query_queue_page(None, QueueListFilter::Overview, QUEUE_PAGE_SIZE)
            .expect("page");
        assert!(first.next_cursor.is_some());
        assert!(!first.items.iter().any(|row| row.id == "late"));

        let fuzzy = store.search_placeholder("brn").expect("fuzzy");
        assert_eq!(fuzzy.len(), 1);
        assert_eq!(fuzzy[0].item_id, "late");
        let upper = store.search_placeholder("BRONZE").expect("case");
        assert_eq!(upper[0].item_id, "late");
        assert!(store.search_placeholder("zzzz").expect("miss").is_empty());
        assert!(store.search_placeholder("cafe").expect("ascii").is_empty());
        let folded = store.search_placeholder("CAFÉ").expect("folded");
        assert_eq!(folded.len(), 1);
        assert_eq!(folded[0].item_id, "i1");

        let safari = store.search_placeholder("sfri").expect("app");
        assert_eq!(safari.len(), 1);
        assert_eq!(safari[0].item_id, "late");

        let all = store.search_placeholder("  ").expect("empty");
        let listed = store.list_items(true).expect("list");
        assert_eq!(all.len(), listed.len());
        assert_eq!(
            all.iter()
                .map(|hit| hit.item_id.as_str())
                .collect::<Vec<_>>(),
            listed.iter().map(|row| row.id.as_str()).collect::<Vec<_>>()
        );

        let turkish = store
            .search_placeholder_in_locale("ıstanbul", "tr")
            .expect("tr miss");
        assert!(turkish.is_empty());
        store
            .conn
            .execute(
                "INSERT INTO items VALUES ('ist','s1','note','Istanbul','und','queued','y',NULL,1,1,1,NULL,NULL,NULL)",
                [],
            )
            .expect("istanbul");
        let dotted = store
            .search_placeholder_in_locale("istanbul", "tr")
            .expect("tr dotted");
        assert!(dotted.is_empty());
        let dotless = store
            .search_placeholder_in_locale("ıstanbul", "tr")
            .expect("tr dotless");
        assert_eq!(dotless.len(), 1);
        assert_eq!(dotless[0].item_id, "ist");
        let english = store
            .search_for_locale("istanbul", SearchKind::PlaceholderSubstring, "en")
            .expect("en");
        assert_eq!(english[0].item_id, "ist");
    }
}
