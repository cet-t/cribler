//! Named-case batching for [`cribler_core::run_paranoid`].

use std::collections::HashSet;

use cribler_core::{ParanoidConfig, ParanoidError, ParanoidResult, run_paranoid};

struct Case<'a> {
    name: String,
    trial: Box<dyn FnMut(usize) -> f64 + 'a>,
}

/// A suite that runs multiple named paranoid meta-test cases and collects their [`ParanoidResult`]s.
/// Bit-width-agnostic: cases are just p-value
/// producing closures, regardless of which underlying test they wrap.
#[derive(Default)]
pub struct ParanoidSuite<'a> {
    config: ParanoidConfig,
    cases: Vec<Case<'a>>,
}

impl<'a> ParanoidSuite<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ParanoidConfig) -> Result<Self, ParanoidError> {
        config.validate()?;
        Ok(Self {
            config,
            cases: Vec::new(),
        })
    }

    pub fn config(&self) -> ParanoidConfig {
        self.config
    }

    pub fn set_config(&mut self, config: ParanoidConfig) -> Result<(), ParanoidError> {
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

    /// Registers a named trial closure: `trial(i)` must return the p-value of
    /// the `i`-th independent run of the underlying test.
    pub fn add_trial(
        &mut self,
        name: impl Into<String>,
        trial: impl FnMut(usize) -> f64 + 'a,
    ) -> Result<&mut Self, ParanoidError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(ParanoidError::EmptyCaseName);
        }
        self.cases.push(Case {
            name,
            trial: Box::new(trial),
        });
        Ok(self)
    }

    pub fn run(&mut self) -> Result<Vec<ParanoidResult>, ParanoidError> {
        let mut seen = HashSet::with_capacity(self.cases.len());
        let mut out = Vec::with_capacity(self.cases.len());
        for case in &mut self.cases {
            if !seen.insert(case.name.clone()) {
                return Err(ParanoidError::DuplicateCaseName {
                    name: case.name.clone(),
                });
            }
            out.push(run_paranoid(
                case.name.clone(),
                self.config,
                case.trial.as_mut(),
            )?);
        }
        Ok(out)
    }
}
