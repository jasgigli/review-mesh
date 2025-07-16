use rusqlite::{Connection, Result, params};
use common::{ReviewSession, Comment, ChatLine};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        // Create tables if not exist
        conn.execute_batch(r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT,
                created_at TEXT,
                participants TEXT
            );
            CREATE TABLE IF NOT EXISTS comments (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                author TEXT,
                file TEXT,
                hunk_id TEXT,
                line INTEGER,
                body TEXT,
                created_at TEXT,
                resolved INTEGER
            );
            CREATE TABLE IF NOT EXISTS chat (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                author TEXT,
                body TEXT,
                created_at TEXT
            );
        "#)?;
        Ok(Self { conn })
    }

    pub fn save_comment(&self, comment: &Comment) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO comments (id, session_id, author, file, hunk_id, line, body, created_at, resolved) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                comment.id,
                comment.session_id,
                comment.author,
                comment.file,
                comment.hunk_id,
                comment.line as i64,
                comment.body,
                comment.created_at.to_rfc3339(),
                comment.resolved as i64,
            ],
        )?;
        Ok(())
    }

    pub fn save_chat(&self, chat: &ChatLine) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO chat (id, session_id, author, body, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                chat.id,
                chat.session_id,
                chat.author,
                chat.body,
                chat.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn load_comments(&self, session_id: &str) -> Result<Vec<Comment>> {
        let mut stmt = self.conn.prepare("SELECT id, session_id, author, file, hunk_id, line, body, created_at, resolved FROM comments WHERE session_id = ?1")?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Comment {
                id: row.get(0)?,
                session_id: row.get(1)?,
                author: row.get(2)?,
                file: row.get(3)?,
                hunk_id: row.get(4)?,
                line: row.get::<_, i64>(5)? as usize,
                body: row.get(6)?,
                created_at: row.get::<_, String>(7)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
                resolved: row.get::<_, i64>(8)? != 0,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn load_chat(&self, session_id: &str) -> Result<Vec<ChatLine>> {
        let mut stmt = self.conn.prepare("SELECT id, session_id, author, body, created_at FROM chat WHERE session_id = ?1")?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(ChatLine {
                id: row.get(0)?,
                session_id: row.get(1)?,
                author: row.get(2)?,
                body: row.get(3)?,
                created_at: row.get::<_, String>(4)?.parse().unwrap_or_else(|_| chrono::Utc::now()),
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    pub fn queue_offline(&self, data: &[u8]) -> Result<()> {
        // Queue for offline replay
        todo!("queue offline")
    }

    pub fn replay_queue(&self) -> Result<()> {
        // Replay queued writes
        todo!("replay queue")
    }
}
