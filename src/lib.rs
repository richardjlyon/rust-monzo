// clippy lint unwrap
#![warn(clippy::unwrap_used)]
// unused code warn
// #![warn(clippy::unused)]
#![warn(clippy::pedantic)]
// ban unsafe
#![forbid(unsafe_code)]

use chrono::{NaiveDateTime, TimeDelta};

pub mod cli;
pub mod client;
pub mod configuration;
pub mod error;
pub mod model;
pub mod routes;
pub mod telemetry;
pub mod tests;

/// Utility function to generate date ranges for paged requests
pub fn date_ranges(
    start: NaiveDateTime,
    end: NaiveDateTime,
    days: i64,
) -> Vec<(NaiveDateTime, NaiveDateTime)> {
    let mut ranges = Vec::new();
    let mut current = start;

    while current < (end - TimeDelta::days(days)) {
        let next = current + TimeDelta::days(days);
        ranges.push((current, next));
        current = next;
    }

    ranges.push((current, end));

    ranges
}

#[cfg(test)]
mod test {
    use super::*;
    use chrono::NaiveDateTime;

    #[test]
    fn test_date_range() {
        let start =
            NaiveDateTime::parse_from_str("2024-04-01 12:23:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let end =
            NaiveDateTime::parse_from_str("2024-05-21 12:23:00", "%Y-%m-%d %H:%M:%S").unwrap();
        let ranges = date_ranges(start, end, 30);

        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].0, start);
        assert_eq!(ranges[1].1, end);
    }
}
