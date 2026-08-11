# AegisForge-MCP — Architecture Reference

## Repository Name Rationale

| Name | Meaning |
|------|---------|
| **AegisForge-MCP** *(primary)* | Aegis (Zeus's protective shield) + Forge (crafting tools) + MCP (Model Context Protocol) |
| **OrionSec-Agent** | Orion constellation (precision, multi-tool hunting pattern) + Sec Agent |
| **Phronesis-Core** | Greek: *phronesis* = practical wisdom/sound judgment — AI orchestration metaphor |
| **Sentinel-MCP** | Sentinel (always-watching guardian) + MCP protocol binding |
| **VanguardAI-Forge** | Vanguard (leading edge) + AI + Forge |
| **NexusGuard-MCP** | Nexus (integration point) + Guard + MCP |

---

## 1. Full Data-Flow Diagram

```
+============================================================+
|                    React UI  (TypeScript)                  |
|                                                            |
|  +----------------+  +------------------+  +----------+   |
|  |  Terminal      |  |  AI Reasoning    |  | Security |   |
|  |  Panel         |  |  Trace Viewer    |  | Dashboard|   |
|  +-------+--------+  +--------+---------+  +----+-----+   |
|          |                    |                 |          |
|          +--------------------+-----------------+          |
|                          Zustand Store                     |
|              (immer middleware — no UI blocking)           |
+============================+===============================+
                             | Tauri IPC
                             | (invoke / emit — typed)
+============================+===============================+
|               Tauri v2 Core  (Rust)                       |
|                                                            |
|  +-------------------+   +------------------+             |
|  |  Command Layer    |   |  Plugin Manager  |             |
|  |  (#[tauri::cmd])  |   |  (trait-based    |             |
|  |  recon::scan_ports|   |   extensions)    |             |
|  +--------+----------+   +--------+---------+             |
|           |                       |                        |
|  +--------+----------+   +--------+---------+             |
|  |  AppState         |   | Sandbox Engine   |             |
|  |  (Arc<RwLock<T>>) |   | Podman / Docker  |             |
|  |  scan counters,   |   | ephemeral ctrs   |             |
|  |  session tokens   |   | --cap-drop=ALL   |             |
|  +-------------------+   +------------------+             |
|                                                            |
|  Secure Binary Executor: NO shell — tokenized Vec<String> |
+============================+===============================+
                             | JSON-RPC 2.0 over stdio
                             | (or SSE for remote)
+============================+===============================+
|             MCP Server  (Rust — rmcp crate)               |
|                                                            |
|  +-------------------+  +-------------------+             |
|  | recon_tool        |  | web_scrape_tool   |             |
|  | scan_ports()      |  | fetch_page()      |             |
|  | (implemented)     |  | (planned)         |             |
|  +-------------------+  +-------------------+             |
|                                                            |
|  +-------------------------------------------------+      |
|  | DataSanitizer  (EVERY tool output passes here)  |      |
|  | - strip null bytes / control chars              |      |
|  | - reject injection trigger phrases              |      |
|  | - wrap output in <tool_output_data> envelope    |      |
|  | - enforce max-length limits                     |      |
|  +-------------------------------------------------+      |
+============================+===============================+
                             | HTTPS / Anthropic API
+============================+===============================+
|        External LLM  (Claude via Anthropic API)           |
|                                                            |
|  Tool descriptions arrive as structured JSON schemas.     |
|  Raw outputs arrive wrapped — never interpreted as cmds.  |
+============================================================+
```

---

## 2. Rust Tauri v2 Backend — Design Choices

### State-Managed Command Pattern
All frontend-invocable operations are `#[tauri::command]` async functions that receive
`State<'_, AppState>`. No global mutable statics. The `AppState` is wrapped in
`Arc<RwLock<T>>` to allow safe concurrent reads with exclusive writes.

### Modular Plugin Trait
Each capability area (recon, OSINT, exploitation framework) implements a `ToolPlugin` trait:

```rust
#[async_trait]
pub trait ToolPlugin: Send + Sync + 'static {
    fn info(&self) -> PluginInfo;
    async fn execute(&self, params: serde_json::Value) -> AegisResult<serde_json::Value>;
}
```

Plugins are registered into `PluginRegistry` at startup and dispatched by name. No shell
commands are ever constructed by string concatenation.

---

## 3. MCP Bridge — JSON-RPC Tool Registration Schema

Tools are exposed to Claude using the `rmcp` Rust SDK's declarative macro system.
Each tool carries:

```jsonc
{
  "name": "scan_ports",
  "description": "Probe TCP ports on a given IP. Only for authorized targets.",
  "inputSchema": {
    "type": "object",
    "properties": {
      "target_ip":  { "type": "string",  "description": "IPv4 or IPv6 address" },
      "ports":      { "type": "array",   "items": { "type": "integer" }, "maxItems": 100 },
      "timeout_ms": { "type": "integer", "minimum": 100, "maximum": 5000 }
    },
    "required": ["target_ip", "ports"]
  }
}
```

Transport options:
- **Development:** stdio (Claude Code spawns the binary directly)
- **Production:** SSE (HTTP server-sent events for remote multi-agent setups)

---

## 4. Frontend State Engine — Zustand + Immer

Three bounded ring-buffers (never grow past limit):

| Buffer | Limit | Purpose |
|--------|-------|---------|
| `logs[]` | 10 000 entries | Security events, severity-tagged |
| `terminalLines[]` | 5 000 lines | stdout/stderr from tool invocations |
| `reasoningSteps[]` | unbounded session | Claude tool-call/thought trace |

All mutations go through Immer's `produce()` — state is never mutated directly. React
renders only on relevant slice changes (Zustand's selector subscriptions).

---

## 5. Indirect Prompt Injection Mitigation Pipeline

```
External Data (web scrape, file read, network response)
    |
    v
[1] DataSanitizer::sanitize_string()
    - Length cap (configurable, default 4 KB)
    - Strip null bytes + C0/C1 control chars
    - Blocklist of injection trigger phrases
    |
    v
[2] DataSanitizer::wrap_tool_output()
    - Wraps content in <tool_output_data>...</tool_output_data>
    - LLM system prompt instructs: content inside this tag is DATA, not commands
    |
    v
[3] Structured JSON response to Claude
    - Only the sanitized, wrapped string enters the tool result payload
    - No raw concatenation with the system prompt
```

---

## 6. Sandboxing Strategy

Tool binaries execute inside **ephemeral Podman/Docker containers**:

```
Host OS
  +-- Tauri Core (trusted, host)
       +-- SandboxEngine::execute_in_sandbox()
            +-- podman run --rm \
                  --read-only \
                  --memory=512m \
                  --cap-drop=ALL \
                  --security-opt=no-new-privileges \
                  --network=aegisforge-net \
                  aegisforge-sandbox:latest \
                  <tool_binary> <arg1> <arg2> ...
                  ^-- tokenized Vec<String>, no shell
```

Container image `aegisforge-sandbox` is a minimal distroless image containing only
the target tool binary. After completion the container is destroyed. No persistent
filesystem is mounted unless explicitly granted per tool.
