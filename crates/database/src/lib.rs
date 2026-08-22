#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]
#![allow(
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::cast_possible_wrap,
    clippy::cast_possible_truncation
)]

pub mod contests;
pub mod problems;
pub mod submissions;
pub mod tools;
pub mod users;
