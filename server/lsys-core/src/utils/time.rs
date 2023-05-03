use chrono::offset::Local;
use chrono::{DateTime, FixedOffset, NaiveDateTime, TimeZone};
use std::time::{SystemTime, SystemTimeError};
pub fn now_time() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub fn str_time(str_time: &str) -> Result<DateTime<FixedOffset>, String> {
    let dt =
        NaiveDateTime::parse_from_str(str_time, "%Y-%m-%d %H:%M:%S").map_err(|e| e.to_string())?;
    let ze = Local::now().timezone().offset_from_utc_datetime(&dt);
    match dt.and_local_timezone(ze) {
        chrono::LocalResult::Single(t) => Ok(t),
        _ => Err("parse time fail,on add zone".to_string()),
    }
}