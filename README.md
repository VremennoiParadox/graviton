# graviton

Terminal N-body gravitational simulator with NASA JPL HORIZONS ephemeris data, custom RK4 integration, and a ratatui interface.

> **Status:** Phase 4 — gravitational field heatmap, polished HUD, mouse pan/zoom, scenario switcher. See [PLANNING.md](PLANNING.md) for the full roadmap.

## Features (planned)

- Newtonian N-body gravity with RK4 integration
- Interactive terminal UI (zoom, pan, trails, heatmap)
- Real Solar System initial conditions via NASA JPL HORIZONS (`graviton fetch solar-system`)
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

### Solar System from HORIZONS

Fetch barycentric state vectors for the default preset (Sun through Pluto) and write `scenarios/solar-system.toml`:

```bash
cargo run --release -- fetch solar-system --date 2026-06-01
cargo run --release -- validate scenarios/solar-system.toml
cargo run --release -- run scenarios/solar-system.toml
```

Raw JSON responses are cached under `~/.cache/graviton/horizons/raw/`. Re-run with `--offline` to rebuild the scenario from cache without network access, or `--force` to refresh cached data.

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
| `g` | Toggle gravitational field heatmap |
| `c` | Toggle center-of-mass marker |
| `e` / `p` | Toggle energy / momentum diagnostics |
| `o` | Cycle overlay presets |
| `s` | Scenario switcher |
| `v` | Validate current scenario |
| `Shift+R` | Reload scenario from disk |
| `.` / `,` | Increase / decrease time warp |
| `[` / `]` | Decrease / increase dt |
| `r` | Reset simulation |
| `?` | Help |
| `q` / `Esc` | Quit |
| Mouse click | Select nearest body |
| Mouse drag | Pan |
| Mouse wheel | Zoom at cursor |

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
