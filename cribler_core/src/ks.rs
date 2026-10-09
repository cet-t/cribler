//! Kolmogorov-Smirnov test engine: checks the maximum deviation between the
//! empirical CDF of the samples and the ideal uniform CDF.

use alloc::{
    string::{String, ToString},
    vec::Vec,
};

use thiserror::Error;

use crate::verdict::Verdict;

/// Configuration for a Kolmogorov-Smirnov uniformity test.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KsConfig {
    /// Number of random samples drawn per test run.
    pub samples: usize,
    /// Maximum allowed KS statistic `D`, expressed via the asymptotic
    /// significance level `alpha` (e.g. `0.01` for a 1% false-positive rate).
    pub alpha: f64,
}

impl Default for KsConfig {
    /// Returns the default configuration: 1,000,000 samples, alpha 0.01.
    fn default() -> Self {
        Self {
            samples: 1_000_000,
            alpha: 0.01,
        }
    }
}

impl KsConfig {
    /// Validates the configuration, returning a [`KsError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), KsError> {
        if self.samples == 0 {
            return Err(KsError::InvalidSamples {
                samples: self.samples,
            });
        }
        if !self.alpha.is_finite() || !(0.0..1.0).contains(&self.alpha) {
            return Err(KsError::InvalidAlpha { alpha: self.alpha });
        }
        Ok(())
    }

    /// Critical value for the two-sided one-sample KS statistic, using the
    /// standard asymptotic (Kolmogorov) approximation:
    /// `D_alpha = sqrt(-0.5 * ln(alpha / 2)) / sqrt(n)`.
    pub fn critical_value(&self) -> f64 {
        (-0.5 * (self.alpha / 2.0).ln()).sqrt() / (self.samples as f64).sqrt()
    }
}

/// Full result of a single Kolmogorov-Smirnov test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KsResult {
    /// Name of the test case.
    pub name: String,
    /// Number of samples drawn.
    pub samples: usize,
    /// Observed KS statistic: `max |F_empirical(x) - x|`.
    pub d_statistic: f64,
    /// Critical value at the configured significance level.
    pub critical_value: f64,
    /// Significance level used for the verdict.
    pub alpha: f64,
    /// Final pass/fail verdict.
    pub verdict: Verdict,
}

/// Errors that can occur while configuring or running a KS test.
#[derive(Debug, Error)]
pub enum KsError {
    /// Zero samples were requested.
    #[error("samples must be greater than zero: samples={samples}")]
    InvalidSamples { samples: usize },

    /// An out-of-range or non-finite significance level was supplied.
    #[error("alpha must be finite and in (0, 1): alpha={alpha}")]
    InvalidAlpha { alpha: f64 },

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
pub(crate) fn validate_case_name(name: String) -> Result<String, KsError> {
    if name.trim().is_empty() {
        return Err(KsError::EmptyCaseName);
    }
    Ok(name)
}

fn validate_sample(case: &str, sample_index: usize, x: f64) -> Result<(), KsError> {
    if !x.is_finite() {
        return Err(KsError::NonFiniteSample {
            case: case.to_string(),
            sample_index,
            value: x,
        });
    }
    if !(0.0..1.0).contains(&x) {
        return Err(KsError::OutOfRangeSample {
            case: case.to_string(),
            sample_index,
            value: x,
        });
    }
    Ok(())
}

/// Runs a one-sample Kolmogorov-Smirnov uniformity test for the given named
/// sampler and configuration.
///
/// Draws `config.samples` values, sorts them, and computes the maximum
/// deviation between the empirical CDF and the ideal `Uniform(0, 1)` CDF:
/// `D = max_i max(i/n - x_(i), x_(i) - (i-1)/n)`.
pub fn run_ks(
    name: impl Into<String>,
    sampler: &mut dyn FnMut() -> f64,
    config: KsConfig,
) -> Result<KsResult, KsError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;

    let n = config.samples;
    let mut xs = Vec::with_capacity(n);
    for sample_index in 0..n {
        let x = sampler();
        validate_sample(&name, sample_index, x)?;
        xs.push(x);
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let n_f = n as f64;
    let mut d_statistic = 0.0f64;
    for (i, &x) in xs.iter().enumerate() {
        let i_f = (i + 1) as f64;
        let d_plus = i_f / n_f - x;
        let d_minus = x - (i_f - 1.0) / n_f;
        d_statistic = d_statistic.max(d_plus).max(d_minus);
    }

    let critical_value = config.critical_value();
    let verdict = if d_statistic <= critical_value {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    Ok(KsResult {
        name,
        samples: n,
        d_statistic,
        critical_value,
        alpha: config.alpha,
        verdict,
    })
}
