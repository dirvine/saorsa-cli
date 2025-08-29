# Repository Guidelines

## Project Structure & Module Organization
- Rust workspace with crates: `cli/` (menu runner), `sb/` (Markdown TUI), `sdisk/` (disk analyzer).
- Source in each crate’s `src/`; shared workspace config in root `Cargo.toml`.
- Tests live alongside modules (`mod tests`) and, for `sb`, in `sb/src/tests/*_tests.rs`.
- CI workflows in `.github/workflows/`; helper scripts in `scripts/` (e.g., `create-release.sh`).

## Build, Test, and Development Commands
- Build all: `cargo build --release` (workspace build).
- Run tools: `cargo run --bin saorsa` | `cargo run --bin sb -- /path` | `cargo run --bin sdisk -- --interactive`.
- Test all: `cargo test --all` (unit + integration tests).
- Format: `cargo fmt --all -- --check` (CI enforces on stable).
- Lint: `cargo clippy --all-targets --all-features -- -D warnings` (CI blocks warnings).
- Release helper: `./scripts/create-release.sh vX.Y.Z` (tags and packages binaries).

## Coding Style & Naming Conventions
- Language: Rust 2021; follow `rustfmt` defaults (4‑space indent, max width per toolchain).
- Naming: `snake_case` for functions/files, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts.
- Modules: keep files focused; prefer small, testable units; avoid one‑letter identifiers.
- Logging: use `tracing`; debug locally with `RUST_LOG=debug`.

## Testing Guidelines
- Framework: Rust built‑in test harness; property tests in `sb` use `proptest`/`quickcheck`.
- Placement: unit tests in the same module; broader tests under `sb/src/tests/` or a crate `tests/` dir.
- Expectations: add tests for new behavior and bug fixes; keep I/O deterministic.
- Run: `cargo test --all`; benchmarks (where present) via `cargo bench`.

## Commit & Pull Request Guidelines
- Commits: concise, imperative subject with type prefix when helpful (e.g., `fix:`, `feat:`, `ci:`, `build:`, `security:`).
- PRs: clear description, linked issues, test coverage notes, and updates to README/docs as needed; include terminal screenshots/GIFs for UI changes.
- CI must pass (fmt, clippy, build, tests) before review.

## Security & Configuration Tips
- Do not commit secrets or generated binaries; respect `.gitignore`.
- `cli` config at `~/.config/saorsa-cli/config.toml` (or OS‑specific cache/config dirs); avoid hard‑coding paths.
- For `sb` media playback, document `ffmpeg` requirement when relevant.
