use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::Result;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS usage_minute (
    ts INTEGER NOT NULL, profile_id TEXT NOT NULL, mode TEXT NOT NULL,
    up INTEGER NOT NULL DEFAULT 0, down INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts, profile_id, mode)
);
CREATE TABLE IF NOT EXISTS usage_hour (
    ts INTEGER NOT NULL, profile_id TEXT NOT NULL, mode TEXT NOT NULL,
    up INTEGER NOT NULL DEFAULT 0, down INTEGER NOT NULL DEFAULT 0, secs INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts, profile_id, mode)
);
CREATE TABLE IF NOT EXISTS usage_dim (
    ts_hour INTEGER NOT NULL, profile_id TEXT NOT NULL, kind TEXT NOT NULL, key TEXT NOT NULL,
    up INTEGER NOT NULL DEFAULT 0, down INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (ts_hour, profile_id, kind, key)
);
CREATE TABLE IF NOT EXISTS latency (
    profile_id TEXT NOT NULL, ts INTEGER NOT NULL, ms INTEGER
);
CREATE INDEX IF NOT EXISTS idx_latency ON latency (profile_id, ts);
CREATE INDEX IF NOT EXISTS idx_dim_key ON usage_dim (kind, key);
";

const MINUTE_RETENTION_SECS: i64 = 30 * 24 * 3600;
const LATENCY_KEEP: i64 = 40;

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.pragma_update(None, "journal_mode", "WAL")?;
        // Keep the WAL (and the page cache) small: checkpoint every ~100 pages, cap the WAL file.
        conn.pragma_update(None, "wal_autocheckpoint", 100)?;
        conn.pragma_update(None, "journal_size_limit", 1_048_576)?;
        conn.pragma_update(None, "cache_size", -2000)?;
        conn.execute_batch(SCHEMA)?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn upsert_minute(&self, ts: i64, profile: &str, mode: &str, up: u64, down: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO usage_minute (ts, profile_id, mode, up, down) VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(ts, profile_id, mode) DO UPDATE SET up = up + excluded.up, down = down + excluded.down",
            params![ts, profile, mode, up as i64, down as i64],
        )?;
        Ok(())
    }

    pub fn upsert_dim(&self, hour: i64, profile: &str, kind: &str, key: &str, up: u64, down: u64) -> Result<()> {
        self.conn.execute(
            "INSERT INTO usage_dim (ts_hour, profile_id, kind, key, up, down) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(ts_hour, profile_id, kind, key) DO UPDATE SET up = up + excluded.up, down = down + excluded.down",
            params![hour, profile, kind, key, up as i64, down as i64],
        )?;
        Ok(())
    }

    pub fn sum_since(&self, from: i64, to: i64) -> Result<(u64, u64)> {
        let (up, down): (i64, i64) = self.conn.query_row(
            "SELECT COALESCE(SUM(up), 0), COALESCE(SUM(down), 0) FROM (
                SELECT up, down FROM usage_minute WHERE ts >= ?1 AND ts < ?2
                UNION ALL SELECT up, down FROM usage_hour WHERE ts >= ?1 AND ts < ?2)",
            params![from, to],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        Ok((up.max(0) as u64, down.max(0) as u64))
    }

    /// Folds minute rows older than 30 days into hourly rows and drops old detail rows.
    pub fn maintain(&self, now: i64, detail_retention_days: u32) -> Result<()> {
        let cutoff = (now - MINUTE_RETENTION_SECS) - (now - MINUTE_RETENTION_SECS).rem_euclid(3600);
        self.conn.execute(
            "INSERT INTO usage_hour (ts, profile_id, mode, up, down, secs)
             SELECT ts - ts % 3600, profile_id, mode, SUM(up), SUM(down), COUNT(*) * 60
             FROM usage_minute WHERE ts < ?1 GROUP BY 1, 2, 3
             ON CONFLICT(ts, profile_id, mode) DO UPDATE SET
               up = up + excluded.up, down = down + excluded.down, secs = secs + excluded.secs",
            params![cutoff],
        )?;
        self.conn.execute("DELETE FROM usage_minute WHERE ts < ?1", params![cutoff])?;
        let detail_cutoff = now - i64::from(detail_retention_days) * 24 * 3600;
        self.conn.execute("DELETE FROM usage_dim WHERE ts_hour < ?1", params![detail_cutoff])?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<()> {
        self.conn
            .execute_batch("DELETE FROM usage_minute; DELETE FROM usage_hour; DELETE FROM usage_dim;")?;
        Ok(())
    }

    pub fn clear_details(&self) -> Result<()> {
        self.conn.execute_batch("DELETE FROM usage_dim;")?;
        Ok(())
    }

    pub fn record_latency(&self, profile: &str, ts: i64, ms: Option<u64>) -> Result<()> {
        self.conn.execute(
            "INSERT INTO latency (profile_id, ts, ms) VALUES (?1, ?2, ?3)",
            params![profile, ts, ms.map(|m| m as i64)],
        )?;
        self.conn.execute(
            "DELETE FROM latency WHERE profile_id = ?1 AND ts NOT IN
             (SELECT ts FROM latency WHERE profile_id = ?1 ORDER BY ts DESC LIMIT ?2)",
            params![profile, LATENCY_KEEP],
        )?;
        Ok(())
    }

    pub fn latency_history(&self) -> Result<Vec<(String, Vec<Option<u64>>)>> {
        let mut statement = self.conn.prepare("SELECT profile_id, ms FROM latency ORDER BY profile_id, ts")?;
        let rows = statement.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<i64>>(1)?)))?;
        let mut grouped: Vec<(String, Vec<Option<u64>>)> = Vec::new();
        for row in rows {
            let (profile, ms) = row?;
            let ms = ms.map(|m| m.max(0) as u64);
            match grouped.last_mut() {
                Some((last, values)) if *last == profile => values.push(ms),
                _ => grouped.push((profile, vec![ms])),
            }
        }
        Ok(grouped)
    }
}
