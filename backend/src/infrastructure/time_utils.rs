use chrono::{DateTime, Utc};
use time::OffsetDateTime;

pub fn offset_to_chrono(offset: OffsetDateTime) -> Option<DateTime<Utc>> {
    let unix_timestamp = offset.unix_timestamp();
    let nanos = offset.nanosecond();

    DateTime::from_timestamp(unix_timestamp, nanos)
}

pub fn chrono_to_offset(dt: DateTime<Utc>) -> Result<OffsetDateTime, time::error::ComponentRange> {
    let secs = dt.timestamp();
    let nanos = dt.timestamp_subsec_nanos();

    let offset_dt_sec = OffsetDateTime::from_unix_timestamp(secs);

    offset_dt_sec.and_then(|t| t.replace_nanosecond(nanos))
}
