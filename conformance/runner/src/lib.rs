//! `markstone-conformance` provides the conformance test suite harness,
//! golden file generator, and language-agnostic byte-for-byte comparator runner.

pub mod cases;
pub mod diff;
pub mod generator;
pub mod mode;
pub mod runner;

pub use cases::{BUILTIN_CASES, CaseDefinition, TestCase, find_cases_dir, load_cases};
pub use diff::{ByteDiff, compare_bytes};
pub use generator::{GenerationSummary, generate_all_goldens};
pub use mode::Mode;
pub use runner::{CheckFailure, RunnerOptions, SuiteSummary, run_suite};
