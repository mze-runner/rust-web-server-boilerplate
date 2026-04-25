use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn encode(created_at: DateTime<Utc>, id: Uuid) -> String {
    let raw = format!("{}:{}", created_at.timestamp_micros(), id);
    URL_SAFE_NO_PAD.encode(raw.as_bytes())
}

pub fn decode(s: &str) -> Option<(DateTime<Utc>, Uuid)> {
    let bytes = URL_SAFE_NO_PAD.decode(s).ok()?;
    let raw = std::str::from_utf8(&bytes).ok()?;
    let (ts_part, id_part) = raw.split_once(':')?;
    let micros: i64 = ts_part.parse().ok()?;
    let id: Uuid = id_part.parse().ok()?;
    let created_at = DateTime::from_timestamp_micros(micros)?;
    Some((created_at, id))
}
