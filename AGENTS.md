# Repository Guidelines

## Project Structure & Modules
- Core Rust workspace under `crates/` (e.g., `codeprism-core`, `codeprism-lang-*`, `codeprism-utils`, `codeprism-mcp-server`). Each crate follows `src/`, optional `tests/`, and `benches/` layout.
- Top-level integration tests live in `tests/` (e.g., MCP tools and end-to-end suites). Benchmarks in `benches/`.
- Docs in `docs/` and `codeprism-docs/`; helper scripts in `scripts/`. Docker and compose files at repo root.

## Build, Test, and Development Commands
- `make build` / `make build-release`: Build all crates (`cargo build --all [--release]`).
- `make test`: Run all tests across the workspace (`cargo test --all`).
- `make check`: Format, lint, and test (gate for PRs).
- `make fmt` / `make fmt-check`: Apply or verify `rustfmt`.
- `make lint` / `make lint-fix`: Run Clippy (CI treats warnings as errors).
- `make coverage`: Generate coverage via Tarpaulin → `tarpaulin-report.html`.
- `make dev-up` / `make dev-down`: Start/stop Neo4j, Kafka, Redis for local integration testing.

## Coding Style & Naming Conventions
- Rust edition 2021; minimum `rustc` per `rust-toolchain.toml` (workspace targets 1.82).
- Use `rustfmt` defaults; no unresolved Clippy warnings (`-D warnings`).
- Crate names: `codeprism-*`; modules and files: `snake_case`. Types: `UpperCamelCase`; functions/vars: `snake_case`.
- Keep public APIs documented; prefer `anyhow`/`thiserror` for errors and `tracing` for logging.

## Testing Guidelines
- Unit tests colocated with code via `#[cfg(test)]`; integration tests under each crate’s `tests/` and at repo `tests/`.
- Frameworks: Rust test harness; property tests via `proptest`; snapshots via `insta`; benches via `criterion`.
- Naming: `tests/test_*.rs` for integration; keep focused, deterministic tests.
- Commands: `make test` for all, `cargo test -p <crate>` for a crate, `make bench` for benchmarks.

## Commit & Pull Request Guidelines
- Commit style: conventional commits with optional scope, e.g. `fix(codeprism-lang-js): handle unicode escapes` or `docs: update readme`.
- Before opening a PR: run `make check` and `make coverage`; update docs and CHANGELOG when user-facing changes occur.
- PRs should include: clear description, linked issues, reproduction steps, and test/benchmark results (logs or coverage deltas). Screenshots optional for docs.

## Security & Configuration Tips
- Do not hardcode credentials; use Docker services via `make dev-up` for local deps. Review `SECURITY.md` before changes touching IO, networking, or deserialization.
