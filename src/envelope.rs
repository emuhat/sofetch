use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::{Write};
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LastGood {
    pub fetched_at: DateTime<Utc>,
    pub expires: Option<DateTime<Utc>>,
    pub data: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StateEnvelope {
    pub status: String, // "ok" or "error"
    pub fetched_at: DateTime<Utc>, // time of last attempt
    pub error: Option<String>,
    pub last_good: Option<LastGood>,
}

/// Load envelope from disk. If it doesn't exist or is unreadable, return None.
pub fn load_envelope<P: AsRef<Path>>(path: P) -> Option<StateEnvelope> {
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Write an envelope atomically to a file.
pub fn write_envelope_atomic<P: AsRef<Path>>(
    path: P,
    envelope: &StateEnvelope
) -> std::io::Result<()> {

    let path = path.as_ref();
    let tmp_path = path.with_extension("tmp");

    // Serialize JSON
    let json = serde_json::to_vec_pretty(&envelope)?;

    // Write to temporary file
    {
        let mut tmp = fs::File::create(&tmp_path)?;
        tmp.write_all(&json)?;
        tmp.sync_all()?; // ensure it's flushed to disk
    }

    // Atomic rename into place
    fs::rename(&tmp_path, &path)?;

    Ok(())
}

/// Update envelope on successful fetch.
pub fn update_success<P: AsRef<Path>>(
    path: P,
    payload: Value,
    expires: Option<DateTime<Utc>>,
) -> std::io::Result<()> {

    let now = Utc::now();

    let new_last_good = LastGood {
        fetched_at: now,
        expires,
        data: payload,
    };

    let new_env = StateEnvelope {
        status: "ok".to_string(),
        fetched_at: now,
        error: None,
        last_good: Some(new_last_good),
    };

    write_envelope_atomic(path, &new_env)
}

/// Update envelope on failed fetch.
pub fn update_failure<P: AsRef<Path>>(
    path: P,
    error_message: impl Into<String>,
) -> std::io::Result<()> {

    let now = Utc::now();
    let old = load_envelope(&path);

    let new_env = StateEnvelope {
        status: "error".to_string(),
        fetched_at: now,
        error: Some(error_message.into()),
        last_good: old.and_then(|e| e.last_good), // preserve last good
    };

    write_envelope_atomic(path, &new_env)
}

