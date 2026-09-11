//! Conformance test execution engine for internal and external bindings.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;

use crate::cases::{TestCase, load_cases};
use crate::diff::{ByteDiff, compare_bytes};
use crate::mode::Mode;

/// Configuration options for executing the conformance runner.
#[derive(Debug, Clone)]
pub struct RunnerOptions {
    /// Path to the directory containing conformance cases.
    pub cases_dir: PathBuf,
    /// Optional external executable command. If `None`, executes internal Rust crates directly.
    pub exec_cmd: Option<String>,
    /// Optional filter for mode. If `None`, all four modes are checked.
    pub mode: Option<Mode>,
    /// Optional substring filter for case directory names.
    pub filter: Option<String>,
    /// Whether to print progress for every individual test case.
    pub verbose: bool,
}

/// Description of a check failure.
#[derive(Debug, Clone)]
pub struct CheckFailure {
    pub case_name: String,
    pub mode: Mode,
    pub message: String,
    pub diff: Option<ByteDiff>,
}

/// Overall execution summary of the conformance test suite.
#[derive(Debug, Clone)]
pub struct SuiteSummary {
    pub total_cases: usize,
    pub total_checks: usize,
    pub passed: usize,
    pub failed: usize,
    pub failures: Vec<CheckFailure>,
}

/// Executes internal markstone Rust crates for a given case and mode.
fn execute_internal(case: &TestCase, mode: Mode) -> Result<Vec<u8>, String> {
    match mode {
        Mode::GenericHtml => markstone_core::to_html(&case.input)
            .map(|s| s.into_bytes())
            .map_err(|e| format!("core::to_html error: {e:?}")),
        Mode::GenericAst => markstone_core::to_ast(&case.input)
            .map(|s| s.into_bytes())
            .map_err(|e| format!("core::to_ast error: {e:?}")),
        Mode::ActosHtml => markstone_actos::to_html(&case.input)
            .map(|s| s.into_bytes())
            .map_err(|e| format!("actos::to_html error: {e:?}")),
        Mode::ActosAst => markstone_actos::to_ast(&case.input)
            .map(|s| s.into_bytes())
            .map_err(|e| format!("actos::to_ast error: {e:?}")),
    }
}

/// Executes an external binding command for a given case and mode.
fn execute_external(case: &TestCase, mode: Mode, cmd_template: &str) -> Result<Vec<u8>, String> {
    let input_file = case.dir.join("input.md");
    let input_str = input_file.to_string_lossy();
    let mode_str = mode.as_str();
    let filename_str = mode.filename();
    let case_dir_str = case.dir.to_string_lossy();
    let golden_file = case.dir.join(mode.filename());
    let golden_str = golden_file.to_string_lossy();

    let has_placeholders = cmd_template.contains("{mode}")
        || cmd_template.contains("{input}")
        || cmd_template.contains("{case_dir}")
        || cmd_template.contains("{filename}")
        || cmd_template.contains("{file}");

    let mut child = if has_placeholders {
        let full_cmd = cmd_template
            .replace("{mode}", mode_str)
            .replace("{input}", &input_str)
            .replace("{case_dir}", &case_dir_str)
            .replace("{filename}", filename_str)
            .replace("{file}", &golden_str);

        Command::new("/bin/sh")
            .arg("-c")
            .arg(full_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn shell command: {e}"))?
    } else {
        let parts: Vec<&str> = cmd_template.split_whitespace().collect();
        if parts.is_empty() {
            return Err("Empty command provided".to_string());
        }

        let program = parts[0];
        let mut cmd = Command::new(program);
        if parts.len() > 1 {
            cmd.args(&parts[1..]);
        }
        cmd.arg("--mode").arg(mode_str).arg(&*input_str);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        cmd.spawn()
            .map_err(|e| format!("Failed to spawn '{program}': {e}"))?
    };

    // Send input.md content via stdin concurrently in a background thread to prevent buffer deadlocks
    let input_bytes = case.input.as_bytes().to_vec();
    if let Some(mut stdin) = child.stdin.take() {
        thread::spawn(move || {
            let _ = stdin.write_all(&input_bytes);
            let _ = stdin.flush();
        });
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed waiting for child process: {e}"))?;

    if !output.status.success() {
        let stderr_str = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);
        return Err(format!(
            "Process exited with status code {code}\nStderr:\n{stderr_str}"
        ));
    }

    Ok(output.stdout)
}

/// Runs the conformance test suite according to the provided options.
///
/// # Errors
/// Returns an error string if test cases cannot be loaded from filesystem.
pub fn run_suite(options: &RunnerOptions) -> Result<SuiteSummary, String> {
    let cases = load_cases(&options.cases_dir, options.filter.as_deref()).map_err(|e| {
        format!(
            "Failed to load test cases from {}: {e}",
            options.cases_dir.display()
        )
    })?;

    if cases.is_empty() {
        return Err(format!(
            "No conformance test cases found in {}",
            options.cases_dir.display()
        ));
    }

    let modes_to_run: &[Mode] = match options.mode {
        Some(m) => std::slice::from_ref(Box::leak(Box::new(m))),
        None => &Mode::ALL,
    };

    let total_cases = cases.len();
    let total_checks = total_cases * modes_to_run.len();
    let mut passed = 0;
    let mut failed = 0;
    let mut failures = Vec::new();

    let target_desc = match &options.exec_cmd {
        Some(cmd) => format!("external command: `{cmd}`"),
        None => "internal Rust crates (`markstone-core` & `markstone-actos`)".to_string(),
    };

    println!(
        "Running conformance suite: {} cases, {} checks across {}",
        total_cases, total_checks, target_desc
    );

    for case in &cases {
        for &mode in modes_to_run {
            let expected_bytes = match case.expected.get(&mode) {
                Some(bytes) => bytes,
                None => {
                    let msg = format!(
                        "Golden file '{}' missing for case '{}'",
                        mode.filename(),
                        case.name
                    );
                    failures.push(CheckFailure {
                        case_name: case.name.clone(),
                        mode,
                        message: msg,
                        diff: None,
                    });
                    failed += 1;
                    continue;
                }
            };

            let actual_res = match &options.exec_cmd {
                Some(cmd) => execute_external(case, mode, cmd),
                None => execute_internal(case, mode),
            };

            match actual_res {
                Ok(actual_bytes) => match compare_bytes(expected_bytes, &actual_bytes) {
                    Ok(()) => {
                        passed += 1;
                        if options.verbose {
                            println!("  [PASS] {} ({})", case.name, mode.as_str());
                        }
                    }
                    Err(diff) => {
                        let report = diff.format_report(&case.name, mode.as_str());
                        eprintln!("{report}");
                        failures.push(CheckFailure {
                            case_name: case.name.clone(),
                            mode,
                            message: format!("Byte-for-byte mismatch in {}", mode.as_str()),
                            diff: Some(diff),
                        });
                        failed += 1;
                    }
                },
                Err(err) => {
                    eprintln!(
                        "  [FAIL] {} ({}) execution error: {}",
                        case.name,
                        mode.as_str(),
                        err
                    );
                    failures.push(CheckFailure {
                        case_name: case.name.clone(),
                        mode,
                        message: err,
                        diff: None,
                    });
                    failed += 1;
                }
            }
        }
    }

    println!("--------------------------------------------------------------------------------");
    println!("Conformance Test Results:");
    println!("  Total Cases:  {total_cases}");
    println!("  Total Checks: {total_checks}");
    println!("  Passed:       {passed}");
    println!("  Failed:       {failed}");
    println!("--------------------------------------------------------------------------------");

    Ok(SuiteSummary {
        total_cases,
        total_checks,
        passed,
        failed,
        failures,
    })
}
