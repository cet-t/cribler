//! [`NistSuite`]: named-case batching over [`cribler_core::run_nist`].

use hashbrown::HashMap;

use cribler_core::{NistConfig, NistError, NistResult, run_nist};

struct Case<'a> {
    name: String,
    sampler: Box<dyn FnMut() -> u64 + 'a>,
    word_bits: u32,
}

/// A suite that runs multiple named [`run_nist`] cases under a shared config.
#[derive(Default)]
pub struct NistSuite<'a> {
    #[allow(dead_code)]
    seed: u64,
    config: NistConfig,
    cases: Vec<Case<'a>>,
}

impl<'a> NistSuite<'a> {
    /// Creates an empty suite seeded with `seed` — used by
    /// `from_urng32`/`from_urng64`/`from_rand` to construct their
    /// generators.
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            ..Self::default()
        }
    }

    pub fn with_config(seed: u64, config: NistConfig) -> Result<Self, NistError> {
        config.validate()?;
        Ok(Self {
            seed,
            config,
            cases: Vec::new(),
        })
    }

    pub fn config(&self) -> NistConfig {
        self.config
    }

    pub fn set_config(&mut self, config: NistConfig) -> Result<(), NistError> {
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

    /// Registers a named raw-word sampler closure as a test case. `word_bits`
    /// is the number of low bits of each `sampler()` draw that carry entropy.
    pub fn add_sampler<F>(
        &mut self,
        name: impl Into<String>,
        word_bits: u32,
        sampler: F,
    ) -> Result<&mut Self, NistError>
    where
        F: FnMut() -> u64 + 'a,
    {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(NistError::EmptyCaseName);
        }
        self.cases.push(Case {
            name,
            sampler: Box::new(sampler),
            word_bits,
        });
        Ok(self)
    }

    /// Registers a `urng::Rng32` generator `R`, seeded from this suite's seed
    /// via [`crate::source::UrngSeed32`], as a named test case (case name =
    /// `R`'s type name; `word_bits` = 32). No instance to pass in; feature
    /// `urng`.
    #[cfg(feature = "urng")]
    pub fn from_urng32<R>(mut self) -> Result<Self, NistError>
    where
        R: crate::source::UrngSeed32 + 'a,
    {
        let mut rng = R::from_seed(self.seed as u32);
        self.add_sampler(crate::short_type_name::<R>(), 32, move || {
            rng.nextu() as u64
        })?;
        Ok(self)
    }

    /// Registers a `urng::Rng64` generator `R`, seeded from this suite's seed
    /// via [`crate::source::UrngSeed64`], as a named test case (case name =
    /// `R`'s type name; `word_bits` = 64). No instance to pass in; feature
    /// `urng`.
    #[cfg(feature = "urng")]
    pub fn from_urng64<R>(mut self) -> Result<Self, NistError>
    where
        R: crate::source::UrngSeed64 + 'a,
    {
        let mut rng = R::from_seed(self.seed);
        self.add_sampler(crate::short_type_name::<R>(), 64, move || rng.nextu())?;
        Ok(self)
    }

    /// Registers a `rand_core::Rng` generator `R` seeded via
    /// `SeedableRng::seed_from_u64` from this suite's seed, as a named test
    /// case (case name = `R`'s type name; `word_bits` = 64). No instance to
    /// pass in; feature `rand`.
    #[cfg(feature = "rand")]
    pub fn from_rand<R>(mut self) -> Result<Self, NistError>
    where
        R: crate::source::RandSource + rand_core::SeedableRng + 'a,
    {
        let mut rng = R::seed_from_u64(self.seed);
        self.add_sampler(crate::short_type_name::<R>(), 64, move || rng.sample_word())?;
        Ok(self)
    }

    /// Registers a custom generator as a named test case. Implement
    /// `From<YourType> for `[`crate::WordSource`]` directly; no wrapper
    /// needed. The source's `word_bits` is forwarded to [`run_nist`].
    pub fn from_custom<S>(mut self, source: S) -> Result<Self, NistError>
    where
        S: Into<crate::WordSource<'a>>,
    {
        let source = source.into();
        self.add_sampler(source.name, source.word_bits, source.sampler)?;
        Ok(self)
    }

    pub fn run(&mut self) -> Result<HashMap<String, NistResult>, NistError> {
        let mut out = HashMap::with_capacity(self.cases.len());
        for case in &mut self.cases {
            if out.contains_key(&case.name) {
                return Err(NistError::DuplicateCaseName {
                    name: case.name.clone(),
                });
            }
            let result = run_nist(
                case.name.clone(),
                case.sampler.as_mut(),
                case.word_bits,
                self.config,
            )?;
            out.insert(case.name.clone(), result);
        }
        Ok(out)
    }
}
