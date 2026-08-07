//! [`Test`]: `run_chisq`/`run_mcpi`/... directly on any urng generator
//! (`urng::Rng` whose `Word` is `u32` or `u64`), feature `urng`.
//!
//! To register a `urng` generator on a `Suite` instead (batching several
//! generators under one shared config), use `Suite::from_source` with the
//! [`crate::Urng32`]/[`crate::Urng64`] wrapper from [`crate::source`].
//!
//! ```
//! use cribler::Test;
//! use urng::Sfc32;
//!
//! let mut rng = Sfc32::new(1);
//! let result = rng.run_chisq("sfc32").unwrap();
//! ```
//!
//! `cribler` cannot be a dependency of `urng` and simultaneously have `urng`
//! as a dependency of itself (Cargo rejects that as a cyclic package
//! dependency the moment both are used together). Since `urng` no longer
//! depends on `cribler` at all, this module can safely depend on `urng`
//! (via this optional `urng` feature) to provide the same ergonomics `urng`
//! used to bundle itself.

use urng::{Rng, Seed, Word};

use crate::{
    BirthdayConfig, BirthdayError, BirthdayResult, ChiSqConfig, ChiSqError, ChiSqResult, KsConfig,
    KsError, KsResult, McPiConfig, McPiError, McPiResult, NistConfig, NistError, NistResult,
    RunsConfig, RunsError, RunsResult, SerialConfig, SerialError, SerialResult, run_birthday,
    run_chisq, run_ks, run_mcpi, run_nist, run_runs, run_serial, unit_f64_from_u32,
    unit_f64_from_u64,
};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

/// Bridges a urng output [`Word`] to the draws the cribler engines consume:
/// `[0, 1)` sampling via [`unit_f64_from_u32`]/[`unit_f64_from_u64`] (matching
/// the word width), and the raw word widened to `u64` for the bit-level
/// engines ([`run_birthday`]/[`run_nist`]).
///
/// Sealed: implemented only for `u32`/`u64` inside `cribler`, so it cannot be
/// added to other types — it exists purely to let the single [`Test`] trait
/// span both urng word widths.
pub trait TestWord: Word + sealed::Sealed {
    #[doc(hidden)]
    fn to_unit_f64(self) -> f64;
    #[doc(hidden)]
    fn to_u64(self) -> u64;
}

impl TestWord for u32 {
    #[inline]
    fn to_unit_f64(self) -> f64 {
        unit_f64_from_u32(self)
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self as u64
    }
}

impl TestWord for u64 {
    #[inline]
    fn to_unit_f64(self) -> f64 {
        unit_f64_from_u64(self)
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self
    }
}

/// Adds `run_chisq`/`run_mcpi`/`run_serial`/`run_runs`/`run_ks`/`run_birthday`/
/// `run_nist` directly to any urng generator, so `rng.run_chisq("name")` works
/// without going through a `Suite` at all.
///
/// Blanket-implemented for every `urng::Rng` whose `Word` is `u32` or `u64`
/// (i.e. every urng generator) that is also `urng::Seed`. The word width is
/// read from `Word::BITS`, so 32- and 64-bit generators share one trait; the
/// `[0, 1)` conversion and the NIST word width follow the generator's own word
/// automatically.
pub trait Test: Rng + Seed + Sized
where
    Self::Word: TestWord,
{
    fn run_chisq(&mut self, name: impl Into<String>) -> Result<ChiSqResult, ChiSqError> {
        let mut sampler = || self.nextu().to_unit_f64();
        run_chisq(name, &mut sampler, ChiSqConfig::default())
    }

    fn run_mcpi(&mut self, name: impl Into<String>) -> Result<McPiResult, McPiError> {
        let mut sampler = || self.nextu().to_unit_f64();
        run_mcpi(name, &mut sampler, McPiConfig::default())
    }

    fn run_serial(&mut self, name: impl Into<String>) -> Result<SerialResult, SerialError> {
        let mut sampler = || self.nextu().to_unit_f64();
        run_serial(name, &mut sampler, SerialConfig::default())
    }

    fn run_runs(&mut self, name: impl Into<String>) -> Result<RunsResult, RunsError> {
        let mut sampler = || self.nextu().to_unit_f64();
        run_runs(name, &mut sampler, RunsConfig::default())
    }

    fn run_ks(&mut self, name: impl Into<String>) -> Result<KsResult, KsError> {
        let mut sampler = || self.nextu().to_unit_f64();
        run_ks(name, &mut sampler, KsConfig::default())
    }

    fn run_birthday(&mut self, name: impl Into<String>) -> Result<BirthdayResult, BirthdayError> {
        let mut sampler = || self.nextu().to_u64();
        run_birthday(name, &mut sampler, BirthdayConfig::default())
    }

    fn run_nist(&mut self, name: impl Into<String>) -> Result<NistResult, NistError> {
        let word_bits = std::mem::size_of::<<Self::Word as Word>::Int>() * 8;
        let mut sampler = || self.nextu().to_u64();
        run_nist(name, &mut sampler, word_bits as u32, NistConfig::default())
    }
}

impl<T: Rng + Seed> Test for T where T::Word: TestWord {}

#[cfg(test)]
mod tests {
    use super::*;
    use urng::{Sfc32, Sfc64};

    #[test]
    fn test32_run_chisq_works_directly_on_rng() {
        let mut rng = Sfc32::new(0);
        let result = rng.run_chisq("Sfc32").unwrap();
        assert_eq!(result.verdict, crate::Verdict::Pass);
    }

    #[test]
    fn test64_run_chisq_works_directly_on_rng() {
        let mut rng = Sfc64::new(0);
        let result = rng.run_chisq("Sfc64").unwrap();
        assert_eq!(result.verdict, crate::Verdict::Pass);
    }

    #[test]
    fn test32_nist_and_birthday_work_directly_on_rng() {
        let mut rng = Sfc32::new(1);
        assert_eq!(rng.run_nist("Sfc32").unwrap().verdict, crate::Verdict::Pass);
        assert!(rng.run_birthday("Sfc32").is_ok());
    }
}
