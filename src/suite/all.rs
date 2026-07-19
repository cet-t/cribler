//! [`Suite`]: runs every engine (`ChiSq`, `McPi`, `Serial`, `Runs`, `Ks`,
//! `Birthday`, `Nist`) as one batch.

use hashbrown::HashMap;

use crate::{
    BirthdayConfig, BirthdayError, BirthdayResult, BirthdaySuite, ChiSqConfig, ChiSqError,
    ChiSqResult, ChiSqSuite, KsConfig, KsError, KsResult, KsSuite, McPiConfig, McPiError,
    McPiResult, McPiSuite, NistConfig, NistError, NistResult, NistSuite, RunsConfig, RunsError,
    RunsResult, RunsSuite, SerialConfig, SerialError, SerialResult, SerialSuite,
};

/// Every engine's result for one generator registered on a [`Suite`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SuiteResult {
    pub chisq: ChiSqResult,
    pub mcpi: McPiResult,
    pub serial: SerialResult,
    pub runs: RunsResult,
    pub ks: KsResult,
    pub birthday: BirthdayResult,
    pub nist: NistResult,
}

/// Aggregated results from a [`Suite`] run: one [`SuiteResult`] (all seven
/// engines) per registered generator, keyed by its case name.
pub type SuiteResults = HashMap<String, SuiteResult>;

/// An error from any single engine while building or running a [`Suite`].
#[derive(Debug, thiserror::Error)]
pub enum SuiteError {
    #[error("chisq: {0}")]
    ChiSq(#[from] ChiSqError),
    #[error("mcpi: {0}")]
    McPi(#[from] McPiError),
    #[error("serial: {0}")]
    Serial(#[from] SerialError),
    #[error("runs: {0}")]
    Runs(#[from] RunsError),
    #[error("ks: {0}")]
    Ks(#[from] KsError),
    #[error("birthday: {0}")]
    Birthday(#[from] BirthdayError),
    #[error("nist: {0}")]
    Nist(#[from] NistError),
}

/// Runs every engine over the same generator(s) as one batch. Each engine
/// gets its own independent generator instance seeded identically (via
/// `Suite::new(seed)`), so registering a generator once here runs the full
/// randomness-test battery against it in one call.
pub struct Suite<'a> {
    chisq: ChiSqSuite<'a>,
    mcpi: McPiSuite<'a>,
    serial: SerialSuite<'a>,
    runs: RunsSuite<'a>,
    ks: KsSuite<'a>,
    birthday: BirthdaySuite<'a>,
    nist: NistSuite<'a>,
}

impl<'a> Suite<'a> {
    /// Creates an empty suite seeded with `seed`, one sub-suite per engine.
    pub fn new(seed: u64) -> Self {
        Self {
            chisq: ChiSqSuite::new(seed),
            mcpi: McPiSuite::new(seed),
            serial: SerialSuite::new(seed),
            runs: RunsSuite::new(seed),
            ks: KsSuite::new(seed),
            birthday: BirthdaySuite::new(seed),
            nist: NistSuite::new(seed),
        }
    }

    /// Creates an empty suite seeded with `seed`, validating an explicit
    /// config per engine up front.
    #[allow(clippy::too_many_arguments)]
    pub fn with_config(
        seed: u64,
        chisq: ChiSqConfig,
        mcpi: McPiConfig,
        serial: SerialConfig,
        runs: RunsConfig,
        ks: KsConfig,
        birthday: BirthdayConfig,
        nist: NistConfig,
    ) -> Result<Self, SuiteError> {
        Ok(Self {
            chisq: ChiSqSuite::with_config(seed, chisq)?,
            mcpi: McPiSuite::with_config(seed, mcpi)?,
            serial: SerialSuite::with_config(seed, serial)?,
            runs: RunsSuite::with_config(seed, runs)?,
            ks: KsSuite::with_config(seed, ks)?,
            birthday: BirthdaySuite::with_config(seed, birthday)?,
            nist: NistSuite::with_config(seed, nist)?,
        })
    }

    /// Registers a `urng::Rng32` generator `R` on every engine, each from its
    /// own independent instance seeded from this suite's seed. No instance to
    /// pass in; feature `urng`.
    #[cfg(feature = "urng")]
    pub fn from_urng32<R>(mut self) -> Result<Self, SuiteError>
    where
        R: crate::source::UrngSeed32 + 'a,
    {
        self.chisq = self.chisq.from_urng32::<R>()?;
        self.mcpi = self.mcpi.from_urng32::<R>()?;
        self.serial = self.serial.from_urng32::<R>()?;
        self.runs = self.runs.from_urng32::<R>()?;
        self.ks = self.ks.from_urng32::<R>()?;
        self.birthday = self.birthday.from_urng32::<R>()?;
        self.nist = self.nist.from_urng32::<R>()?;
        Ok(self)
    }

    /// Registers a `urng::Rng64` generator `R` on every engine, each from its
    /// own independent instance seeded from this suite's seed. No instance to
    /// pass in; feature `urng`.
    #[cfg(feature = "urng")]
    pub fn from_urng64<R>(mut self) -> Result<Self, SuiteError>
    where
        R: crate::source::UrngSeed64 + 'a,
    {
        self.chisq = self.chisq.from_urng64::<R>()?;
        self.mcpi = self.mcpi.from_urng64::<R>()?;
        self.serial = self.serial.from_urng64::<R>()?;
        self.runs = self.runs.from_urng64::<R>()?;
        self.ks = self.ks.from_urng64::<R>()?;
        self.birthday = self.birthday.from_urng64::<R>()?;
        self.nist = self.nist.from_urng64::<R>()?;
        Ok(self)
    }

    /// Registers a `rand_core::Rng` generator `R` on every engine, each from
    /// its own independent instance seeded from this suite's seed via
    /// `SeedableRng::seed_from_u64`. No instance to pass in; feature `rand`.
    #[cfg(feature = "rand")]
    pub fn from_rand<R>(mut self) -> Result<Self, SuiteError>
    where
        R: crate::source::RandSource + rand_core::SeedableRng + 'a,
    {
        self.chisq = self.chisq.from_rand::<R>()?;
        self.mcpi = self.mcpi.from_rand::<R>()?;
        self.serial = self.serial.from_rand::<R>()?;
        self.runs = self.runs.from_rand::<R>()?;
        self.ks = self.ks.from_rand::<R>()?;
        self.birthday = self.birthday.from_rand::<R>()?;
        self.nist = self.nist.from_rand::<R>()?;
        Ok(self)
    }

    /// Registers a custom generator on every engine. Custom generators are
    /// always integer-based: implement `From<YourType> for `[`crate::WordSource`]`
    /// directly (see [`crate::source`]). Since each engine needs its own
    /// independent instance, `S` must be `Clone`.
    pub fn from_custom<S>(mut self, source: S) -> Result<Self, SuiteError>
    where
        S: Clone + Into<crate::WordSource<'a>>,
    {
        self.chisq = self.chisq.from_custom(source.clone())?;
        self.mcpi = self.mcpi.from_custom(source.clone())?;
        self.serial = self.serial.from_custom(source.clone())?;
        self.runs = self.runs.from_custom(source.clone())?;
        self.ks = self.ks.from_custom(source.clone())?;
        self.birthday = self.birthday.from_custom(source.clone())?;
        self.nist = self.nist.from_custom(source)?;
        Ok(self)
    }

    /// Runs every engine and collects one [`SuiteResult`] per registered
    /// generator (keyed by its case name). Every engine is registered with
    /// the same set of case names (`from_urng32`/`from_urng64`/`from_rand`/
    /// `from_custom` always add to all seven at once), so each name is
    /// present in every engine's result map.
    pub fn run(&mut self) -> Result<SuiteResults, SuiteError> {
        let mut chisq = self.chisq.run()?;
        let mut mcpi = self.mcpi.run()?;
        let mut serial = self.serial.run()?;
        let mut runs = self.runs.run()?;
        let mut ks = self.ks.run()?;
        let mut birthday = self.birthday.run()?;
        let mut nist = self.nist.run()?;

        let names: Vec<_> = chisq.keys().cloned().collect();
        let mut out = HashMap::with_capacity(names.len());
        for name in names {
            let result = SuiteResult {
                chisq: chisq
                    .remove(&name)
                    .expect("case registered on every engine"),
                mcpi: mcpi.remove(&name).expect("case registered on every engine"),
                serial: serial
                    .remove(&name)
                    .expect("case registered on every engine"),
                runs: runs.remove(&name).expect("case registered on every engine"),
                ks: ks.remove(&name).expect("case registered on every engine"),
                birthday: birthday
                    .remove(&name)
                    .expect("case registered on every engine"),
                nist: nist.remove(&name).expect("case registered on every engine"),
            };
            out.insert(name, result);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "urng")]
    #[test]
    fn suite_runs_every_engine_over_the_same_generator() {
        use crate::Suite;
        use urng::Sfc32;

        let results = Suite::new(1).from_urng32::<Sfc32>().unwrap().run().unwrap();
        let result = results.get("Sfc32").unwrap();
        assert_eq!(result.chisq.verdict, crate::ChiSqVerdict::Pass);
        assert_eq!(result.nist.verdict, crate::NistVerdict::Pass);
    }

    #[test]
    fn suite_from_custom_registers_on_every_engine() {
        use crate::{Suite, WordSource};

        #[derive(Clone)]
        struct MyRng(u64);

        impl<'a> From<MyRng> for WordSource<'a> {
            fn from(mut rng: MyRng) -> Self {
                WordSource::new("MyRng", 64, move || {
                    rng.0 = rng.0.wrapping_mul(6364136223846793005).wrapping_add(1);
                    rng.0
                })
            }
        }

        let results = Suite::new(1).from_custom(MyRng(1)).unwrap().run().unwrap();
        assert!(results.contains_key("MyRng"));
    }

    #[cfg(all(feature = "serde", feature = "urng"))]
    #[test]
    fn suite_results_roundtrip_through_json() {
        use crate::Suite;
        use urng::Sfc32;

        let results = Suite::new(1).from_urng32::<Sfc32>().unwrap().run().unwrap();
        let json = serde_json::to_string(&results).unwrap();
        let round_tripped: super::SuiteResults = serde_json::from_str(&json).unwrap();
        assert_eq!(
            results["Sfc32"].chisq.verdict,
            round_tripped["Sfc32"].chisq.verdict
        );
    }
}
