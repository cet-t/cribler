#![doc = include_str!("../README.md")]
//!
//! # Module map
//! * [`chisq`]: chi-squared uniformity test.
//! * [`mcpi`]: Monte Carlo estimation of pi.
//! * [`serial`]: multi-dimensional chi-squared test over tuples of consecutive outputs.
//! * [`runs`]: detects sequential correlation via monotonic run lengths.
//! * [`ks`]: Kolmogorov-Smirnov test against the ideal uniform CDF.
//! * [`birthday`]: Marsaglia's DIEHARD birthday-spacings test.
//! * [`nist`]: NIST SP 800-22 bit-level test suite (5-test subset).
//! * [`paranoid`]: elevates any single test into a battery-level test (NIST SP 800-22 4.2).

pub mod birthday;
pub mod chisq;
pub mod internal;
pub mod ks;
pub mod mcpi;
pub mod nist;
pub mod paranoid;
pub mod runs;
pub mod serial;
pub mod units;
pub mod verdict;

pub use birthday::*;
pub use chisq::*;
pub use internal::{erfc, igamc, normal_cdf};
pub use ks::*;
pub use mcpi::*;
pub use nist::*;
pub use paranoid::*;
pub use runs::*;
pub use serial::*;
pub use units::*;
pub use verdict::*;
