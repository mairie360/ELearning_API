//! Ids travel as `u64` in the API and as `INT4` in Postgres: convert them with
//! the lib's `id_to_sql` / `id_from_sql`, never `as` (MAIR-422).
#![deny(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]

pub mod database;
pub mod endpoints;
pub mod logging;
pub mod rate_limit;
pub mod storage;
