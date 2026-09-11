//! Standalone CLI runner for markstone conformance suite.
//!
//! Can run in two modes:
//! 1. Direct internal check: validates that markstone-core and markstone-actos match all golden files byte-for-byte.
//! 2. External binding check: executes an external binding binary/script, passes markdown input,
//!    and verifies byte-for-byte match against golden files.

use std::env;
use std::path::PathBuf;
use std::process::exit;

use markstone_conformance::{Mode, RunnerOptions, find_cases_dir, generate_all_goldens, run_suite};

fn print_help() {
    println!("markstone-conformance-runner - Language-agnostic byte-for-byte conformance runner");
    println!();
    println!("USAGE:");
    println!("    markstone-conformance-runner [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!(
        "    --exec <CMD>          Run external binding command instead of internal Rust crates."
    );
    println!("                          Supports placeholders in CMD:");
    println!(
        "                            {{mode}}     - Mode name (generic-html, generic-ast, actos-html, actos-ast)"
    );
    println!("                            {{input}}    - Path to input.md file");
    println!("                            {{case_dir}} - Path to case directory");
    println!("                            {{filename}} - Golden filename (e.g. generic.html)");
    println!("                            {{file}}     - Absolute path to golden file");
    println!("                          If no placeholders are used, CMD is invoked as:");
    println!("                            CMD --mode <mode> <input_path>");
    println!("                          and input markdown is also passed via stdin.");
    println!(
        "    --mode <MODE>         Filter mode: generic-html, generic-ast, actos-html, actos-ast, or all [default: all]"
    );
    println!("    --cases-dir <PATH>    Directory containing test cases [default: auto-detected]");
    println!("    --filter <STRING>     Filter cases by name substring");
    println!("    --update, --generate  Generate / update golden files for all cases");
    println!("    -v, --verbose         Verbose progress output");
    println!("    -h, --help            Print help information");
    println!();
    println!("EXAMPLES:");
    println!("    # Direct internal check against all golden files");
    println!("    markstone-conformance-runner");
    println!();
    println!("    # Test external Python binding");
    println!("    markstone-conformance-runner --exec \"python3 bindings/python/runner.py\"");
    println!();
    println!("    # Test external Node binding with custom template");
    println!(
        "    markstone-conformance-runner --exec \"node bindings/node/runner.js --mode {{mode}} {{input}}\""
    );
}

fn main() {
    let mut cases_dir: Option<PathBuf> = None;
    let mut exec_cmd: Option<String> = None;
    let mut mode: Option<Mode> = None;
    let mut filter: Option<String> = None;
    let mut update = false;
    let mut verbose = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--exec" => {
                exec_cmd = args.next();
                if exec_cmd.is_none() {
                    eprintln!("Error: --exec requires a command argument");
                    exit(1);
                }
            }
            "--mode" => {
                let m_str = match args.next() {
                    Some(s) => s,
                    None => {
                        eprintln!("Error: --mode requires a mode argument");
                        exit(1);
                    }
                };
                if m_str != "all" {
                    match m_str.parse::<Mode>() {
                        Ok(m) => mode = Some(m),
                        Err(_) => {
                            eprintln!(
                                "Error: invalid mode '{m_str}'. Allowed: all, generic-html, generic-ast, actos-html, actos-ast"
                            );
                            exit(1);
                        }
                    }
                }
            }
            "--cases-dir" => {
                cases_dir = args.next().map(PathBuf::from);
                if cases_dir.is_none() {
                    eprintln!("Error: --cases-dir requires a directory path");
                    exit(1);
                }
            }
            "--filter" => {
                filter = args.next();
                if filter.is_none() {
                    eprintln!("Error: --filter requires a string argument");
                    exit(1);
                }
            }
            "--update" | "--generate" => {
                update = true;
            }
            "-v" | "--verbose" => {
                verbose = true;
            }
            "-h" | "--help" => {
                print_help();
                return;
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                eprintln!("Run with --help for usage instructions.");
                exit(1);
            }
        }
    }

    let resolved_cases_dir = cases_dir
        .or_else(find_cases_dir)
        .unwrap_or_else(|| PathBuf::from("conformance/cases"));

    if update {
        println!(
            "Generating / updating golden files in '{}'...",
            resolved_cases_dir.display()
        );
        match generate_all_goldens(&resolved_cases_dir) {
            Ok(summary) => {
                println!(
                    "Golden files updated successfully: {} cases, {} files written.",
                    summary.cases_processed, summary.files_written
                );
            }
            Err(err) => {
                eprintln!("Error updating golden files: {err}");
                exit(1);
            }
        }
    }

    let options = RunnerOptions {
        cases_dir: resolved_cases_dir,
        exec_cmd,
        mode,
        filter,
        verbose,
    };

    match run_suite(&options) {
        Ok(summary) => {
            if summary.failed > 0 {
                eprintln!("FAILED: {} conformance check(s) failed.", summary.failed);
                exit(1);
            } else {
                println!(
                    "SUCCESS: All {} conformance check(s) passed byte-for-byte!",
                    summary.passed
                );
                exit(0);
            }
        }
        Err(err) => {
            eprintln!("Runner error: {err}");
            exit(1);
        }
    }
}
