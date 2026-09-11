use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_c_integration_shared_and_static() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_dir = manifest_dir.parent().expect("workspace parent dir");
    let target_debug = workspace_dir.join("target").join("debug");
    let include_dir = manifest_dir.join("include");
    let c_source = manifest_dir.join("tests").join("c_test.c");

    // Check for C compiler: prefer gcc, fallback to clang
    let compiler = if Command::new("gcc").arg("--version").output().is_ok() {
        "gcc"
    } else if Command::new("clang").arg("--version").output().is_ok() {
        "clang"
    } else {
        eprintln!("Neither gcc nor clang found on system; skipping C integration compilation.");
        return;
    };

    // Ensure cdylib and staticlib are built
    let build_status = Command::new("cargo")
        .args(["build", "-p", "markstone-abi"])
        .current_dir(workspace_dir)
        .status()
        .expect("failed to run cargo build -p markstone-abi");
    assert!(
        build_status.success(),
        "cargo build -p markstone-abi failed"
    );

    // Also create libmarkstone_abi.so and libmarkstone_abi.a symlinks if they don't exist,
    // to support both libmarkstone and libmarkstone_abi naming conventions.
    let so_src = target_debug.join("libmarkstone.so");
    let so_dst = target_debug.join("libmarkstone_abi.so");
    if so_src.exists() && !so_dst.exists() {
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&so_src, &so_dst);
    }

    let a_src = target_debug.join("libmarkstone.a");
    let a_dst = target_debug.join("libmarkstone_abi.a");
    if a_src.exists() && !a_dst.exists() {
        #[cfg(unix)]
        let _ = std::os::unix::fs::symlink(&a_src, &a_dst);
    }

    // --- 1. Test Shared Library Linking (-lmarkstone) ---
    let shared_bin = target_debug.join("c_test_shared");
    let compile_shared = Command::new(compiler)
        .args([
            "-Wall",
            "-Wextra",
            "-Werror",
            "-I",
            include_dir.to_str().unwrap(),
            c_source.to_str().unwrap(),
            "-L",
            target_debug.to_str().unwrap(),
            "-lmarkstone",
            &format!("-Wl,-rpath,{}", target_debug.display()),
            "-o",
            shared_bin.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute C compiler for shared test");

    assert!(
        compile_shared.status.success(),
        "C compilation (shared) failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&compile_shared.stdout),
        String::from_utf8_lossy(&compile_shared.stderr)
    );

    let run_shared = Command::new(&shared_bin)
        .env("LD_LIBRARY_PATH", &target_debug)
        .output()
        .expect("failed to run shared C test binary");

    assert!(
        run_shared.status.success(),
        "C test binary (shared) failed with exit code {:?}:\nstdout: {}\nstderr: {}",
        run_shared.status.code(),
        String::from_utf8_lossy(&run_shared.stdout),
        String::from_utf8_lossy(&run_shared.stderr)
    );
    let stdout_str = String::from_utf8_lossy(&run_shared.stdout);
    assert!(stdout_str.contains("All C integration assertions passed successfully!"));

    // --- 2. Test Static Library Linking (libmarkstone.a) ---
    let static_bin = target_debug.join("c_test_static");
    let compile_static = Command::new(compiler)
        .args([
            "-Wall",
            "-Wextra",
            "-Werror",
            "-I",
            include_dir.to_str().unwrap(),
            c_source.to_str().unwrap(),
            target_debug.join("libmarkstone.a").to_str().unwrap(),
            "-lpthread",
            "-ldl",
            "-lm",
            "-o",
            static_bin.to_str().unwrap(),
        ])
        .output()
        .expect("failed to execute C compiler for static test");

    assert!(
        compile_static.status.success(),
        "C compilation (static) failed:\nstdout: {}\nstderr: {}",
        String::from_utf8_lossy(&compile_static.stdout),
        String::from_utf8_lossy(&compile_static.stderr)
    );

    let run_static = Command::new(&static_bin)
        .output()
        .expect("failed to run static C test binary");

    assert!(
        run_static.status.success(),
        "C test binary (static) failed with exit code {:?}:\nstdout: {}\nstderr: {}",
        run_static.status.code(),
        String::from_utf8_lossy(&run_static.stdout),
        String::from_utf8_lossy(&run_static.stderr)
    );
    let stdout_static_str = String::from_utf8_lossy(&run_static.stdout);
    assert!(stdout_static_str.contains("All C integration assertions passed successfully!"));
}
