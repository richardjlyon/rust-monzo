// clippy lint unwrap
#![warn(clippy::unwrap_used)]
// unused code warn
// #![warn(clippy::unused)]
#![warn(clippy::pedantic)]
// ban unsafe
#![forbid(unsafe_code)]

pub mod beancount;
pub mod cli;
pub mod client;
pub mod configuration;
pub mod error;
pub mod model;
pub mod routes;
pub mod telemetry;
pub mod tests;

#[cfg(test)]
mod test {

    use chrono::DateTime;
    use chrono_intervals::{Grouping, IntervalGenerator};

    #[test]
    fn monthly_date_pairs() {
        let begin = DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z").unwrap();
        let end = DateTime::parse_from_rfc3339("2024-06-11T09:31:12.000000Z").unwrap();

        let monthly_intervals = IntervalGenerator::new()
            .with_grouping(Grouping::PerMonth)
            .get_intervals(begin, end);

        println!("{:?}", monthly_intervals);
    }
}
