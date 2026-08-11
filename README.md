# AegisForge-MCP

[![Rust](https://img.shields.io/badge/Rust-1.78%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/Tauri-v2-blue?logo=tauri)](https://tauri.app/)
[![MCP](https://img.shields.io/badge/MCP-Model%20Context%20Protocol-purple)](https://modelcontextprotocol.io/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![Security Research](https://img.shields.io/badge/Purpose-Security%20Research-red)](#)

**AegisForge-MCP** is a zero-trust AI security research engine that connects Claude (via the Model Context Protocol) to a sandboxed Rust backend for controlled, auditable security operations. Every tool call passes through a prompt injection defense layer before results ever reach the LLM.

> Named after Aegis, the protective shield of Zeus. Forged in Rust. Driven by MCP.

**[Live project site →](https://sunilgentyala.github.io/AegisForge-MCP/)**

> **Status:** early-stage scaffold. The Tauri shell, MCP server, `scan_ports` tool, and injection-defense pipeline are implemented and build cleanly. Sandboxed (Podman/Docker) tool execution and additional tools such as `fetch_page` are designed and wired for, but not yet exercised end-to-end — see [Roadmap](#roadmap).

---

## Why AegisForge

Most AI security tools pass raw tool output directly back to the language model. That design is the attack surface. A malicious web page, a crafted network response, or a poisoned file can embed instructions that hijack the LLM's next action.

AegisForge treats every byte of external data as untrusted. The `DataSanitizer` pipeline strips control characters, enforces length caps, blocks injection trigger phrases, and wraps output in a structured envelope before the model ever sees it. Tool execution itself is designed to run inside ephemeral, capability-dropped containers that are destroyed on exit.

This is not a convenience wrapper around existing CLI tools. It is a research-grade platform designed from the ground up for safe, explainable AI-driven security analysis.

---

## Features

| Capability | Detail |
|---|---|
| MCP-native tool server | Rust `rmcp` crate, JSON-RPC 2.0 over stdio or SSE |
| Zero-trust sandboxing | Podman/Docker ephemeral containers, `--cap-drop=ALL`, no shell interpolation |
| Prompt injection defense | DataSanitizer with blocklist, length cap, control-char stripping, structured envelope |
| AI reasoning trace | Real-time view of Claude's tool calls and chain-of-thought steps |
| Security dashboard | Live log feed with severity tagging across all running tools |
| Typed Tauri IPC | `#[tauri::command]` async handlers, `Arc<RwLock<T>>` shared state, no global mutables |
| Bounded ring buffers | Log (10,000 entries), terminal (5,000 lines) — never leaks memory on long sessions |
| Plugin trait | Each tool area implements `ToolPlugin` and is registered at startup by name |

---

## Architecture

```
React UI (TypeScript + Zustand + Immer)
    |
    | Tauri IPC (invoke / emit — typed)
    |
Tauri v2 Core (Rust)
 +-- #[tauri::command] async handlers
 +-- PluginRegistry (trait-based dispatch)
 +-- SandboxEngine (Podman/Docker)
    |
    | JSON-RPC 2.0 over stdio
    |
MCP Server (Rust — rmcp crate)
 +-- recon_tool    (port scanning — implemented)
 +-- web_scrape_tool (planned)
 +-- DataSanitizer (every output sanitized before LLM sees it)
    |
    | HTTPS / Anthropic API
    |
Claude (LLM)
```

Full data-flow diagrams, state design, sandboxing strategy, and the prompt injection mitigation pipeline are documented in [ARCHITECTURE.md](ARCHITECTURE.md).

---

## Quick Start

### Prerequisites

- [Rust 1.78+](https://rustup.rs/)
- [Node.js 20+](https://nodejs.org/)
- [Tauri CLI v2](https://tauri.app/start/create-project/)
- Podman or Docker (optional — only needed for sandboxed tool execution)
- Claude API key ([Anthropic Console](https://console.anthropic.com/))

### Run in Development

```bash
# Clone
git clone https://github.com/sunilgentyala/AegisForge-MCP.git
cd AegisForge-MCP

# Install frontend dependencies
npm install

# App icons aren't checked in yet — generate them once from a source image
# before your first `tauri build` (dev mode below doesn't need this):
npx tauri icon path/to/1024x1024-source.png

# Start the Tauri dev environment (compiles Rust + starts Vite)
npm run tauri dev
```

### Run the MCP Server Standalone

```bash
# Start MCP server over stdio (Claude Code picks this up from .mcp.json)
cargo run --manifest-path mcp-server/Cargo.toml --release

# Or with debug logging
RUST_LOG=aegisforge_mcp=debug cargo run --manifest-path mcp-server/Cargo.toml
```

### Connect to Claude Code

The `.mcp.json` at the repo root is pre-configured. Open this repo in Claude Code and the `aegisforge` MCP server registers automatically.

---

## MCP Tools

### `scan_ports` — implemented

Probe TCP ports on an authorized target.

```json
{
  "target_ip":  "192.168.1.1",
  "ports":      [22, 80, 443, 8080],
  "timeout_ms": 1500
}
```

Returns open/closed status and a best-effort service hint per port. Maximum 100 ports per call enforced at the schema level. Only bare IP addresses are accepted (no hostnames, no DNS resolution) and loopback/multicast targets are rejected before any network activity.

### `fetch_page` — planned

Retrieve and sanitize web page content before passing it to the LLM. This will be the first tool to exercise the sanitizer against genuinely adversarial (attacker-controlled) input rather than structured local data. See [Roadmap](#roadmap).

All output from every implemented tool passes through `DataSanitizer` before the MCP server returns the result. The LLM receives structured, bounded, injection-free data.

---

## Security Model

AegisForge enforces defense at three layers.

**Layer 1 — No shell interpolation.** Every tool binary is executed as a tokenized `Vec<String>`. String concatenation to build shell commands does not exist in this codebase.

**Layer 2 — DataSanitizer pipeline.** Every byte returned from an external source (network, file, web) passes through:
1. Length cap (default 4 KB, configurable)
2. Null byte and control character stripping
3. Injection trigger phrase blocklist
4. Structural wrapping in `<tool_output_data>` envelope

The LLM system prompt explicitly instructs Claude to treat content inside that envelope as data, not instructions.

**Layer 3 — Ephemeral containers.** Tool binaries are designed to execute in distroless Podman/Docker containers with `--cap-drop=ALL`, `--read-only`, `--memory=512m`, and `--security-opt=no-new-privileges`, destroyed after each invocation. This sandbox path is implemented (`src-tauri/src/plugins/sandbox.rs`) but not yet wired into a running tool — `scan_ports` currently runs directly on the host via the safe binary executor (no shell, absolute paths only), not inside a container.

---

## UI Panels

| Panel | Purpose |
|---|---|
| Terminal | Streaming stdout/stderr from all tool invocations |
| AI Trace | Claude's tool-call sequence and reasoning steps in real time |
| Dashboard | Severity-tagged security event log across the full session |

---

## Project Structure

```
src/                   React UI (Terminal, AI trace viewer, security dashboard, Zustand store)
src-tauri/              Tauri v2 Rust core — commands, state, sandbox engine, plugin trait
mcp-server/             Standalone MCP server (rmcp) — tool registration + DataSanitizer
ARCHITECTURE.md         Full design doc: data-flow diagram, injection-mitigation pipeline, sandboxing
docs/                   GitHub Pages project site
```

---

## Roadmap

- [ ] Exercise the Podman/Docker sandbox path end-to-end (implemented, not yet wired into a running tool)
- [ ] `fetch_page` (`web_scrape_tool`) as the second MCP tool
- [ ] OSINT module (WHOIS, ASN, certificate transparency)
- [ ] CVE cross-reference lookup via NVD API
- [ ] SSE transport for remote multi-agent deployments
- [ ] Signed container images with pinned digests
- [ ] Export session report as structured JSON
- [ ] Ship real app icons and a signed release build

---

## Author

**Sunil Gentyala**
Independent Security Researcher | IEEE Member
[sunil.gentyala@ieee.org](mailto:sunil.gentyala@ieee.org) | [GitHub](https://github.com/sunilgentyala)

Research focus: AI system security, zero-trust architectures, LLM tool-use safety.

---

## License

MIT. See [LICENSE](LICENSE) for details.

This tool is intended for authorized security research and educational use. Always obtain explicit permission before scanning or probing any target system.
