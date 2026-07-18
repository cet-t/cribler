//! [`RunsSuite`]: named-case batching over [`cribler_core::run_runs`].

use cribler_core::{RunsConfig, RunsError, RunsResult, run_runs};

crate::suite::impl_f64_suite!(
    RunsSuite,
    RunsConfig,
    RunsError,
    RunsResult,
    run_runs,
    EmptyCaseName,
    DuplicateCaseName
);
