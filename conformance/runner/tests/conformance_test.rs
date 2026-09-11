use markstone_conformance::{Mode, RunnerOptions, find_cases_dir, run_suite};

#[test]
fn test_all_conformance_cases_byte_for_byte() {
    let cases_dir = find_cases_dir().expect("Failed to locate conformance/cases directory");
    let options = RunnerOptions {
        cases_dir,
        exec_cmd: None,
        mode: None,
        filter: None,
        verbose: false,
    };

    let summary = run_suite(&options).expect("Failed running conformance suite");
    assert_eq!(
        summary.failed, 0,
        "Expected 0 failures, got {} failures",
        summary.failed
    );
    assert!(
        summary.passed >= 50 * 4,
        "Expected at least 200 checks (50+ cases * 4 modes), but only {} passed",
        summary.passed
    );
}

#[test]
fn test_mode_properties() {
    for mode in Mode::ALL {
        assert_eq!(mode.as_str().parse::<Mode>(), Ok(mode));
        assert!(mode.filename().ends_with(".html") || mode.filename().ends_with(".json"));
    }
}
