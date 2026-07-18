//! Birthday spacing test engine (Marsaglia's DIEHARD "birthday spacings"
//! test): detects short periods and lattice structure by checking whether
//! collisions among sampled "birthdays" occur at the expected rate.

use thiserror::Error;

/// Configuration for a birthday spacing test.
///
/// Places `points` "birthdays" into a space of `2^bits_per_axis` possible
/// days per axis (a `dim`-dimensional grid), sorts the resulting values, and
/// counts collisions among consecutive spacings. Under the null hypothesis
/// the number of such spacing-collisions is approximately Poisson-distributed
/// with mean `lambda = points^3 / (4 * space)` (Marsaglia, 1985).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BirthdayConfig {
    /// Number of "birthday" points sampled per test run.
    pub points: usize,
    /// Dimensionality of the space each birthday is drawn from.
    pub dim: usize,
    /// Number of bits per axis (space size per axis is `2^bits_per_axis`).
    pub bits_per_axis: u32,
    /// Maximum absolute z-score (`|z|`) permitted for a passing result,
    /// computed against the Poisson mean/variance of collision counts.
    pub z_limit: f64,
}

impl Default for BirthdayConfig {
    /// Returns the default configuration: 4096 points, dim 2, 16 bits/axis, z-limit 3.0.
    ///
    /// `16` bits/axis (32-bit total space) keeps the default safe for word
    /// sources with as little as 32 bits of entropy per draw.
    fn default() -> Self {
        Self {
            points: 4096,
            dim: 2,
            bits_per_axis: 16,
            z_limit: 3.0,
        }
    }
}

impl BirthdayConfig {
    /// Validates the configuration, returning a [`BirthdayError`] describing the first problem found.
    pub fn validate(&self) -> Result<(), BirthdayError> {
        if self.points < 2 {
            return Err(BirthdayError::InvalidPoints {
                points: self.points,
            });
        }
        if self.dim == 0 {
            return Err(BirthdayError::InvalidDim { dim: self.dim });
        }
        if self.bits_per_axis == 0 || (self.bits_per_axis as u64) * (self.dim as u64) > 63 {
            return Err(BirthdayError::InvalidBitsPerAxis {
                bits_per_axis: self.bits_per_axis,
            });
        }
        if !self.z_limit.is_finite() || self.z_limit <= 0.0 {
            return Err(BirthdayError::InvalidZLimit {
                z_limit: self.z_limit,
            });
        }
        Ok(())
    }

    fn space_bits(&self) -> u32 {
        self.bits_per_axis * self.dim as u32
    }
}

/// Outcome of a single birthday spacing test run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BirthdayVerdict {
    /// The observed z-score was within the configured limit.
    Pass,
    /// The observed z-score exceeded the configured limit.
    Fail,
}

/// Full result of a single birthday spacing test run.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BirthdayResult {
    /// Name of the test case.
    pub name: String,
    /// Number of birthday points sampled.
    pub points: usize,
    /// Observed number of spacing collisions (equal consecutive spacings).
    pub collisions: usize,
    /// Expected number of collisions under independence: `points^3 / (4 * space)`.
    pub expected_collisions: f64,
    /// Normalized z-score of the observed collision count under a Poisson model.
    pub z_score: f64,
    /// z-score threshold used for the verdict.
    pub z_limit: f64,
    /// Final pass/fail verdict.
    pub verdict: BirthdayVerdict,
}

/// Errors that can occur while configuring or running a birthday spacing test.
#[derive(Debug, Error)]
pub enum BirthdayError {
    /// Fewer than two points were requested.
    #[error("points must be at least 2: points={points}")]
    InvalidPoints { points: usize },

    /// A zero dimension was requested.
    #[error("dim must be greater than zero: dim={dim}")]
    InvalidDim { dim: usize },

    /// An invalid bits-per-axis was requested (zero, or total space > 63 bits).
    #[error(
        "bits_per_axis must be > 0 and bits_per_axis * dim <= 63: bits_per_axis={bits_per_axis}"
    )]
    InvalidBitsPerAxis { bits_per_axis: u32 },

    /// A non-positive or non-finite z-limit was supplied.
    #[error("z_limit must be finite and > 0: z_limit={z_limit}")]
    InvalidZLimit { z_limit: f64 },

    /// A test case name was empty or whitespace-only.
    #[error("test name must not be empty")]
    EmptyCaseName,
    /// Two cases in the same suite shared a name.
    #[error("duplicate test name in suite: {name}")]
    DuplicateCaseName { name: String },
}

/// Validates a test-case name, rejecting empty or whitespace-only names.
pub(crate) fn validate_case_name(name: String) -> Result<String, BirthdayError> {
    if name.trim().is_empty() {
        return Err(BirthdayError::EmptyCaseName);
    }
    Ok(name)
}

/// Runs a birthday spacing test for the given named 64-bit-word sampler and configuration.
///
/// Draws `config.points` raw words, folds each into a `config.space_bits()`-bit
/// "birthday" value, sorts the resulting sequence, computes the spacings
/// between consecutive sorted birthdays, and counts how many spacing values
/// repeat (a "collision"). The collision count is compared against the
/// theoretical Poisson mean `points^3 / (4 * space)` (Marsaglia's birthday
/// spacings test, as used in DIEHARD).
pub fn run_birthday(
    name: impl Into<String>,
    sampler: &mut dyn FnMut() -> u64,
    config: BirthdayConfig,
) -> Result<BirthdayResult, BirthdayError> {
    config.validate()?;
    let name = validate_case_name(name.into())?;

    let space_bits = config.space_bits();
    let mask: u64 = if space_bits >= 64 {
        u64::MAX
    } else {
        (1u64 << space_bits) - 1
    };
    let space = if space_bits >= 64 {
        u64::MAX as f64 + 1.0
    } else {
        (1u64 << space_bits) as f64
    };

    let mut days: Vec<u64> = (0..config.points).map(|_| sampler() & mask).collect();
    days.sort_unstable();

    let mut spacings: Vec<u64> = days.windows(2).map(|w| w[1] - w[0]).collect();
    spacings.sort_unstable();

    let mut collisions = 0usize;
    for w in spacings.windows(2) {
        if w[0] == w[1] {
            collisions += 1;
        }
    }

    let n = config.points as f64;
    let expected_collisions = n.powi(3) / (4.0 * space);
    let variance = expected_collisions.max(f64::EPSILON);
    let z_score = (collisions as f64 - expected_collisions) / variance.sqrt();
    let verdict = if z_score.abs() <= config.z_limit {
        BirthdayVerdict::Pass
    } else {
        BirthdayVerdict::Fail
    };

    Ok(BirthdayResult {
        name,
        points: config.points,
        collisions,
        expected_collisions,
        z_score,
        z_limit: config.z_limit,
        verdict,
    })
}
