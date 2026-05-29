use crate::config::data_dir;
use crate::terminal::heading;
use anyhow::Result;
use chrono::Utc;
use rusqlite::{Connection, params};
use uuid::Uuid;

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn open_default() -> Result<Self> {
        let dir = data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let conn = Connection::open(dir.join("forgeflow.sqlite"))?;
        let store = Self { conn };
        store.migrate()?;
        Ok(store)
    }

    pub fn remember(&self, kind: &str, content: &str, project_path: Option<&str>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO memories (id, kind, content, project_path, metadata_json, created_at)
             VALUES (?1, ?2, ?3, ?4, '{}', ?5)",
            params![
                Uuid::new_v4().to_string(),
                kind,
                content,
                project_path,
                Utc::now().to_rfc3339()
            ],
        )?;
        Ok(())
    }

    pub fn print_recent(&self, limit: usize) -> Result<()> {
        heading("Recent Memory");
        let mut stmt = self.conn.prepare(
            "SELECT kind, content, created_at FROM memories ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map([limit as i64], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (kind, content, created_at) = row?;
            println!("[{kind}] {created_at}\n{content}\n");
        }
        Ok(())
    }

    pub fn search(&self, query: &str) -> Result<()> {
        heading("Memory Search");
        let pattern = format!("%{query}%");
        let mut stmt = self.conn.prepare(
            "SELECT kind, content, created_at FROM memories
             WHERE content LIKE ?1 OR kind LIKE ?1
             ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([pattern], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?;
        for row in rows {
            let (kind, content, created_at) = row?;
            println!("[{kind}] {created_at}\n{content}\n");
        }
        Ok(())
    }

    pub fn clear(&self) -> Result<()> {
        self.conn.execute("DELETE FROM memories", [])?;
        Ok(())
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS memories (
                id TEXT PRIMARY KEY,
                kind TEXT NOT NULL,
                content TEXT NOT NULL,
                project_path TEXT,
                metadata_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_memories_kind ON memories(kind);
            CREATE INDEX IF NOT EXISTS idx_memories_project_path ON memories(project_path);",
        )?;
        Ok(())
    }
}
