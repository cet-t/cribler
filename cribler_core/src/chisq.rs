//! Chi-squared uniformity test engine.

use alloc::{
    string::{String, ToString},
    vec,
};

use thiserror::Error;

use crate::verdict::Verdict;

/// Configuration for a chi-squared randomness test.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChiSqConfig {
    /// Number of random samples drawn per test run.
    pub samples: usize,
    /// Number of equal-width histogram bins over `[0, 1)`.
    pub bins: usize,
    /// Maximum absolute z-score (`|z|`) permitted for a passing result.
    pub z_limit: f64,
}

impl Default for ChiSqConfig {
    /// Returns the default configuration: 1,000,000 samples, 256 bins, z-limit 3.0.
    fn default() -> Self {
        Self {
            samples: 1_000_000,
            bins: 256,
            z_limit: 3.0,
        }
    }
}

impl ChiSqConfig {
    /// Validates the configuration, returning a [`ChiSqError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), ChiSqError> {
        if self.samples == 0 {
            return Err(ChiSqError::InvalidSamples {
                samples: self.samples,
            });
        }
        if self.bins < 2 {
            return Err(ChiSqError::InvalidBins { bins: self.bins });
        }
        if self.samples < self.bins {
            return Err(ChiSqError::SamplesLessThanBins {
                samples: self.samples,
                bins: self.bins,
            });
        }
        if !self.z_limit.is_finite() || self.z_limit <= 0.0 {
            return Err(ChiSqError::InvalidZLimit {
                z_limit: self.z_limit,
            });
        }
        Ok(())
    }
}

/// Full result of a single chi-squared test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChiSqResult {
    /// Name of the test case.
    pub name: String,
    /// Number of samples drawn.
    pub samples: usize,
    /// Number of histogram bins.
    pub bins: usize,
    /// Computed chi-squared statistic.
    pub chi2: f64,
    /// Degrees of freedom (`bins - 1`).
    pub df: f64,
    /// Normalized z-score: `(chi2 - df) / sqrt(2 * df)`.
    pub z_score: f64,
    /// z-score threshold used for the verdict.
    pub z_limit: f64,
    /// Final pass/fail verdict.
    pub verdict: Verdict,
}

/// Errors that can occur while configuring or running a chi-squared test.
#[derive(Debug, Error)]
pub enum ChiSqError {
    /// Zero samples were requested.
    #[error("samples must be greater than zero: samples={samples}")]
    InvalidSamples { samples: usize },

    /// Fewer than two bins were requested.
    #[error("bins must be at least 2: bins={bins}")]
    InvalidBins { bins: usize },

    /// Fewer samples than bins were requested (the test would be meaningless).
    #[error("samples must be >= bins: samples={samples}, bins={bins}")]
    SamplesLessThanBins { samples: usize, bins: usize },

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
pub(crate) fn validate_case_name(name: String) -> Result<String, ChiSqError> {
    if name.trim().is_empty() {
        return Err(ChiSqError::EmptyCaseName);
    }
    Ok(name)
}

fn validate_sample(case: &str, sample_index: usize, x: f64) -> Result<(), ChiSqError> {
    let case = case.to_string();

    if !x.is_finite() {
        return Err(ChiSqError::NonFiniteSample {
            case,
            sample_index,
            value: x,
        });
    }
    if !(0.0..1.0).contains(&x) {
        return Err(ChiSqError::OutOfRangeSample {
            case,
            sample_index,
            value: x,
        });
    }
    Ok(())
}

/// Runs a chi-squared uniformity test for the given named sampler and configuration.
///
/// Draws `config.samples` values from `sampler` (each expected in `[0, 1)`),
/// bins them into `config.bins` equal-width buckets, computes the
/// chi-squared statistic and its normalized z-score, and returns a
/// [`ChiSqResult`] carrying the [`Verdict`].
pub fn run_chisq(
    name: impl Into<String>,
    sampler: &mut dyn FnMut() -> f64,
    config: ChiSqConfig,
) -> Result<ChiSqResult, ChiSqError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;
    let mut counts = vec![0u64; config.bins];

    for sample_index in 0..config.samples {
        let x = sampler();
        validate_sample(&name, sample_index, x)?;
        let bin = ((x * config.bins as f64) as usize).min(config.bins - 1);
        counts[bin] += 1;
    }

    let expected = config.samples as f64 / config.bins as f64;
    let chi2 = counts
        .iter()
        .map(|&c| {
            let delta = c as f64 - expected;
            (delta * delta) / expected
        })
        .sum::<f64>();
    let df = (config.bins - 1) as f64;
    let z_score = (chi2 - df) / (2.0 * df).sqrt();
    let verdict = if z_score.abs() <= config.z_limit {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    Ok(ChiSqResult {
        name,
        samples: config.samples,
        bins: config.bins,
        chi2,
        df,
        z_score,
        z_limit: config.z_limit,
        verdict,
    })
}
