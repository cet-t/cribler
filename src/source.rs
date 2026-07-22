//! Generator sources for `Suite::from_urng32`/`from_urng64`/`from_rand`/`from_custom`.
//!
//! Every `XxxSuite` is created with a seed (`Suite::new(seed)`) and has three
//! kinds of methods that register a named test case from a generator seeded
//! from it:
//!
//! * `from_urng32::<R>()`/`from_urng64::<R>()` — a `urng::Rng32`/`Rng64` `R`,
//!   constructed via [`UrngSeed32`]/[`UrngSeed64`] from the suite's seed
//!   (feature `urng`). No instance to pass in; the case name is taken from
//!   `R`'s type name.
//! * `from_rand::<R>()` — a `rand_core::Rng` `R`, constructed via
//!   `SeedableRng::seed_from_u64` from the suite's seed (feature `rand`),
//!   same auto-naming.
//! * `from_custom(source)` — anything else. Custom generators are always
//!   integer-based, never float-based: implement `From<YourType> for`
//!   [`WordSource`] directly and pass an instance through (the suite's seed
//!   isn't involved; construct your generator however you like before
//!   handing it over). On the `[0, 1)`-sampling engines, the raw `u64` draws
//!   are converted with [`crate::unit_f64_from_u32`]/[`crate::unit_f64_from_u64`]
//!   depending on `word_bits`:
//!
//! ```
//! use cribler::{ChiSqSuite, WordSource};
//!
//! struct MyRng(u64);
//!
//! impl<'a> From<MyRng> for WordSource<'a> {
//!     fn from(mut rng: MyRng) -> Self {
//!         WordSource::new("MyRng", 64, move || {
//!             rng.0 = rng.0.wrapping_mul(6364136223846793005).wrapping_add(1);
//!             rng.0
//!         })
//!     }
//! }
//!
//! let results = ChiSqSuite::new(1).from_custom(MyRng(1)).unwrap().run().unwrap();
//! assert!(results.contains_key("MyRng"));
//! ```
//!
//! [`UrngSeed32`]/[`UrngSeed64`] are blanket-implemented for every
//! `urng::Rng32`/`Rng64` that also implements `urng::Seed32`/`Seed64`
//! (construction goes through `urng::Seed32::from_seed`/`Seed64::from_seed`),
//! mirroring how [`RandSource`] is blanket-implemented for every
//! `rand_core::Rng`. Any generator carrying those `urng` traits works with
//! `from_urng32`/`from_urng64` out of the box.

/// A named raw-word sampler, ready to be registered on a `Suite` via
/// `.from_custom(source)`, where `source: impl Into<WordSource>`. `word_bits`
/// is the number of low bits of each draw that carry entropy (used by
/// [`crate::run_nist`]; ignored by [`crate::run_birthday`]).
pub struct WordSource<'a> {
    pub(crate) name: String,
    pub(crate) word_bits: u32,
    pub(crate) sampler: Box<dyn FnMut() -> u64 + 'a>,
}

impl<'a> WordSource<'a> {
    pub fn new(name: impl Into<String>, word_bits: u32, sampler: impl FnMut() -> u64 + 'a) -> Self {
        Self {
            name: name.into(),
            word_bits,
            sampler: Box::new(sampler),
        }
    }
}

/// A `urng::Rng32` that `Suite::from_urng32` knows how to build from a `u32`
/// seed (feature `urng`). Blanket-implemented for every `urng::Rng32` that is
/// also `urng::Seed32`.
#[cfg(feature = "urng")]
pub trait UrngSeed32: urng::Rng32 + Sized {
    fn from_seed(seed: u32) -> Self;
}

/// A `urng::Rng64` that `Suite::from_urng64` knows how to build from a `u64`
/// seed (feature `urng`). Blanket-implemented for every `urng::Rng64` that is
/// also `urng::Seed64`.
#[cfg(feature = "urng")]
pub trait UrngSeed64: urng::Rng64 + Sized {
    fn from_seed(seed: u64) -> Self;
}

#[cfg(feature = "urng")]
impl<R: urng::Rng32 + urng::Seed32> UrngSeed32 for R {
    fn from_seed(seed: u32) -> Self {
        <Self as urng::Seed32>::from_seed(seed)
    }
}

#[cfg(feature = "urng")]
impl<R: urng::Rng64 + urng::Seed64> UrngSeed64 for R {
    fn from_seed(seed: u64) -> Self {
        <Self as urng::Seed64>::from_seed(seed)
    }
}

/// A `rand_core::Rng` that `Suite::from_rand` samples from (feature `rand`).
/// Implemented for every `rand_core::Rng`; construction goes through
/// `rand_core::SeedableRng::seed_from_u64`.
#[cfg(feature = "rand")]
pub trait RandSource {
    fn sample_f64(&mut self) -> f64;
    fn sample_word(&mut self) -> u64;
}

#[cfg(feature = "rand")]
impl<R: rand_core::Rng> RandSource for R {
    fn sample_f64(&mut self) -> f64 {
        crate::unit_f64_from_u64(self.next_u64())
    }

    fn sample_word(&mut self) -> u64 {
        self.next_u64()
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "urng")]
    #[test]
    fn from_urng32_and_from_urng64_are_seeded_from_the_suite() {
        use crate::{ChiSqSuite, NistSuite};
        use urng::{Sfc32, Sfc64};

        let chisq = ChiSqSuite::new(1)
            .from_urng32::<Sfc32>()
            .unwrap()
            .from_urng64::<Sfc64>()
            .unwrap()
            .run()
            .unwrap();
        assert!(chisq.contains_key("Sfc32"));
        assert!(chisq.contains_key("Sfc64"));

        let nist = NistSuite::new(1)
            .from_urng32::<Sfc32>()
            .unwrap()
            .run()
            .unwrap();
        assert!(nist.contains_key("Sfc32"));
    }

    #[cfg(feature = "rand")]
    #[test]
    fn from_rand_is_seeded_from_the_suite() {
        use crate::ChiSqSuite;

        let results = ChiSqSuite::new(1)
            .from_rand::<rand::rngs::StdRng>()
            .unwrap()
            .run()
            .unwrap();
        assert!(results.contains_key("StdRng"));
    }

    #[test]
    fn from_custom_accepts_custom_type() {
        use crate::{ChiSqSuite, NistSuite, WordSource};

        struct MyRng(u64);

        impl<'a> From<MyRng> for WordSource<'a> {
            fn from(mut rng: MyRng) -> Self {
                WordSource::new("MyRng", 64, move || {
                    rng.0 = rng.0.wrapping_mul(6364136223846793005).wrapping_add(1);
                    rng.0
                })
            }
        }

        let results = ChiSqSuite::new(1)
            .from_custom(MyRng(1))
            .unwrap()
            .run()
            .unwrap();
        assert!(results.contains_key("MyRng"));

        let nist = NistSuite::new(1)
            .from_custom(MyRng(2))
            .unwrap()
            .run()
            .unwrap();
        assert!(nist.contains_key("MyRng"));
    }
}
