use std::process::{Command, Output};

fn run_cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_sandbox-exec"))
        .args(args)
        .output()
        .expect("sandbox-exec should start")
}

#[cfg(target_os = "linux")]
fn run_cli_with_bwrap_shim(args: &[&str]) -> (Output, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    use std::time::{SystemTime, UNIX_EPOCH};

    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("ai-sandbox-cli-bwrap-{suffix}"));
    std::fs::create_dir_all(&directory).unwrap();
    let shim = directory.join("bwrap");
    std::fs::write(
        &shim,
        "#!/bin/sh\nwhile [ \"$#\" -gt 0 ] && [ \"$1\" != \"--\" ]; do shift; done\n[ \"$#\" -gt 0 ] || exit 2\nshift\nexec \"$@\"\n",
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&shim).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&shim, permissions).unwrap();

    let output = Command::new(env!("CARGO_BIN_EXE_sandbox-exec"))
        .args(args)
        .env("PATH", &directory)
        .output()
        .expect("sandbox-exec should start with the bwrap shim");
    (output, directory)
}

#[test]
fn cli_rejects_missing_policy_and_command() {
    let missing_policy = run_cli(&[]);
    assert_eq!(missing_policy.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing_policy.stderr).contains("Usage:"));

    let missing_command = run_cli(&["readonly"]);
    assert_eq!(missing_command.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&missing_command.stderr).contains("No command provided"));
}

#[test]
fn cli_rejects_unprotected_and_unknown_policies() {
    let unprotected = run_cli(&["none", "true"]);
    assert_eq!(unprotected.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unprotected.stderr).contains("not permitted"));

    let unknown = run_cli(&["unexpected", "true"]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&unknown.stderr).contains("Unknown policy"));
}

#[test]
fn cli_reports_protected_execution_result() {
    let output = run_cli(&["readonly", "/usr/bin/true"]);
    if output.status.success() {
        return;
    }

    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("Error executing sandboxed command")
            || error.contains("Error creating sandbox request"),
        "unexpected CLI failure: {error}"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn cli_propagates_success_and_nonzero_status_from_protected_child() {
    let (success, success_dir) = run_cli_with_bwrap_shim(&["readonly", "/usr/bin/true"]);
    assert_eq!(success.status.code(), Some(0));

    let (failure, failure_dir) = run_cli_with_bwrap_shim(&["readonly", "/usr/bin/false"]);
    assert_eq!(failure.status.code(), Some(1));

    std::fs::remove_dir_all(success_dir).unwrap();
    std::fs::remove_dir_all(failure_dir).unwrap();
}
