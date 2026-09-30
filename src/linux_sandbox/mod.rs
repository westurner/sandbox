//! Linux Sandbox Implementation
//!
//! Provides Linux sandboxing via Bubblewrap. Landlock helpers are capability
//! metadata only and are not part of the active executor.

mod bsd;
mod landlock;

pub mod bwrap;

pub use bwrap::BwrapBuildError;

pub use bsd::{
    create_pledge_promises_from_policy, execute_with_capsicum, execute_with_pledge,
    is_capsicum_available, is_pledge_available, CapsicumLevel, PledgePromises,
};

pub use landlock::{
    create_readonly_ruleset, create_workspace_ruleset, get_landlock_version, is_landlock_available,
    landlock_access,
};

use crate::SandboxPolicy;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(target_os = "linux")]
use which::which;

/// Find system bubblewrap in PATH
pub fn find_system_bwrap_in_path() -> Option<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        which("bwrap").ok()
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}

/// Get warning message about system bwrap
pub fn system_bwrap_warning() -> Option<String> {
    if find_system_bwrap_in_path().is_none() {
        Some("bubblewrap not found in PATH. Install with: apt install bubblewrap".to_string())
    } else {
        None
    }
}

/// Verify that Bubblewrap can create the namespaces required by the executor.
pub fn ensure_bwrap_support() -> Result<(), String> {
    let executable = find_system_bwrap_in_path()
        .ok_or_else(|| "bubblewrap executable was not found in PATH".to_string())?;
    let status = Command::new(&executable)
        .args([
            "--unshare-user",
            "--unshare-pid",
            "--unshare-ipc",
            "--ro-bind",
            "/usr",
            "/usr",
            "--proc",
            "/proc",
            "--dev",
            "/dev",
            "--",
            "/usr/bin/true",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to probe bubblewrap: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("bubblewrap capability probe exited with {status}"))
    }
}

/// Linux sandbox argument builder
pub fn create_linux_sandbox_command_args_for_policies(
    argv: Vec<String>,
    cwd: &std::path::Path,
    policy: &SandboxPolicy,
    _use_legacy_landlock: bool,
) -> Result<Vec<String>, BwrapBuildError> {
    let env = [];
    match policy.filesystem_policy() {
        crate::FileSystemSandboxPolicy::FullAccess => {
            bwrap::create_full_access_bwrap_command(argv, cwd, &env, policy.network_policy())
        }
        crate::FileSystemSandboxPolicy::ReadOnly => {
            bwrap::create_readonly_bwrap_command(argv, cwd, &env, policy.network_policy())
        }
        crate::FileSystemSandboxPolicy::WorkspaceWrite { writable_roots } => {
            bwrap::create_workspace_bwrap_command(
                argv,
                cwd,
                &writable_roots,
                &env,
                policy.network_policy(),
            )
        }
        crate::FileSystemSandboxPolicy::External => Err(
            BwrapBuildError::UnsupportedFilesystemPolicy("external filesystem sandbox"),
        ),
    }
}

/// Linux sandbox arg0 constant
pub const CODEX_LINUX_SANDBOX_ARG0: &str = "linux-sandbox";

/// Check if network should be allowed for proxy
pub fn allow_network_for_proxy(enforce_managed_network: bool) -> bool {
    enforce_managed_network
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{FileSystemSandboxPolicy, NetworkSandboxPolicy};

    #[test]
    fn test_create_linux_sandbox_args() {
        let policy = SandboxPolicy::ReadOnly {
            file_system: FileSystemSandboxPolicy::ReadOnly,
            network_access: NetworkSandboxPolicy::FullAccess,
        };

        let args = create_linux_sandbox_command_args_for_policies(
            vec!["ls".to_string(), "-la".to_string()],
            std::path::Path::new("/tmp"),
            &policy,
            false,
        )
        .unwrap();

        assert!(args.contains(&"--chdir".to_string()));
    }
}
