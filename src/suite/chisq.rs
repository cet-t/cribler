//! [`ChiSqSuite`]: named-case batching over [`cribler_core::run_chisq`].

use cribler_core::{ChiSqConfig, ChiSqError, ChiSqResult, run_chisq};

crate::suite::impl_f64_suite!(
    ChiSqSuite,
    ChiSqConfig,
    ChiSqError,
    ChiSqResult,
    run_chisq,
    EmptyCaseName,
    DuplicateCaseName
);
