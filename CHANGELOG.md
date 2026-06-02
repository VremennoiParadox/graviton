# Changelog

All notable changes to orrery-tui are documented here.

## [Unreleased]

### Changed

- Project renamed from **graviton** to **orrery-tui** (crate, binary, docs, cache path `~/.cache/orrery-tui/`).

## [0.1.0] - 2026-06-02

### Added

- Terminal TUI (ratatui) with pan, zoom, trails, heatmap, HUD, scenario switcher
- Newtonian N-body physics with RK4 and Plummer softening
- Direct O(n²) and Barnes–Hut octree gravity
- NASA JPL HORIZONS fetch, cache, and `solar-system` scenario
- TOML scenarios, validation, procedural asteroid belt
- Headless mode, benchmarks, and GitHub Actions CI
- Documentation in `docs/` and contribution templates

[0.1.0]: https://github.com/VremennoiParadox/orrery-tui/releases/tag/v0.1.0
