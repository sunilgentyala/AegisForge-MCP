// Ephemeral container orchestration for sandboxed tool execution.
//
// Tools run inside short-lived Podman/Docker containers rather than
// directly on the host OS, providing:
//   - Separate network namespace (aegisforge-net bridge only)
//   - Read-only root filesystem
//   - Dropped Linux capabilities (--cap-drop=ALL)
//   - Hard memory and CPU limits
//   - Automatic cleanup on exit (--rm)

use std::process::Stdio;
use crate::error::{AegisError, AegisResult};

#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub image: String,
    pub network_mode: NetworkMode,
    pub memory_limit_mb: u64,
    pub cpu_shares: u64,
}

#[derive(Debug, Clone)]
pub enum NetworkMode {
    None,
    Bridge(String),
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            image: "aegisforge-sandbox:latest".into(),
            network_mode: NetworkMode::Bridge("aegisforge-net".into()),
            memory_limit_mb: 512,
            cpu_shares: 512,
        }
    }
}

impl NetworkMode {
    fn as_flag(&self) -> String {
        match self {
            NetworkMode::None => "--network=none".into(),
            NetworkMode::Bridge(name) => format!("--network={}", name),
        }
    }
}

/// Run `command args` inside an ephemeral, hardened container.
///
/// Returns stdout as a String on success.
/// The container is always destroyed after execution regardless of exit code.
pub async fn execute_in_sandbox(
    config: &SandboxConfig,
    command: &str,
    args: Vec<String>,
) -> AegisResult<String> {
    let runtime = locate_runtime()?;

    // Build the full argument list as discrete tokens — no shell, no quotes,
    // no string-concatenation of user-supplied values.
    let mut run_args: Vec<String> = vec![
        "run".into(),
        "--rm".into(),
        "--read-only".into(),
        "--cap-drop=ALL".into(),
        "--security-opt=no-new-privileges".into(),
        "--no-new-privileges".into(),
        format!("--memory={}m", config.memory_limit_mb),
        format!("--cpu-shares={}", config.cpu_shares),
        config.network_mode.as_flag(),
        config.image.clone(),
        command.into(),
    ];
    run_args.extend(args);

    let output = tokio::process::Command::new(&runtime)
        .args(&run_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env_clear()
        .kill_on_drop(true)
        .output()
        .await
        .map_err(|e| AegisError::SandboxError(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AegisError::SandboxError(format!(
            "container exited {:?}: {}",
            output.status.code(),
            stderr.chars().take(256).collect::<String>()
        )));
    }

    String::from_utf8(output.stdout).map_err(|e| AegisError::Parse(e.to_string()))
}

/// Walk known absolute paths for Podman then Docker.
/// Prefers Podman (rootless by default, no daemon required).
fn locate_runtime() -> AegisResult<std::path::PathBuf> {
    const CANDIDATES: &[&str] = &[
        "/usr/bin/podman",
        "/usr/local/bin/podman",
        "/opt/homebrew/bin/podman",
        "/usr/bin/docker",
        "/usr/local/bin/docker",
    ];

    CANDIDATES
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.exists())
        .ok_or_else(|| {
            AegisError::SandboxError(
                "no container runtime found — install Podman (preferred) or Docker".into(),
            )
        })
}
