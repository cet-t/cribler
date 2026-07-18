//! [`Test32`]/[`Test64`]: `run_chisq`/`run_mcpi`/... directly on any
//! `urng::Rng32`/`Rng64` generator (feature `urng`).
//!
//! To register a `urng` generator on a `Suite` instead (batching several
//! generators under one shared config), use `Suite::from_source` with the
//! [`crate::Urng32`]/[`crate::Urng64`] wrapper from [`crate::source`].
//!
//! ```
//! use cribler::Test32;
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

use urng::{Rng32, Rng64};

use crate::{
    BirthdayConfig, BirthdayError, BirthdayResult, ChiSqConfig, ChiSqError, ChiSqResult, KsConfig,
    KsError, KsResult, McPiConfig, McPiError, McPiResult, NistConfig, NistError, NistResult,
    RunsConfig, RunsError, RunsResult, SerialConfig, SerialError, SerialResult, run_birthday,
    run_chisq, run_ks, run_mcpi, run_nist, run_runs, run_serial, unit_f64_from_u32,
    unit_f64_from_u64,
};

/// Adds `run_chisq`/`run_mcpi`/`run_serial`/`run_runs`/`run_ks`/`run_birthday`/
/// `run_nist` directly to any `urng::Rng32`, so `rng.run_chisq("name")` works
/// without going through a `Suite` at all.
pub trait Test32: Rng32 + Sized {
    fn run_chisq(&mut self, name: impl Into<String>) -> Result<ChiSqResult, ChiSqError> {
        let mut sampler = || unit_f64_from_u32(self.nextu());
        run_chisq(name, &mut sampler, ChiSqConfig::default())
    }

    fn run_mcpi(&mut self, name: impl Into<String>) -> Result<McPiResult, McPiError> {
        let mut sampler = || unit_f64_from_u32(self.nextu());
        run_mcpi(name, &mut sampler, McPiConfig::default())
    }

    fn run_serial(&mut self, name: impl Into<String>) -> Result<SerialResult, SerialError> {
        let mut sampler = || unit_f64_from_u32(self.nextu());
        run_serial(name, &mut sampler, SerialConfig::default())
    }

    fn run_runs(&mut self, name: impl Into<String>) -> Result<RunsResult, RunsError> {
        let mut sampler = || unit_f64_from_u32(self.nextu());
        run_runs(name, &mut sampler, RunsConfig::default())
    }

    fn run_ks(&mut self, name: impl Into<String>) -> Result<KsResult, KsError> {
        let mut sampler = || unit_f64_from_u32(self.nextu());
        run_ks(name, &mut sampler, KsConfig::default())
    }

    fn run_birthday(&mut self, name: impl Into<String>) -> Result<BirthdayResult, BirthdayError> {
        let mut sampler = || self.nextu() as u64;
        run_birthday(name, &mut sampler, BirthdayConfig::default())
    }

    fn run_nist(&mut self, name: impl Into<String>) -> Result<NistResult, NistError> {
        let mut sampler = || self.nextu() as u64;
        run_nist(name, &mut sampler, 32, NistConfig::default())
    }
}

impl<T: Rng32> Test32 for T {}

/// Adds `run_chisq`/`run_mcpi`/`run_serial`/`run_runs`/`run_ks`/`run_birthday`/
/// `run_nist` directly to any `urng::Rng64`, so `rng.run_chisq("name")` works
/// without going through a `Suite` at all.
pub trait Test64: Rng64 + Sized {
    fn run_chisq(&mut self, name: impl Into<String>) -> Result<ChiSqResult, ChiSqError> {
        let mut sampler = || unit_f64_from_u64(self.nextu());
        run_chisq(name, &mut sampler, ChiSqConfig::default())
    }

    fn run_mcpi(&mut self, name: impl Into<String>) -> Result<McPiResult, McPiError> {
        let mut sampler = || unit_f64_from_u64(self.nextu());
        run_mcpi(name, &mut sampler, McPiConfig::default())
    }

    fn run_serial(&mut self, name: impl Into<String>) -> Result<SerialResult, SerialError> {
        let mut sampler = || unit_f64_from_u64(self.nextu());
        run_serial(name, &mut sampler, SerialConfig::default())
    }

    fn run_runs(&mut self, name: impl Into<String>) -> Result<RunsResult, RunsError> {
        let mut sampler = || unit_f64_from_u64(self.nextu());
        run_runs(name, &mut sampler, RunsConfig::default())
    }

    fn run_ks(&mut self, name: impl Into<String>) -> Result<KsResult, KsError> {
        let mut sampler = || unit_f64_from_u64(self.nextu());
        run_ks(name, &mut sampler, KsConfig::default())
    }

    fn run_birthday(&mut self, name: impl Into<String>) -> Result<BirthdayResult, BirthdayError> {
        let mut sampler = || self.nextu();
        run_birthday(name, &mut sampler, BirthdayConfig::default())
    }

    fn run_nist(&mut self, name: impl Into<String>) -> Result<NistResult, NistError> {
        let mut sampler = || self.nextu();
        run_nist(name, &mut sampler, 64, NistConfig::default())
    }
}

impl<T: Rng64> Test64 for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use urng::{Sfc32, Sfc64};

    #[test]
    fn test32_run_chisq_works_directly_on_rng() {
        let mut rng = Sfc32::new(0);
        let result = rng.run_chisq("Sfc32").unwrap();
        assert_eq!(result.verdict, crate::ChiSqVerdict::Pass);
    }

    #[test]
    fn test64_run_chisq_works_directly_on_rng() {
        let mut rng = Sfc64::new(0);
        let result = rng.run_chisq("Sfc64").unwrap();
        assert_eq!(result.verdict, crate::ChiSqVerdict::Pass);
    }

    #[test]
    fn test32_nist_and_birthday_work_directly_on_rng() {
        let mut rng = Sfc32::new(1);
        assert_eq!(
            rng.run_nist("Sfc32").unwrap().verdict,
            crate::NistVerdict::Pass
        );
        assert!(rng.run_birthday("Sfc32").is_ok());
    }
}
