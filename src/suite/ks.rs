//! [`KsSuite`]: named-case batching over [`cribler_core::run_ks`].

use cribler_core::{KsConfig, KsError, KsResult, run_ks};

crate::suite::impl_f64_suite!(
    KsSuite,
    KsConfig,
    KsError,
    KsResult,
    run_ks,
    EmptyCaseName,
    DuplicateCaseName
);
