# AGENTS

This repository is optimized for coding agents and automation workflows.

## Working Rules

- Use Bun for frontend dependency management and script execution.
- Use `cargo check --manifest-path src-tauri/Cargo.toml` for Rust validation.
- Use `bun run check` for Svelte and TypeScript validation.
- Prefer focused changes over broad refactors.
- Keep Tauri, frontend, and workflow changes aligned when touching release behavior.

## Project Shape

- `src/`: SvelteKit frontend
- `src-tauri/`: Rust backend and Tauri application shell
- `.github/workflows/`: CI and release automation
- `.vscode/`: local workspace tooling

## Release Expectations

- Tag-based releases should produce downloadable Windows installers.
- GitHub Actions should upload build artifacts for CI and release runs.
- Changes affecting board behavior should preserve keyboard support and task grouping.
