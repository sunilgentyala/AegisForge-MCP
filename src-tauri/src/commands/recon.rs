// Network reconnaissance command handler.
//
// Security invariants enforced here:
//   1. IP-only targets — no hostname resolution (prevents SSRF via DNS)
//   2. Loopback/multicast/private ranges blocked unless explicitly authorized
//   3. Port count hard-capped to prevent resource exhaustion
//   4. All external binaries executed WITHOUT a shell (tokenized Vec<String>)
//   5. Absolute binary paths enforced — no PATH-lookup hijacking

use std::net::{IpAddr, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AegisError, AegisResult};
use crate::state::AppState;

// ── Public types exposed to the frontend via IPC ─────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ScanRequest {
    /// Must be a bare IPv4/IPv6 address string — no hostnames accepted.
    target: String,
    /// TCP ports to probe. Limited to 1–1024 items per request.
    ports: Vec<u16>,
    /// Per-port connection timeout in ms. Clamped to [100, 5000].
    timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct PortResult {
    port: u16,
    open: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    service_hint: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub struct ScanResponse {
    target: String,
    results: Vec<PortResult>,
    elapsed_ms: u64,
    total_open: usize,
}

// ── Input validation ──────────────────────────────────────────────────────────

fn parse_ip(raw: &str) -> AegisResult<IpAddr> {
    IpAddr::from_str(raw.trim()).map_err(|_| {
        AegisError::InvalidTarget(format!(
            "target must be a bare IP address, got: {}",
            raw.chars().take(64).collect::<String>()
        ))
    })
}

fn check_address_policy(ip: IpAddr) -> AegisResult<()> {
    if ip.is_loopback() {
        return Err(AegisError::PermissionDenied(
            "loopback addresses are not permitted".into(),
        ));
    }
    if ip.is_multicast() {
        return Err(AegisError::PermissionDenied(
            "multicast addresses are not permitted".into(),
        ));
    }
    // Block RFC-1918 / RFC-4193 private ranges to prevent SSRF against internal services.
    // Remove this guard in isolated lab environments where internal scanning is intended.
    if is_private(ip) {
        return Err(AegisError::PermissionDenied(
            "private RFC-1918/4193 addresses blocked by default policy".into(),
        ));
    }
    Ok(())
}

fn is_private(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let o = v4.octets();
            // 10.0.0.0/8
            o[0] == 10
            // 172.16.0.0/12
            || (o[0] == 172 && (16..=31).contains(&o[1]))
            // 192.168.0.0/16
            || (o[0] == 192 && o[1] == 168)
            // 169.254.0.0/16  (link-local)
            || (o[0] == 169 && o[1] == 254)
        }
        IpAddr::V6(v6) => {
            // fc00::/7  (unique local)
            (v6.segments()[0] & 0xfe00) == 0xfc00
            // fe80::/10 (link-local)
            || (v6.segments()[0] & 0xffc0) == 0xfe80
        }
    }
}

fn validate_ports(ports: &[u16]) -> AegisResult<()> {
    if ports.is_empty() {
        return Err(AegisError::InvalidTarget("port list must not be empty".into()));
    }
    if ports.len() > 1024 {
        return Err(AegisError::InvalidTarget(format!(
            "port count {} exceeds maximum of 1024",
            ports.len()
        )));
    }
    Ok(())
}

// ── TCP probe ─────────────────────────────────────────────────────────────────

fn well_known_service(port: u16) -> Option<&'static str> {
    match port {
        21 => Some("ftp"),
        22 => Some("ssh"),
        23 => Some("telnet"),
        25 => Some("smtp"),
        53 => Some("dns"),
        80 => Some("http"),
        110 => Some("pop3"),
        143 => Some("imap"),
        443 => Some("https"),
        445 => Some("smb"),
        3306 => Some("mysql"),
        3389 => Some("rdp"),
        5432 => Some("postgres"),
        6379 => Some("redis"),
        8080 => Some("http-alt"),
        8443 => Some("https-alt"),
        _ => None,
    }
}

async fn probe_port(addr: IpAddr, port: u16, timeout: Duration) -> PortResult {
    let sa = SocketAddr::new(addr, port);
    // spawn_blocking keeps the async executor free during the blocking TCP connect.
    let open = tokio::task::spawn_blocking(move || {
        TcpStream::connect_timeout(&sa, timeout).is_ok()
    })
    .await
    .unwrap_or(false);

    PortResult {
        port,
        open,
        service_hint: if open { well_known_service(port) } else { None },
    }
}

// ── Tauri commands ────────────────────────────────────────────────────────────

/// Async TCP port scanner exposed to the frontend.
///
/// Validates all inputs strictly before touching the network.
/// Concurrent probes run via join_all — no thread-per-port overhead.
#[tauri::command]
pub async fn scan_ports(
    request: ScanRequest,
    state: State<'_, AppState>,
) -> Result<ScanResponse, String> {
    let addr = parse_ip(&request.target).map_err(String::from)?;
    check_address_policy(addr).map_err(String::from)?;
    validate_ports(&request.ports).map_err(String::from)?;

    let timeout_ms = request.timeout_ms.unwrap_or(1000).clamp(100, 5000);
    let timeout = Duration::from_millis(timeout_ms);

    state.begin_scan().await;
    let start = std::time::Instant::now();

    let tasks: Vec<_> = request.ports.iter().map(|&p| probe_port(addr, p, timeout)).collect();
    let results = join_all(tasks).await;

    state.end_scan().await;

    let elapsed_ms = start.elapsed().as_millis() as u64;
    let total_open = results.iter().filter(|r| r.open).count();

    tracing::info!(
        target = %request.target,
        ports   = request.ports.len(),
        open    = total_open,
        elapsed = elapsed_ms,
        "scan completed"
    );

    Ok(ScanResponse {
        target: request.target,
        results,
        elapsed_ms,
        total_open,
    })
}

#[tauri::command]
pub async fn get_scan_status(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    Ok(state.get_status().await)
}

// ── Safe binary executor ──────────────────────────────────────────────────────

/// Execute a local CLI utility WITHOUT spawning a shell.
///
/// This function is the ONLY sanctioned way to call external binaries.
/// It enforces:
///   - absolute binary path (prevents PATH-hijacking)
///   - tokenized argument vector (no metachar expansion)
///   - cleared environment (prevents env-var injection)
///   - kill_on_drop (no zombie processes on cancellation)
pub async fn execute_binary_safe(
    binary_path: &str,
    args: Vec<String>,
    working_dir: Option<&str>,
) -> AegisResult<String> {
    let path = PathBuf::from(binary_path);

    if !path.is_absolute() {
        return Err(AegisError::PermissionDenied(format!(
            "binary path must be absolute: {}",
            binary_path
        )));
    }
    if !path.exists() {
        return Err(AegisError::InvalidTarget(format!(
            "binary not found: {}",
            binary_path
        )));
    }

    // Build Command directly — NO Command::new("sh") or Command::new("cmd")
    let mut cmd = tokio::process::Command::new(&path);
    cmd.args(&args);
    cmd.kill_on_drop(true);

    if let Some(dir) = working_dir {
        cmd.current_dir(dir);
    }

    // Wipe the inherited environment to prevent injection via env vars.
    cmd.env_clear();
    #[cfg(unix)]
    cmd.env("PATH", "/usr/local/bin:/usr/bin:/bin");
    #[cfg(windows)]
    cmd.env("SystemRoot", std::env::var("SystemRoot").unwrap_or_default());

    let output = cmd
        .output()
        .await
        .map_err(|e| AegisError::ExecutionFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AegisError::ExecutionFailed(format!(
            "exit {:?}: {}",
            output.status.code(),
            stderr.chars().take(256).collect::<String>()
        )));
    }

    String::from_utf8(output.stdout).map_err(|e| AegisError::Parse(e.to_string()))
}
