# Changelog

All notable changes to this project are documented in this file.

## [0.2.0] — 2026-08-11

The project now actually builds. Every change in this release was found and
verified by running real tools (`cargo build`, `tsc`, `vite build`, a live
MCP JSON-RPC session) against the repository, not by inspection alone.

### Added
- `README.md` covering architecture, setup, and the `scan_ports` tool
- `LICENSE` (MIT)
- `docs/index.html` — a themed landing page published to GitHub Pages at
  [sunilgentyala.github.io/AegisForge-MCP](https://sunilgentyala.github.io/AegisForge-MCP/)
- `CHANGELOG.md` (this file)
- Full desktop app icon set (`32x32`, `64x64`, `128x128`, `128x128@2x`,
  `icon.ico`, `icon.icns`) — `tauri.conf.json` referenced these paths from
  the start, but the `icons/` directory never existed, so `tauri build`
  could not produce a bundled app until now
- `package-lock.json` for reproducible installs

### Fixed
Bugs found and confirmed by actually compiling/running the project for the
first time:
- **`tsconfig.json`** was missing `noEmit`, so `npm run build`
  (`tsc && vite build`) silently wrote compiled `.js` files into `src/`
  next to every source file
- **`mcp-server/src/main.rs`**: the default `RUST_LOG` filter targeted
  `aegisforge_mcp_server`, but the compiled binary crate is named
  `aegisforge_mcp` (per the `[[bin]]` name in `Cargo.toml`) — info-level
  startup/shutdown logs were silently dropped whenever `RUST_LOG` was unset
- **`src-tauri/Cargo.toml`**: the `tauri` dependency was missing the
  `protocol-asset` feature required by `assetProtocol.enable: true` in
  `tauri.conf.json` — Tauri's own build script hard-errors without it
- **`src-tauri/Cargo.toml`**: `async-trait` was used in
  `src-tauri/src/plugins/mod.rs` but never declared as a dependency
  (`E0432: unresolved import`)
- **`mcp-server/src/tools/recon_tool.rs`**: imported a `tool_handler` macro
  that does not exist in `rmcp` 0.1.5 (the version actually resolved by
  `rmcp = "0.1"`). Fixed to the real 0.1.5 API — `#[tool(tool_box)]` on
  both the tool-methods `impl` and the `ServerHandler impl` — confirmed
  against `rmcp`'s own test suite
- **`src-tauri/Cargo.toml`**: dropped `cdylib` from the library
  `crate-type`. This project has no mobile (Android/iOS) target, and
  building the unused `cdylib` artifact hit a GNU `ld` limitation
  ("export ordinal too large") once the dependency graph grew large enough
- **`ARCHITECTURE.md`**: the illustrated `ToolPlugin` trait no longer
  matched the real trait in `src-tauri/src/plugins/mod.rs`; also marked
  `web_scrape_tool` as planned rather than implemented in the data-flow
  diagram, since it doesn't exist in code yet

### Verified
- `cargo build --workspace` succeeds end-to-end on a clean machine
  (Rust + MinGW-w64 GNU toolchain)
- `aegisforge-mcp.exe` smoke-tested directly over stdio: `initialize`,
  `tools/list`, and a real `scan_ports` `tools/call` all returned correct,
  `DataSanitizer`-wrapped responses
- `npm run tauri build` (release, LTO, full bundling) exercised end-to-end

## [0.1.0] — 2026-05-29

Initial scaffold.

### Added
- Tauri v2 + React 18 + TypeScript desktop shell (Terminal, AI Reasoning
  Trace, and Security Dashboard panels; Zustand/Immer state store)
- Standalone MCP server (Rust, `rmcp`) exposing a `scan_ports` tool for
  authorized TCP port reconnaissance
- `DataSanitizer` pipeline: length caps, control-character stripping,
  an injection-trigger-phrase blocklist, and `<tool_output_data>` envelope
  wrapping on every tool output
- Sandboxing design for ephemeral, capability-dropped Podman/Docker
  container execution (`src-tauri/src/plugins/sandbox.rs`)
- `.mcp.json` for direct use with Claude Code

Note: this release was not buildable — no README, no LICENSE, no icons,
and the repository had never been compiled end-to-end. See 0.2.0.
