//! Golden file generation engine for markstone conformance suite.

use std::fs;
use std::io;
use std::path::Path;

use crate::cases::BUILTIN_CASES;
use crate::mode::Mode;

/// Results of generating or refreshing golden files.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GenerationSummary {
    pub cases_processed: usize,
    pub files_written: usize,
}

/// Generates or refreshes all golden files in `cases_dir`.
///
/// 1. Ensures `cases_dir` exists.
/// 2. Populates `cases_dir/<name>/input.md` for all built-in test cases.
/// 3. Reads all case directories containing `input.md` (both built-in and user-added).
/// 4. Generates:
///    - `generic.html` via `markstone_core::to_html`
///    - `generic.ast.json` via `markstone_core::to_ast`
///    - `actos.html` via `markstone_actos::to_html`
///    - `actos.ast.json` via `markstone_actos::to_ast`
/// 5. Overwrites the golden files with the exact bytes.
///
/// # Errors
/// Returns an `io::Error` on filesystem read/write failures or markdown conversion errors.
pub fn generate_all_goldens(
    cases_dir: &Path,
) -> Result<GenerationSummary, Box<dyn std::error::Error>> {
    fs::create_dir_all(cases_dir)?;

    // 1. Ensure built-in cases have input.md written
    for builtin in BUILTIN_CASES {
        let case_dir = cases_dir.join(builtin.name);
        fs::create_dir_all(&case_dir)?;
        let input_path = case_dir.join("input.md");
        fs::write(&input_path, builtin.input)?;
    }

    // 2. Discover all directories containing input.md
    let mut entries: Vec<_> = fs::read_dir(cases_dir)?
        .filter_map(Result::ok)
        .filter(|e| e.path().is_dir())
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut cases_processed = 0;
    let mut files_written = 0;

    for entry in entries {
        let dir_path = entry.path();
        let input_path = dir_path.join("input.md");
        if !input_path.is_file() {
            continue;
        }

        let input = fs::read_to_string(&input_path)?;

        // Generate outputs
        let generic_html = markstone_core::to_html(&input)
            .map_err(|e| io::Error::other(format!("core::to_html failed: {e:?}")))?;
        let generic_ast = markstone_core::to_ast(&input)
            .map_err(|e| io::Error::other(format!("core::to_ast failed: {e:?}")))?;
        let actos_html = markstone_actos::to_html(&input)
            .map_err(|e| io::Error::other(format!("actos::to_html failed: {e:?}")))?;
        let actos_ast = markstone_actos::to_ast(&input)
            .map_err(|e| io::Error::other(format!("actos::to_ast failed: {e:?}")))?;

        // Write outputs
        fs::write(
            dir_path.join(Mode::GenericHtml.filename()),
            generic_html.as_bytes(),
        )?;
        fs::write(
            dir_path.join(Mode::GenericAst.filename()),
            generic_ast.as_bytes(),
        )?;
        fs::write(
            dir_path.join(Mode::ActosHtml.filename()),
            actos_html.as_bytes(),
        )?;
        fs::write(
            dir_path.join(Mode::ActosAst.filename()),
            actos_ast.as_bytes(),
        )?;

        cases_processed += 1;
        files_written += 4;
    }

    Ok(GenerationSummary {
        cases_processed,
        files_written,
    })
}
