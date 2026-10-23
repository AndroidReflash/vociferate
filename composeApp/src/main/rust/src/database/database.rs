use rusqlite::{Connection, params};
use std::sync::{Arc, Mutex};
use thiserror::Error;

const DEVICE_ID_KEY: &str = "device_id";

#[derive(Debug, Error, uniffi::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlError(String),
}

impl From<rusqlite::Error> for DbError {
    fn from(err: rusqlite::Error) -> Self {
        DbError::SqlError(err.to_string())
    }
}

#[derive(uniffi::Object)]
pub struct DeviceDatabase {
    conn: Arc<Mutex<Connection>>,
}

#[uniffi::export]
impl DeviceDatabase {
    #[uniffi::constructor]
    pub fn open(path: String) -> Result<Self, DbError> {
        let conn = Connection::open(&path)?;

        // Enable WAL mode for better concurrency
        conn.pragma_update(None, "journal_mode", &"WAL")?;

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY NOT NULL,
                value TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn get(&self, key: String) -> Result<Option<String>, DbError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?")?;
        let result = stmt.query_row(params![key], |row| row.get::<_, String>(0));
        match result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::SqlError(e.to_string())),
        }
    }

    pub fn set(&self, key: String, value: String) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn delete(&self, key: String) -> Result<(), DbError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM settings WHERE key = ?", params![key])?;
        Ok(())
    }

    pub fn get_device_id(&self) -> Result<Option<String>, DbError> {
        self.get(DEVICE_ID_KEY.to_string())
    }

    pub fn set_device_id(&self, device_id: String) -> Result<(), DbError> {
        self.set(DEVICE_ID_KEY.to_string(), device_id)
    }

    pub fn get_bool(&self, key: String) -> Result<Option<bool>, DbError> {
        match self.get(key)? {
            Some(val) => match val.to_lowercase().as_str() {
               "true" | "1" => Ok(Some(true)),
               "false" | "0" => Ok(Some(false)),
                _ => Ok(Some(false)), // или вернуть ошибку, если формат поврежден
            },
            None => Ok(None),
        }
    }

    pub fn set_bool(&self, key: String, value: bool) -> Result<(), DbError> {
        let val_str = if value { "true" } else { "false" };
        self.set(key, val_str.to_string())
    }
}
