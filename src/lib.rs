#![doc = include_str!("../README.md")]
//!
//! # Module map
//! * [`suite`] / [`paranoid_suite`]: named-case batching (`XxxSuite`) over the [`cribler_core`] engines. [`suite::Suite`] aggregates every engine into one batch.
//! * [`source`]: [`source::WordSource`] (used by `from_custom` — integer-based, never float) and the [`source::UrngSeed32`]/[`source::UrngSeed64`]/[`source::RandSource`] traits that back `from_urng32`/`from_urng64`/`from_rand`.
//! * [`urng_support`] (feature `urng`): adds [`urng_support::Test`] (`rng.run_chisq(name)`, ...) directly to any `urng::Rng` generator (`Word = u32`/`u64`).

pub mod macros;
pub mod paranoid_suite;
#[cfg(feature = "rand")]
pub mod rand_support;
pub mod source;
pub mod suite;
#[cfg(feature = "urng")]
pub mod urng_support;

/// The crate-qualified type name of `T`, e.g. `"urng::Sfc32"` for `urng::rng::Sfc32`.
/// Used to auto-name cases registered via `Suite::from_urng32`/`from_urng64`/`from_rand`;
/// including the crate name keeps identically-named types from different crates
/// (e.g. `urng::Sfc32` vs. `rand_sfc::Sfc32`) from colliding as case keys.
#[allow(unused)]
pub(crate) fn short_type_name<T>() -> String {
    let full = std::any::type_name::<T>();
    let type_name = full.rsplit("::").next().unwrap_or(full);
    match full.split_once("::") {
        Some((cr, _)) if cr != type_name => format!("{cr}::{type_name}"),
        _ => type_name.to_string(),
    }
}

pub use cribler_core::*;
pub use paranoid_suite::ParanoidSuite;
#[cfg(feature = "rand")]
pub use rand_support::*;
#[cfg(feature = "rand")]
pub use source::RandSource;
#[cfg(feature = "urng")]
pub use source::UrngSeed;
pub use source::WordSource;
pub use suite::{
    BirthdaySuite, ChiSqSuite, KsSuite, McPiSuite, NistSuite, RunsSuite, SerialSuite, Suite,
    SuiteError, SuiteResults,
};
#[cfg(feature = "urng")]
pub use urng_support::*;
