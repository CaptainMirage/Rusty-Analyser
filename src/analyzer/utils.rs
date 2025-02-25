use chrono::{DateTime, TimeZone, Utc};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::DATE_FORMAT;

// helper function to convert system time to formatted string
pub fn system_time_to_string(system_time: SystemTime) -> String {
    let datetime: DateTime<Utc> = system_time
        .duration_since(UNIX_EPOCH)
        .map(|duration| Utc.timestamp_opt(duration.as_secs() as i64, 0).unwrap())
        .unwrap_or_else(|_| Utc::now());
    datetime.format(DATE_FORMAT).to_string()
}