# Contributing to orrery-tui

Thank you for your interest in contributing. This project is a learning-friendly N-body simulator; clear issues and focused PRs are especially welcome.

## Getting started

1. Fork and clone the repository.
2. Install [Rust](https://rustup.rs/) (1.75+).
3. Build and test:

```bash
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --all
```

4. Run a quick sanity check:

```bash
cargo run -- run scenarios/earth-moon.toml --headless --steps 500
cargo run -- validate scenarios/earth-moon.toml
```

## What to work on

See [open issues](https://github.com/VremennoiParadox/orrery-tui/issues) and [PLANNING.md](PLANNING.md) for the long-term roadmap. Good first contributions:

- Scenario examples and documentation fixes
- Test coverage for physics edge cases
- TUI accessibility or performance
- HORIZONS cache / offline ergonomics

## Pull request guidelines

- Keep PRs focused; one logical change per PR when possible.
- Match existing code style (`cargo fmt`, `clippy -D warnings`).
- Add or update tests when behavior changes.
- Update `docs/` or `README.md` if user-facing behavior changes.
- Do not commit secrets, `.env` files, or large generated HORIZONS caches.

## Physics changes

If you change integration, gravity, or Barnes–Hut:

- Run `cargo test` and compare energy drift on `scenarios/earth-moon.toml` (headless).
- For approximation changes, document accuracy tradeoffs in `docs/physics.md`.

## Scenario files

New scenarios go in `scenarios/` and must pass:

```bash
cargo run -- validate path/to/your.toml
```

See [docs/scenarios.md](docs/scenarios.md) for the schema.

## Demo assets

If you change the TUI significantly, consider re-recording:

```bash
./scripts/record-demo.sh
```

See [assets/README.md](assets/README.md).

## Code of conduct

This project follows [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md).

## Questions

Open a [discussion or issue](https://github.com/VremennoiParadox/orrery-tui/issues) if you are unsure before investing in a large change.
