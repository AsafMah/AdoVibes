# Contributing

## Setup

1. Install Bun.
2. Install the Rust toolchain.
3. Install Tauri prerequisites for your platform.
4. Run `bun install`.

## Development Commands

```bash
bun run tauri dev
bun run check
cargo check --manifest-path src-tauri/Cargo.toml
```

## Pull Request Guidelines

- Keep changes scoped to a clear problem or feature.
- Validate both frontend and backend when touching shared flows.
- Include workflow updates when changing release behavior.
- Prefer small, reviewable commits.

## Releases

- Create a tag such as `0.1` to trigger a GitHub release build.
- Windows installers are expected as release assets.
- CI should continue to upload downloadable workflow artifacts for validation builds.
