//! Deterministic conformance golden file generator for markstone.
//!
//! Generates or updates all golden files in `conformance/cases/`:
//! - `generic.html`
//! - `generic.ast.json`
//! - `actos.html`
//! - `actos.ast.json`
//!
//! Usage:
//!     cargo run -p markstone-conformance --bin generate-conformance
//!     # or with custom cases directory:
//!     cargo run -p markstone-conformance --bin generate-conformance -- --cases-dir /path/to/cases

use std::env;
use std::path::PathBuf;
use std::process::exit;

fn main() {
    let mut cases_dir = None;
    let mut args = env::args().skip(1);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--cases-dir" => {
                cases_dir = args.next().map(PathBuf::from);
            }
            "--help" | "-h" => {
                println!("markstone-conformance generator");
                println!();
                println!("USAGE:");
                println!("    generate-conformance [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("    --cases-dir <PATH>    Path to conformance cases directory");
                println!("    -h, --help            Print help information");
                return;
            }
            unknown => {
                eprintln!("Unknown argument: {unknown}");
                exit(1);
            }
        }
    }

    let target_dir = cases_dir
        .or_else(markstone_conformance::find_cases_dir)
        .unwrap_or_else(|| PathBuf::from("conformance/cases"));

    println!(
        "Generating / updating conformance golden files in '{}'...",
        target_dir.display()
    );

    match markstone_conformance::generate_all_goldens(&target_dir) {
        Ok(summary) => {
            println!(
                "Successfully generated goldens: {} cases processed, {} files written.",
                summary.cases_processed, summary.files_written
            );
        }
        Err(err) => {
            eprintln!("Error generating conformance goldens: {err}");
            exit(1);
        }
    }
}
