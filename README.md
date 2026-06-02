# graviton

Terminal N-body gravitational simulator with NASA JPL HORIZONS ephemeris data, custom RK4 integration, and a ratatui interface.

> **Status:** Phase 2 — interactive TUI with trails, camera controls, and HUD. See [PLANNING.md](PLANNING.md) for the full roadmap.

## Features (planned)

- Newtonian N-body gravity with RK4 integration
- Interactive terminal UI (zoom, pan, trails, heatmap)
- Real Solar System initial conditions via NASA JPL HORIZONS
- TOML scenario files for custom systems

## Requirements

- Rust 1.75+ (`rustup` recommended)
- A true-color terminal (Kitty, Alacritty, etc.)

## Quick start

```bash
git clone https://github.com/<user>/graviton.git
cd graviton
cargo build --release
cargo run --release -- --help
cargo run --release -- run scenarios/earth-moon.toml --headless --steps 1000
cargo run --release -- run scenarios/earth-moon.toml
cargo run --release -- run scenarios/figure-eight.toml
```

### Controls (TUI)

| Key | Action |
|-----|--------|
| `Space` | Pause / resume |
| `+` / `-` | Zoom in / out |
| `0` | Reset zoom |
| Arrows / `hjkl` | Pan |
| `Tab` / `Shift+Tab` | Select next / previous body |
| `f` | Follow selected body |
| `F` | Frame all bodies |
| `1` / `2` / `3` | XY / XZ / YZ projection |
| `T` | Toggle trails |
| `H` | Toggle HUD |
| `.` / `,` | Increase / decrease time warp |
| `[` / `]` | Decrease / increase dt |
| `r` | Reset simulation |
| `?` | Help |
| `q` / `Esc` | Quit |

## Commands

```text
graviton run <scenario>       Run a simulation (TUI or headless)
graviton fetch solar-system     Fetch HORIZONS ephemeris data
graviton validate <scenario>  Validate a scenario TOML file
graviton list-scenarios       List bundled scenarios
graviton bench                Run physics benchmarks
```

## Development

```bash
cargo check
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.

## Acknowledgements

- NASA JPL [HORIZONS](https://ssd.jpl.nasa.gov/horizons/) ephemeris system
