//! [`McPiSuite`]: named-case batching over [`cribler_core::run_mcpi`].

use cribler_core::{McPiConfig, McPiError, McPiResult, run_mcpi};

crate::suite::impl_f64_suite!(
    McPiSuite,
    McPiConfig,
    McPiError,
    McPiResult,
    run_mcpi,
    EmptyCaseName,
    DuplicateCaseName
);
