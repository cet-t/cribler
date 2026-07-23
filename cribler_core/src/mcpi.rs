//! Monte Carlo pi-estimation test engine.

use std::f64;

use thiserror::Error;

use crate::verdict::Verdict;

/// Configuration for a Monte Carlo pi-estimation test.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct McPiConfig {
    /// Number of random `(x, y)` point pairs drawn per test run.
    pub pairs: usize,
    /// Maximum allowed relative error (in percent) from the true value of pi.
    pub max_error_pct: f64,
}

impl Default for McPiConfig {
    /// Returns the default configuration: 1,000,000 point pairs, max error 0.1%.
    fn default() -> Self {
        Self {
            pairs: 1_000_000,
            max_error_pct: 0.1,
        }
    }
}

impl McPiConfig {
    /// Validates the configuration, returning a [`McPiError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), McPiError> {
        if self.pairs == 0 {
            return Err(McPiError::InvalidPairs { pairs: self.pairs });
        }
        if !self.max_error_pct.is_finite() || self.max_error_pct <= 0.0 {
            return Err(McPiError::InvalidMaxErrorPct {
                max_error_pct: self.max_error_pct,
            });
        }
        Ok(())
    }
}

/// Full result of a single Monte Carlo pi-estimation test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct McPiResult {
    /// Name of the test case.
    pub name: String,
    /// Number of point pairs sampled.
    pub pairs: usize,
    /// Number of points that fell inside the unit quarter-circle.
    pub inside_circle: u64,
    /// Estimated value of pi (`4 * inside / pairs`).
    pub pi_estimate: f64,
    /// Absolute error `|estimate - pi|`.
    pub absolute_error: f64,
    /// Relative error as a percentage of pi.
    pub error_pct: f64,
    /// Relative-error threshold used for the verdict.
    pub max_error_pct: f64,
    /// Final pass/fail verdict.
    pub verdict: Verdict,
}

/// Errors that can occur while configuring or running a Monte Carlo pi test.
#[derive(Debug, Error)]
pub enum McPiError {
    /// Zero point pairs were requested.
    #[error("pairs must be greater than zero: pairs={pairs}")]
    InvalidPairs { pairs: usize },

    /// A non-positive or non-finite `max_error_pct` was supplied.
    #[error("max_error_pct must be finite and > 0: max_error_pct={max_error_pct}")]
    InvalidMaxErrorPct { max_error_pct: f64 },

    /// A test case name was empty or whitespace-only.
    #[error("test name must not be empty")]
    EmptyCaseName,
    /// Two cases in the same suite shared a name.
    #[error("duplicate test name in suite: {name}")]
    DuplicateCaseName { name: String },

    /// The generator produced a non-finite (`NaN`/`inf`) value.
    #[error(
        "sampler produced non-finite value: case={case}, sample_index={sample_index}, axis={axis}, value={value}"
    )]
    NonFiniteSample {
        case: String,
        sample_index: usize,
        axis: &'static str,
        value: f64,
    },

    /// The generator produced a value outside `[0, 1)`.
    #[error(
        "sampler produced out-of-range value [0,1): case={case}, sample_index={sample_index}, axis={axis}, value={value}"
    )]
    OutOfRangeSample {
        case: String,
        sample_index: usize,
        axis: &'static str,
        value: f64,
    },
}

/// Validates a test-case name, rejecting empty or whitespace-only names.
pub(crate) fn validate_case_name(name: String) -> Result<String, McPiError> {
    if name.trim().is_empty() {
        return Err(McPiError::EmptyCaseName);
    }
    Ok(name)
}

fn validate_sample(
    case: &str,
    sample_index: usize,
    axis: &'static str,
    x: f64,
) -> Result<(), McPiError> {
    if !x.is_finite() {
        return Err(McPiError::NonFiniteSample {
            case: case.to_string(),
            sample_index,
            axis,
            value: x,
        });
    }
    if !(0.0..1.0).contains(&x) {
        return Err(McPiError::OutOfRangeSample {
            case: case.to_string(),
            sample_index,
            axis,
            value: x,
        });
    }
    Ok(())
}

/// Runs a Monte Carlo pi-estimation test for the given named sampler and configuration.
///
/// Samples `config.pairs` points uniformly in `[0, 1)^2` (two draws from
/// `sampler` per point), counts how many fall inside the unit quarter-circle,
/// estimates pi as `4 * inside / pairs`, and returns a [`McPiResult`]
/// carrying the [`Verdict`].
pub fn run_mcpi(
    name: impl Into<String>,
    sampler: &mut dyn FnMut() -> f64,
    config: McPiConfig,
) -> Result<McPiResult, McPiError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;

    let mut inside_circle = 0u64;
    for sample_index in 0..config.pairs {
        let x = sampler();
        validate_sample(&name, sample_index, "x", x)?;
        let y = sampler();
        validate_sample(&name, sample_index, "y", y)?;
        if x * x + y * y < 1.0 {
            inside_circle += 1;
        }
    }

    let pi_estimate = 4.0 * inside_circle as f64 / config.pairs as f64;
    let absolute_error = (pi_estimate - f64::consts::PI).abs();
    let error_pct = (absolute_error / f64::consts::PI) * 100.0;
    let verdict = if error_pct <= config.max_error_pct {
        Verdict::Pass
    } else {
        Verdict::Fail
    };

    Ok(McPiResult {
        name,
        pairs: config.pairs,
        inside_circle,
        pi_estimate,
        absolute_error,
        error_pct,
        max_error_pct: config.max_error_pct,
        verdict,
    })
}
