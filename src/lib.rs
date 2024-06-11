// clippy lint unwrap
#![warn(clippy::unwrap_used)]
// unused code warn
// #![warn(clippy::unused)]
#![warn(clippy::pedantic)]
// ban unsafe
#![forbid(unsafe_code)]

pub mod cli;
pub mod client;
pub mod configuration;
pub mod error;
pub mod model;
pub mod routes;
pub mod telemetry;
pub mod tests;
