//! Runs test engine: detects sequential correlation by analyzing the lengths
//! of monotonic ascending/descending runs in the output sequence.

use alloc::string::{String, ToString};

use thiserror::Error;

use crate::verdict::Verdict;

/// Configuration for a runs test.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunsConfig {
    /// Number of random samples drawn per test run.
    pub samples: usize,
    /// Maximum absolute z-score (`|z|`) permitted for a passing result.
    pub z_limit: f64,
}

impl Default for RunsConfig {
    /// Returns the default configuration: 1,000,000 samples, z-limit 3.0.
    fn default() -> Self {
        Self {
            samples: 1_000_000,
            z_limit: 3.0,
        }
    }
}

impl RunsConfig {
    /// Validates the configuration, returning a [`RunsError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), RunsError> {
        if self.samples < 2 {
            return Err(RunsError::InvalidSamples {
                samples: self.samples,
            });
        }
        if !self.z_limit.is_finite() || self.z_limit <= 0.0 {
            return Err(RunsError::InvalidZLimit {
                z_limit: self.z_limit,
            });
        }
        Ok(())
    }
}

/// Full result of a single runs test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RunsResult {
    /// Name of the test case.
    pub name: String,
    /// Number of samples drawn.
    pub samples: usize,
    /// Observed number of monotonic runs.
    pub runs: usize,
    /// Expected number of runs under independence: `(2n - 1) / 3`.
    pub expected_runs: f64,
    /// Normalized z-score of the observed run count.
    pub z_score: f64,
    /// z-score threshold used for the verdict.
    pub z_limit: f64,
    /// Final pass/fail verdict.
    pub verdict: Verdict,
}

/// Errors that can occur while configuring or running a runs test.
#[derive(Debug, Error)]
pub enum RunsError {
    /// Fewer than two samples were requested.
    #[error("samples must be at least 2: samples={samples}")]
    InvalidSamples { samples: usize },

    /// A non-positive or non-finite z-limit was supplied.
    #[error("z_limit must be finite and > 0: z_limit={z_limit}")]
    InvalidZLimit { z_limit: f64 },

    /// A test case name was empty or whitespace-only.
    #[error("test name must not be empty")]
    EmptyCaseName,
    /// Two cases in the same suite shared a name.
    #[error("duplicate test name in suite: {name}")]
    DuplicateCaseName { name: String },

    /// The generator produced a non-finite (`NaN`/`inf`) value.
    #[error(
        "sampler produced non-finite value: case={case}, sample_index={sample_index}, value={value}"
    )]
    NonFiniteSample {
        case: String,
        sample_index: usize,
        value: f64,
    },

    /// The generator produced a value outside `[0, 1)`.
    #[error(
        "sampler produced out-of-range value [0,1): case={case}, sample_index={sample_index}, value={value}"
    )]
    OutOfRangeSample {
        case: String,
        sample_index: usize,
        value: f64,
    },
}

/// Validates a test-case name, rejecting empty or whitespace-only names.
pub(crate) fn validate_case_name(name: String) -> Result<String, RunsError> {
    if name.trim().is_empty() {
        return Err(RunsError::EmptyCaseName);
    }
    Ok(name)
}

fn validate_sample(case: &str, sample_index: usize, x: f64) -> Result<(), RunsError> {
    if !x.is_finite() {
        return Err(RunsError::NonFiniteSample {
            case: case.to_string(),
            sample_index,
            value: x,
        });
    }
    if !(0.0..1.0).contains(&x) {
        return Err(RunsError::OutOfRangeSample {
            case: case.to_string(),
            sample_index,
            value: x,
        });
    }
    Ok(())
}

/// Runs a runs test (Wald-Wolfowitz-style, up/down variant) for the given
/// named sampler and configuration.
///
/// Counts the number of maximal monotonic ascending/descending runs in the
/// sequence and compares it to the closed-form expected value and variance
/// under independence (Knuth TAOCP Vol. 2, 3.3.2, eq. (1)-(2)):
/// `E[runs] = (2n - 1) / 3`, `Var[runs] = (16n - 29) / 90`.
pub fn run_runs(
    name: impl Into<String>,
    sampler: &mut dyn FnMut() -> f64,
    config: RunsConfig,
) -> Result<RunsResult, RunsError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;

    let n = config.samples;
    let mut prev = sampler();
    validate_sample(&name, 0, prev)?;

    let mut runs = 1usize;
    let mut ascending: Option<bool> = None;

    for sample_index in 1..n {
        let x = sampler();
        validate_sample(&name, sample_index, x)?;
        let is_ascending = x > prev;
        match ascending {
            Some(prev_dir) if prev_dir == is_ascending => {}
            _ => runs += 1,
        }
        ascending = Some(is_ascending);
        prev = x;
    }

    let n_f = n as f64;
    let expected_runs = (2.0 * n_f - 1.0) / 3.0;
    let variance = (16.0 * n_f - 29.0) / 90.0;
    let z_score = (runs as f64 - expected_runs) / variance.sqrt();
    let verdict = if z_score.abs() <= config.z_limit {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    Ok(RunsResult {
        name,
        samples: n,
        runs,
        expected_runs,
        z_score,
        z_limit: config.z_limit,
        verdict,
    })
}
