//! Photon store: SQLite persistence behind a dedicated owner.
//! Mirrors Zephium's `zephium-store`: owns blocking DB work + migrations;
//! domain decisions stay in core/app. Private visits are never recorded.

use photon_core::StorePort;
use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error(transparent)]
    Db(#[from] rusqlite::Error),
}

pub struct SqliteStore {
    conn: Connection,
}

impl SqliteStore {
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn open_at(path: &std::path::Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS history (
                 id INTEGER PRIMARY KEY AUTOINCREMENT,
                 url TEXT NOT NULL,
                 title TEXT NOT NULL DEFAULT '',
                 visited_at INTEGER NOT NULL
             );
             CREATE INDEX IF NOT EXISTS idx_history_url ON history(url);
             CREATE TABLE IF NOT EXISTS session_snapshot (
                 id INTEGER PRIMARY KEY CHECK (id = 1),
                 snapshot_json TEXT NOT NULL
             );",
        )?;
        Ok(())
    }
}

impl StorePort for SqliteStore {
    type Error = StoreError;

    fn record_visit(&mut self, url: &str, title: &str, is_private: bool) -> Result<(), StoreError> {
        if is_private {
            return Ok(()); // private sessions never reach durable storage
        }
        self.conn.execute(
            "INSERT INTO history (url, title, visited_at) VALUES (?1, ?2, strftime('%s','now'))",
            params![url, title],
        )?;
        Ok(())
    }

    fn search_history(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<(String, String)>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT url, title FROM history WHERE url LIKE ?1 OR title LIKE ?1 ORDER BY visited_at DESC LIMIT ?2",
        )?;
        let pattern = format!("%{query}%");
        let rows = stmt.query_map(params![pattern, limit as i64], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    fn save_session(&mut self, snapshot_json: &str) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO session_snapshot (id, snapshot_json) VALUES (1, ?1)
             ON CONFLICT(id) DO UPDATE SET snapshot_json = excluded.snapshot_json",
            params![snapshot_json],
        )?;
        Ok(())
    }

    fn load_session(&self) -> Result<Option<String>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT snapshot_json FROM session_snapshot WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get::<_, String>(0)?))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_records_and_searches() {
        let mut s = SqliteStore::open_in_memory().unwrap();
        s.record_visit("https://example.com", "Example", false)
            .unwrap();
        s.record_visit("https://secret.example", "Secret", true)
            .unwrap();
        let hits = s.search_history("example", 10).unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].0, "https://example.com");
    }

    #[test]
    fn private_never_persisted() {
        let mut s = SqliteStore::open_in_memory().unwrap();
        s.record_visit("https://incognito.example", "X", true)
            .unwrap();
        assert!(s.search_history("incognito", 10).unwrap().is_empty());
    }

    #[test]
    fn session_roundtrip() {
        let mut s = SqliteStore::open_in_memory().unwrap();
        assert!(s.load_session().unwrap().is_none());
        s.save_session("{\"tabs\":[]}").unwrap();
        assert_eq!(s.load_session().unwrap().as_deref(), Some("{\"tabs\":[]}"));
    }
}
