//! "Paranoid" meta-test engine: elevates any single statistical test into a
//! battery-level test, following the "Testing Strategy and Result
//! Interpretation" guidance of NIST SP 800-22 rev. 1a, 4.2.
//!
//! A single p-value can pass by chance even for a bad generator, and can
//! fail by chance even for a good one. 4.2 recommends running the test many
//! times over independent samples and checking two things about the
//! resulting collection of p-values:
//!
//! 1. **Proportion of passes** (4.2.1): the fraction of trials with
//!    `p_value >= trial_alpha` should fall within `p_hat +/- 3*sqrt(p_hat*(1-p_hat)/trials)`,
//!    where `p_hat = 1 - trial_alpha`.
//! 2. **Uniformity of p-values** (4.2.2): under the null hypothesis the
//!    p-values themselves should be uniformly distributed over `[0, 1)`.
//!    This is checked with a Kolmogorov-Smirnov test against `Uniform(0, 1)`.
//!
//! This module is generic over *how* a p-value is produced per trial: wrap a
//! call to any of the other engines in this crate (or any external test) in
//! a closure that returns a p-value, and [`run_paranoid`] applies the
//! meta-level checks.

use thiserror::Error;

use crate::internal::erfc;

/// Configuration for a paranoid (battery-level) meta-test.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParanoidConfig {
    /// Number of independent trials to run.
    pub trials: usize,
    /// Per-trial significance level: a trial "passes" when `p_value >= trial_alpha`.
    pub trial_alpha: f64,
    /// Significance level for the Kolmogorov-Smirnov uniformity check over the
    /// collected p-values (NIST recommends a small value, e.g. `0.0001`, to
    /// avoid an excessive false-positive rate at the meta-test level).
    pub uniformity_alpha: f64,
}

impl Default for ParanoidConfig {
    /// Returns the default configuration: 100 trials, trial alpha 0.01, uniformity alpha 0.0001.
    fn default() -> Self {
        Self {
            trials: 100,
            trial_alpha: 0.01,
            uniformity_alpha: 0.0001,
        }
    }
}

impl ParanoidConfig {
    /// Validates the configuration, returning a [`ParanoidError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), ParanoidError> {
        if self.trials < 2 {
            return Err(ParanoidError::InvalidTrials {
                trials: self.trials,
            });
        }
        if !self.trial_alpha.is_finite() || !(0.0..1.0).contains(&self.trial_alpha) {
            return Err(ParanoidError::InvalidTrialAlpha {
                trial_alpha: self.trial_alpha,
            });
        }
        if !self.uniformity_alpha.is_finite() || !(0.0..1.0).contains(&self.uniformity_alpha) {
            return Err(ParanoidError::InvalidUniformityAlpha {
                uniformity_alpha: self.uniformity_alpha,
            });
        }
        Ok(())
    }
}

/// Outcome of a single check within the paranoid meta-test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParanoidVerdict {
    /// The check was within tolerance.
    Pass,
    /// The check exceeded tolerance.
    Fail,
}

/// Full result of a paranoid meta-test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParanoidResult {
    /// Name of the test case.
    pub name: String,
    /// Number of trials run.
    pub trials: usize,
    /// Number of trials with `p_value >= trial_alpha`.
    pub passes: usize,
    /// Observed pass proportion (`passes / trials`).
    pub pass_proportion: f64,
    /// Lower bound of the acceptable pass-proportion range (NIST 4.2.1).
    pub proportion_lower: f64,
    /// Upper bound of the acceptable pass-proportion range (NIST 4.2.1).
    pub proportion_upper: f64,
    /// Verdict of the pass-proportion check.
    pub proportion_verdict: ParanoidVerdict,
    /// Kolmogorov-Smirnov `D` statistic of the p-value distribution against `Uniform(0, 1)`.
    pub ks_d_statistic: f64,
    /// Critical value for the KS uniformity check at `uniformity_alpha`.
    pub ks_critical_value: f64,
    /// Verdict of the p-value uniformity check.
    pub uniformity_verdict: ParanoidVerdict,
    /// Overall verdict: [`ParanoidVerdict::Pass`] iff both sub-checks passed.
    pub verdict: ParanoidVerdict,
}

/// Errors that can occur while configuring or running a paranoid meta-test.
#[derive(Debug, Error)]
pub enum ParanoidError {
    /// Fewer than two trials were requested.
    #[error("trials must be at least 2: trials={trials}")]
    InvalidTrials { trials: usize },

    /// An out-of-range or non-finite trial significance level was supplied.
    #[error("trial_alpha must be finite and in (0, 1): trial_alpha={trial_alpha}")]
    InvalidTrialAlpha { trial_alpha: f64 },

    /// An out-of-range or non-finite uniformity significance level was supplied.
    #[error("uniformity_alpha must be finite and in (0, 1): uniformity_alpha={uniformity_alpha}")]
    InvalidUniformityAlpha { uniformity_alpha: f64 },

    /// A produced p-value was non-finite or outside `[0, 1]`.
    #[error(
        "trial produced out-of-range p-value: case={case}, trial_index={trial_index}, value={value}"
    )]
    OutOfRangePValue {
        case: String,
        trial_index: usize,
        value: f64,
    },

    /// A test case name was empty or whitespace-only.
    #[error("test name must not be empty")]
    EmptyCaseName,

    /// Two cases in the same suite shared a name.
    #[error("duplicate test name in suite: {name}")]
    DuplicateCaseName { name: String },
}

/// Validates a test-case name, rejecting empty or whitespace-only names.
pub(crate) fn validate_case_name(name: String) -> Result<String, ParanoidError> {
    if name.trim().is_empty() {
        return Err(ParanoidError::EmptyCaseName);
    }
    Ok(name)
}

/// Converts a two-sided z-score (as produced by [`crate::chisq::ChiSqResult`],
/// [`crate::serial::SerialResult`], or [`crate::runs::RunsResult`]) into an
/// approximate p-value, suitable as a trial input to [`run_paranoid`].
pub fn p_value_from_z(z_score: f64) -> f64 {
    erfc(z_score.abs() / std::f64::consts::SQRT_2)
}

/// Runs a paranoid (battery-level) meta-test: calls `trial` once per configured
/// trial count, where `trial(i)` must return the p-value of the `i`-th
/// independent run of the underlying test, then applies the NIST SP 800-22
/// 4.2 proportion-of-passes and p-value-uniformity checks.
pub fn run_paranoid(
    name: impl Into<String>,
    config: ParanoidConfig,
    trial: &mut dyn FnMut(usize) -> f64,
) -> Result<ParanoidResult, ParanoidError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;

    let mut p_values = Vec::with_capacity(config.trials);
    for trial_index in 0..config.trials {
        let p = trial(trial_index);
        if !p.is_finite() || !(0.0..=1.0).contains(&p) {
            return Err(ParanoidError::OutOfRangePValue {
                case: name,
                trial_index,
                value: p,
            });
        }
        p_values.push(p);
    }

    let passes = p_values
        .iter()
        .filter(|&&p| p >= config.trial_alpha)
        .count();
    let pass_proportion = passes as f64 / config.trials as f64;

    let p_hat = 1.0 - config.trial_alpha;
    let bound = 3.0 * (p_hat * (1.0 - p_hat) / config.trials as f64).sqrt();
    let proportion_lower = (p_hat - bound).max(0.0);
    let proportion_upper = (p_hat + bound).min(1.0);
    let proportion_verdict = if (proportion_lower..=proportion_upper).contains(&pass_proportion) {
        ParanoidVerdict::Pass
    } else {
        ParanoidVerdict::Fail
    };

    let mut sorted = p_values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n_f = config.trials as f64;
    let mut ks_d_statistic = 0.0f64;
    for (i, &x) in sorted.iter().enumerate() {
        let i_f = (i + 1) as f64;
        let d_plus = i_f / n_f - x;
        let d_minus = x - (i_f - 1.0) / n_f;
        ks_d_statistic = ks_d_statistic.max(d_plus).max(d_minus);
    }
    let ks_critical_value = (-0.5 * (config.uniformity_alpha / 2.0).ln()).sqrt() / n_f.sqrt();
    let uniformity_verdict = if ks_d_statistic <= ks_critical_value {
        ParanoidVerdict::Pass
    } else {
        ParanoidVerdict::Fail
    };

    let verdict = if proportion_verdict == ParanoidVerdict::Pass
        && uniformity_verdict == ParanoidVerdict::Pass
    {
        ParanoidVerdict::Pass
    } else {
        ParanoidVerdict::Fail
    };

    Ok(ParanoidResult {
        name,
        trials: config.trials,
        passes,
        pass_proportion,
        proportion_lower,
        proportion_upper,
        proportion_verdict,
        ks_d_statistic,
        ks_critical_value,
        uniformity_verdict,
        verdict,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chisq::{ChiSqConfig, run_chisq};

    // A tiny xorshift32 PRNG, used only so the crate's own tests have no
    // external RNG dependency.
    struct Xorshift32(u32);
    impl Xorshift32 {
        fn next(&mut self) -> u32 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            self.0 = x;
            x
        }
    }

    #[test]
    fn paranoid_passes_for_good_generator() {
        let config = ParanoidConfig {
            trials: 30,
            ..ParanoidConfig::default()
        };
        let mut trial = |i: usize| {
            let mut rng = Xorshift32((i as u32 + 1).wrapping_mul(2654435761) | 1);
            let mut sampler = || rng.next() as f64 * (1.0 / 4294967296.0);
            let z = run_chisq(format!("trial-{i}"), &mut sampler, ChiSqConfig::default())
                .unwrap()
                .z_score;
            p_value_from_z(z)
        };
        let res = run_paranoid("xorshift32.paranoid_chisq", config, &mut trial).unwrap();
        assert_eq!(res.verdict, ParanoidVerdict::Pass);
    }

    #[test]
    fn paranoid_rejects_biased_generator() {
        let config = ParanoidConfig {
            trials: 20,
            ..ParanoidConfig::default()
        };
        let mut trial = |_i: usize| 0.0f64;
        let res = run_paranoid("always-fail", config, &mut trial).unwrap();
        assert_eq!(res.verdict, ParanoidVerdict::Fail);
    }
}
