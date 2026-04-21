# AdoVibes

AdoVibes is a desktop Azure DevOps sprint board built with Tauri 2, SvelteKit 5, and Rust. It replaces the Azure DevOps web UI for day-to-day sprint work with a focused Kanban experience, keyboard shortcuts, local settings persistence, and native desktop packaging.

## Stack

- Tauri 2 for the desktop shell and release packaging
- SvelteKit 5 + TypeScript for the UI
- Rust for Azure DevOps API integration
- Bun as the JavaScript package manager and task runner
- Skeleton UI + Tailwind CSS for styling

## Features

- Azure CLI and PAT authentication
- Project and team search with cached backend lookups
- Sprint board with New, Active, and Done columns
- Parent/child grouping for PBIs, Bugs, and Tasks
- Keyboard shortcuts for navigation and item movement
- Light and dark mode
- Native Windows installers via Tauri bundling

## Local Development

### Prerequisites

- Bun
- Rust toolchain
- Windows build tools required by Tauri
- Azure DevOps access via Azure CLI login or PAT

### Install dependencies

```bash
bun install
```

### Start the app in development

```bash
bun run tauri dev
```

### Run checks

```bash
bun run check
cargo check --manifest-path src-tauri/Cargo.toml
```

### Build installers locally

```bash
bun run tauri build -- --bundles msi,nsis
```

The generated Windows installers are written to:

- `src-tauri/target/release/bundle/msi`
- `src-tauri/target/release/bundle/nsis`

## GitHub Automation

- `ci.yml` runs checks and uploads Windows installer artifacts for pushes, pull requests, and manual runs.
- `release.yml` builds and publishes Tauri release assets when a tag is pushed.

## Release Process

1. Commit the release state.
2. Create and push a git tag, for example `0.1`.
3. Let the GitHub release workflow publish the installers.
4. Optionally attach additional assets or notes to the GitHub release.

## Recommended VS Code Extensions

- Svelte for VS Code
- Tauri VS Code extension
- rust-analyzer
