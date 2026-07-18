//! [`SerialSuite`]: named-case batching over [`cribler_core::run_serial`].

use cribler_core::{SerialConfig, SerialError, SerialResult, run_serial};

crate::suite::impl_f64_suite!(
    SerialSuite,
    SerialConfig,
    SerialError,
    SerialResult,
    run_serial,
    EmptyCaseName,
    DuplicateCaseName
);
