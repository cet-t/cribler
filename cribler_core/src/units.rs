//! Conversions from raw generator words to `[0, 1)` floats.
//!
//! These are the same constructions used by most RNG libraries (including
//! `urng` and `rand`) to derive uniform floats from integer output, exposed
//! here so callers can wire a bare `FnMut() -> u32`/`FnMut() -> u64` word
//! source into the float-based test engines ([`crate::chisq`], [`crate::mcpi`],
//! [`crate::serial`], [`crate::runs`], [`crate::ks`]) without depending on any
//! particular RNG crate.

const INV_2_POW_32: f64 = 1.0 / 4294967296.0;
const INV_2_POW_53: f64 = 1.0 / 9007199254740992.0;

/// Converts a raw 32-bit word into a uniform `[0, 1)` float.
#[inline(always)]
pub const fn unit_f64_from_u32(x: u32) -> f64 {
    x as f64 * INV_2_POW_32
}

/// Converts a raw 64-bit word into a uniform `[0, 1)` float, using the top 53 bits.
#[inline(always)]
pub const fn unit_f64_from_u64(x: u64) -> f64 {
    ((x >> 11) as f64) * INV_2_POW_53
}
