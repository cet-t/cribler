//! Optional convenience bridges from any [`rand_core::Rng`] implementation (feature `rand`).
//!
//! To register a `rand_core`-backed generator on a `Suite` (batching several generators under one shared config),
//! use `Suite::from_source` with the [`crate::Rand`] wrapper from [`crate::source`].
//! The functions below are lower-level helpers for building a sampler closure by hand.

use cribler_core::{unit_f64_from_u32, unit_f64_from_u64};
use rand_core::Rng;

/// Builds a `[0, 1)` sampler closure from a `rand_core::Rng`,
/// using its full 64-bit output (`unit_f64_from_u64`) for maximum precision.
pub fn f64_sampler<'a, R: Rng + 'a>(rng: &'a mut R) -> impl FnMut() -> f64 + 'a {
    move || unit_f64_from_u64(rng.next_u64())
}

/// Builds a `[0, 1)` sampler closure from a `rand_core::Rng`,
/// using its 32-bit output (`unit_f64_from_u32`).
pub fn f64_sampler_32<'a, R: Rng + 'a>(rng: &'a mut R) -> impl FnMut() -> f64 + 'a {
    move || unit_f64_from_u32(rng.next_u32())
}

/// Builds a raw 64-bit word sampler closure from a `rand_core::Rng`,
/// for use with the word-level engines ([`cribler_core::run_birthday`], [`cribler_core::run_nist`]).
pub fn u64_sampler<'a, R: Rng + 'a>(rng: &'a mut R) -> impl FnMut() -> u64 + 'a {
    move || rng.next_u64()
}
