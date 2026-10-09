//! [`urng_core`] generators as suite sources (feature `urng_core`).
//!
//! `urng_core::Rng` has no seeding trait, so construct the generator yourself
//! and wrap it in [`UrngCore`]; it converts into a [`WordSource`], so it goes
//! through `from_custom`:
//!
//! ```
//! use cribler::{ChiSqSuite, UrngCore};
//!
//! struct Counter(u32);
//!
//! impl urng_core::Rng for Counter {
//!     type Word = u32;
//!     fn nextu(&mut self) -> u32 {
//!         self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
//!         self.0
//!     }
//! }
//!
//! let results = ChiSqSuite::new(1)
//!     .from_custom(UrngCore(Counter(1)))
//!     .unwrap()
//!     .run()
//!     .unwrap();
//! assert_eq!(results.len(), 1);
//! ```
//!
//! `Suite::from_custom` additionally requires `Clone`.
//!
//! To have the suite seed the generator instead, use rom_urng_core:
//! .from_urng_core(|seed| MyRng::new(seed)).

use crate::{
    BirthdayConfig, BirthdayError, BirthdayResult, ChiSqConfig, ChiSqError, ChiSqResult, KsConfig,
    KsError, KsResult, McPiConfig, McPiError, McPiResult, NistConfig, NistError, NistResult,
    RunsConfig, RunsError, RunsResult, SerialConfig, SerialError, SerialResult, WordSource,
    run_birthday, run_chisq, run_ks, run_mcpi, run_nist, run_runs, run_serial, unit_f64_from_u32,
    unit_f64_from_u64,
};

mod sealed {
    pub trait Sealed {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
}

/// Widens a `urng_core` output word to `u64` (sealed: `u32`/`u64` only).
pub trait CoreWord: urng_core::Word + sealed::Sealed {
    #[doc(hidden)]
    const BITS: u32;
    #[doc(hidden)]
    fn from_seed(seed: u64) -> Self;
    #[doc(hidden)]
    fn to_u64(self) -> u64;
    #[doc(hidden)]
    fn to_unit_f64(self) -> f64;
}

impl CoreWord for u32 {
    const BITS: u32 = 32;
    #[inline]
    fn from_seed(seed: u64) -> Self {
        seed as u32
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self as u64
    }
    #[inline]
    fn to_unit_f64(self) -> f64 {
        unit_f64_from_u32(self)
    }
}

impl CoreWord for u64 {
    const BITS: u32 = 64;
    #[inline]
    fn from_seed(seed: u64) -> Self {
        seed
    }
    #[inline]
    fn to_u64(self) -> u64 {
        self
    }
    #[inline]
    fn to_unit_f64(self) -> f64 {
        unit_f64_from_u64(self)
    }
}

/// Wraps a `urng_core::Rng` so it can be registered with `from_custom`.
/// The case name is the generator's type name.
#[derive(Clone, Copy, Debug)]
pub struct UrngCore<R>(pub R);

impl<'a, R> From<UrngCore<R>> for WordSource<'a>
where
    R: urng_core::Rng + 'a,
    R::Word: CoreWord,
{
    fn from(UrngCore(mut rng): UrngCore<R>) -> Self {
        WordSource::new(
            crate::short_type_name::<R>(),
            <R::Word as CoreWord>::BITS,
            move || rng.nextu().to_u64(),
        )
    }
}

/// Adds `run_chisq`/`run_mcpi`/`run_serial`/`run_runs`/`run_ks`/`run_birthday`/
/// `run_nist` directly to any `urng_core::Rng` (`Word = u32`/`u64`), so
/// `rng.run_chisq("name")` works without a `Suite`.
pub trait CoreTest: urng_core::Rng + Sized
where
    Self::Word: CoreWord,
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
        let mut sampler = || self.nextu().to_u64();
        run_nist(
            name,
            &mut sampler,
            <Self::Word as CoreWord>::BITS,
            NistConfig::default(),
        )
    }
}

impl<T: urng_core::Rng> CoreTest for T where T::Word: CoreWord {}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChiSqSuite, NistSuite, Suite, Verdict};

    #[derive(Clone)]
    struct Lcg64(u64);

    impl urng_core::Rng for Lcg64 {
        type Word = u64;
        fn nextu(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
    }

    #[derive(Clone)]
    struct Lcg32(u32);

    impl urng_core::Rng for Lcg32 {
        type Word = u32;
        fn nextu(&mut self) -> u32 {
            self.0 = self.0.wrapping_mul(1664525).wrapping_add(1013904223);
            self.0 >> 0
        }
    }

    #[test]
    fn from_urng_core32_and_64_are_seeded_from_the_suite() {
        let chisq = ChiSqSuite::new(1)
            .from_urng_core(|s| Lcg32(s))
            .unwrap()
            .from_urng_core(|s| Lcg64(s))
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(chisq.len(), 2);

        let nist = NistSuite::new(1)
            .from_urng_core(|s| Lcg64(s))
            .unwrap()
            .run()
            .unwrap();
        assert!(nist.contains_key("urng_core_support::tests::Lcg64") || nist.len() == 1);

        let all = Suite::new(1)
            .from_urng_core(|s| Lcg32(s))
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(all.len(), 1);
    }

    #[test]
    fn core_test_runs_directly_on_rng() {
        use super::CoreTest;
        let mut rng = Lcg64(1);
        assert_eq!(rng.run_chisq("Lcg64").unwrap().verdict, Verdict::Pass);
        assert!(rng.run_birthday("Lcg64").is_ok());
        assert!(rng.run_nist("Lcg64").is_ok());
        let mut rng = Lcg32(1);
        assert!(rng.run_mcpi("Lcg32").is_ok());
    }

    #[test]
    fn from_custom_accepts_urng_core() {
        let nist = NistSuite::new(1)
            .from_custom(UrngCore(Lcg64(1)))
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(nist.len(), 1);

        let all = Suite::new(1)
            .from_custom(UrngCore(Lcg64(1)))
            .unwrap()
            .run()
            .unwrap();
        assert_eq!(all.len(), 1);
    }
}
