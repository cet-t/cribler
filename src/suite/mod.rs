//! Named-case batching (`XxxSuite`) over the [`cribler_core`] engines.
//!
//! Each suite lets you register several named samplers up front and run them
//! all under one shared `Config`, collecting one `Result` per case. One
//! module per engine; the float-sampler engines (`chisq`, `mcpi`, `serial`,
//! `runs`, `ks`) share the [`impl_f64_suite`] macro since their suite shape
//! is identical. `birthday` and `nist` take raw-word samplers instead and
//! are implemented directly.

macro_rules! impl_f64_suite {
    ($suite:ident, $config:ident, $error:ident, $result:ident, $run:path, $empty_err:ident, $dup_err:ident) => {
        struct Case<'a> {
            name: String,
            sampler: Box<dyn FnMut() -> f64 + 'a>,
        }

        #[doc = concat!("A suite that runs multiple named `", stringify!($run), "` cases under a shared config.")]
        pub struct $suite<'a> {
            #[allow(dead_code)]
            seed: u64,
            config: $config,
            cases: Vec<Case<'a>>,
        }

        impl<'a> Default for $suite<'a> {
            fn default() -> Self {
                Self {
                    seed: 0,
                    config: <$config>::default(),
                    cases: Vec::new(),
                }
            }
        }

        impl<'a> $suite<'a> {
            /// Creates an empty suite seeded with `seed` — used by
            /// `from_urng32`/`from_urng64`/`from_rand` to construct their
            /// generators.
            pub fn new(seed: u64) -> Self {
                Self {
                    seed,
                    ..Self::default()
                }
            }

            pub fn with_config(seed: u64, config: $config) -> Result<Self, $error> {
                config.validate()?;
                Ok(Self {
                    seed,
                    config,
                    cases: Vec::new(),
                })
            }

            pub fn config(&self) -> $config {
                self.config
            }

            pub fn set_config(&mut self, config: $config) -> Result<(), $error> {
                config.validate()?;
                self.config = config;
                Ok(())
            }

            pub fn len(&self) -> usize {
                self.cases.len()
            }

            pub fn is_empty(&self) -> bool {
                self.cases.is_empty()
            }

            pub fn clear(&mut self) {
                self.cases.clear();
            }

            /// Registers a named `[0, 1)` sampler closure as a test case.
            pub fn add_sampler<F>(
                &mut self,
                name: impl Into<String>,
                sampler: F,
            ) -> Result<&mut Self, $error>
            where
                F: FnMut() -> f64 + 'a,
            {
                let name = name.into();
                if name.trim().is_empty() {
                    return Err($error::$empty_err);
                }
                self.cases.push(Case {
                    name,
                    sampler: Box::new(sampler),
                });
                Ok(self)
            }

            /// Registers a `urng::Rng32` generator `R`, seeded from this
            /// suite's seed via [`crate::source::UrngSeed32`], as a named
            /// test case (case name = `R`'s type name). No instance to pass
            /// in; feature `urng`.
            #[cfg(feature = "urng")]
            pub fn from_urng32<R>(mut self) -> Result<Self, $error>
            where
                R: $crate::source::UrngSeed<u32> + 'a,
            {
                let mut rng = R::from_seed(self.seed as u32);
                self.add_sampler($crate::short_type_name::<R>(), move || {
                    $crate::unit_f64_from_u32(rng.nextu())
                })?;
                Ok(self)
            }

            /// Registers a `urng::Rng64` generator `R`, seeded from this
            /// suite's seed via [`crate::source::UrngSeed64`], as a named
            /// test case (case name = `R`'s type name). No instance to pass
            /// in; feature `urng`.
            #[cfg(feature = "urng")]
            pub fn from_urng64<R>(mut self) -> Result<Self, $error>
            where
                R: $crate::source::UrngSeed<u64> + 'a,
            {
                let mut rng = R::from_seed(self.seed);
                self.add_sampler($crate::short_type_name::<R>(), move || {
                    $crate::unit_f64_from_u64(rng.nextu())
                })?;
                Ok(self)
            }

            /// Registers a `urng_core::Rng` generator `R`, built by `make` from this
            /// suite's seed (truncated to `R::Word`), as a named test case (case name =
            /// `R`'s type name). `urng_core` has no seeding trait, so any generator
            /// works: `.from_urng_core(|seed| MyRng::new(seed))`; feature `urng_core`.
            #[cfg(feature = "urng_core")]
            pub fn from_urng_core<R, F>(mut self, make: F) -> Result<Self, $error>
            where
                R: urng_core::Rng + 'a,
                R::Word: crate::urng_core_support::CoreWord,
                F: Fn(R::Word) -> R,
            {
use crate::urng_core_support::CoreWord as _;
                let mut rng = make(<R::Word as crate::urng_core_support::CoreWord>::from_seed(self.seed));
                self.add_sampler($crate::short_type_name::<R>(), move || rng.nextu().to_unit_f64())?;
                Ok(self)
            }

            /// Registers a `rand_core::Rng` generator `R` seeded via
            /// `SeedableRng::seed_from_u64` from this suite's seed, as a
            /// named test case (case name = `R`'s type name). No instance to
            /// pass in; feature `rand`.
            #[cfg(feature = "rand")]
            pub fn from_rand<R>(mut self) -> Result<Self, $error>
            where
                R: $crate::source::RandSource + rand_core::SeedableRng + 'a,
            {
                let mut rng = R::seed_from_u64(self.seed);
                self.add_sampler($crate::short_type_name::<R>(), move || rng.sample_f64())?;
                Ok(self)
            }

            /// Registers a custom generator as a named test case. Custom
            /// generators are always integer-based: implement `From<YourType>
            /// for `[`crate::WordSource`]` directly (see [`crate::source`]);
            /// no wrapper needed. The raw `u64` draws are converted to
            /// `[0, 1)` via [`crate::unit_f64_from_u32`]/[`crate::unit_f64_from_u64`]
            /// depending on the source's `word_bits`.
            pub fn from_custom<S>(mut self, source: S) -> Result<Self, $error>
            where
                S: Into<$crate::WordSource<'a>>,
            {
                let source = source.into();
                let word_bits = source.word_bits;
                let mut sampler = source.sampler;
                self.add_sampler(source.name, move || {
                    let word = sampler();
                    if word_bits <= 32 {
                        $crate::unit_f64_from_u32(word as u32)
                    } else {
                        $crate::unit_f64_from_u64(word)
                    }
                })?;
                Ok(self)
            }

            pub fn run(&mut self) -> Result<hashbrown::HashMap<String, $result>, $error> {
                let mut out = hashbrown::HashMap::with_capacity(self.cases.len());
                for case in &mut self.cases {
                    if out.contains_key(&case.name) {
                        let err = $error::$dup_err {
                            name: case.name.clone(),
                        };
                        return Err(err);
                    }
                    let result = $run(case.name.clone(), case.sampler.as_mut(), self.config)?;
                    out.insert(case.name.clone(), result);
                }
                Ok(out)
            }
        }
    };
}
pub(crate) use impl_f64_suite;

pub mod all;
pub mod birthday;
pub mod chisq;
pub mod ks;
pub mod mcpi;
pub mod nist;
pub mod runs;
pub mod serial;

pub use all::{Suite, SuiteError, SuiteResults};
pub use birthday::BirthdaySuite;
pub use chisq::ChiSqSuite;
pub use ks::KsSuite;
pub use mcpi::McPiSuite;
pub use nist::NistSuite;
pub use runs::RunsSuite;
pub use serial::SerialSuite;
