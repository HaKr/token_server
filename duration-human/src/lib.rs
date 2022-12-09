#![deny(
    clippy::all,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::expect_used
)]

mod errors;
pub use errors::*;

mod display;
mod syn;

mod parser;
pub use parser::*;

mod validation;
pub use validation::*;

#[cfg(test)]
#[allow(clippy::unwrap_in_result, clippy::unwrap_used, clippy::expect_used)]
mod test;
