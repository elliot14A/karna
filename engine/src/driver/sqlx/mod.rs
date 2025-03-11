use chrono::{DateTime, NaiveDateTime, Utc};

use crate::error::Error;

pub mod driver;

fn parse_datetime_string(s: &str) -> Result<DateTime<Utc>, Error> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|_| {
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
                .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        })
        .map_err(|e| Error::DateTimeParse {
            value: s.to_string(),
            source: e,
        })
}
