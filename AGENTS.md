# Agent Notes for This Repo (nayu)

This file is for agentic coding tools working in `/home/yoru/Documentos/projects/nayu`.
It captures how to build/lint/test and the house style / invariants.

## Project Shape

- Language: Rust (edition `2024`).
- Workspace: root crate `nayu` + member crates under `crates/`.
- Split (see `docs/architecture.md`):
  - `crates/nayu-core`: protocol/state types shared by CLI and infra.
  - `crates/nayu-infra`: OS adapters (IPC server/client, Wayland/X11 engines, ffmpeg decode).
  - `src/main.rs`: clap CLI + output formatting.

## Commands

This repo provides a `justfile` (recommended). If you have `just` installed:

- List tasks: `just`
- Build/test: `just build`, `just test`
- Format/lint: `just fmt`, `just clippy`
- Run: `just daemon` or `just set IMG=/path/to/file`

If you do not use `just`, use Cargo directly:

- Build: `cargo build`
- Check: `cargo check --workspace`
- Format: `cargo fmt --all`
- Clippy: `cargo clippy --workspace --all-targets -- -D warnings`
- Tests: `cargo test --workspace`

## Invariants

- IPC contract is defined in `docs/ipc.md` and must remain stable.
- Default UX: one-line error; verbose only with `NAYU_DEBUG=1`.
- Wayland default scaling: cover/fill.
