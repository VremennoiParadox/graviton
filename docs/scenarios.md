# Scenario files

graviton loads Newtonian N-body systems from TOML files in `scenarios/`. All internal physics uses **SI units** (meters, seconds, kilograms); the `[units]` section controls how values in the file are interpreted.

## Minimal example

```toml
schema_version = 1
name = "Earth–Moon"
description = "Two-body demo"

[units]
distance = "m"
mass = "kg"
time = "s"

[physics]
integrator = "rk4"
dt = 300.0
softening_m = 1000.0

[[bodies]]
id = "earth"
name = "Earth"
class = "planet"
mass = 5.97219e24
radius = 6371000.0
position = [0.0, 0.0, 0.0]
velocity = [0.0, 0.0, 0.0]

[[bodies]]
id = "moon"
name = "Moon"
class = "moon"
mass = 7.342e22
radius = 1737400.0
position = [384400000.0, 0.0, 0.0]
velocity = [0.0, 1022.0, 0.0]
```

Validate without running the TUI:

```bash
cargo run -- validate scenarios/earth-moon.toml
```

## Schema

| Field | Required | Notes |
|-------|----------|-------|
| `schema_version` | yes | Must be `1` |
| `name` | yes | Display name |
| `description`, `author` | no | Metadata |
| `[units]` | no | Defaults: m, kg, s |
| `[physics]` | yes | Integrator and timestep |
| `[render]` | no | TUI defaults |
| `[[bodies]]` | yes* | At least one body or `[asteroid_belt]` |
| `[asteroid_belt]` | no | Procedural asteroids (see below) |

## Units

| `distance` | Scale to meters |
|------------|-----------------|
| `m` | 1 |
| `au` | 1 AU |
| `custom` | `distance_scale_m` required |

| `mass` | Scale to kg |
|--------|------------|
| `kg` | 1 |
| `solar_mass` | M☉ |
| `earth_mass` | M⊕ |
| `custom` | `mass_scale_kg` required |

| `time` | Scale to seconds |
|--------|------------------|
| `s` | 1 |
| `day` | 86400 s |
| `custom` | `time_scale_s` required |

Velocities in the file use `distance_unit / time_unit` (e.g. m/s, or AU/day when distance is `au` and time is `day`).

## Body fields

| Field | Required | Notes |
|-------|----------|-------|
| `id` | yes | Unique slug |
| `name` | yes | HUD label |
| `class` | yes | `star`, `planet`, `moon`, `dwarf_planet`, `asteroid`, `comet`, `spacecraft`, `custom` |
| `mass`, `radius` | yes | In unit scales above |
| `position`, `velocity` | yes | 3-vectors |
| `color` | no | `#rrggbb` hex |
| `horizons_id` | no | Metadata from fetch |
| `primary`, `trail_enabled`, `notes` | no | Optional metadata |

## Physics section

```toml
[physics]
integrator = "rk4"      # only rk4 supported
dt = 3600.0
dt_unit = "s"           # optional: s, day, or time (scenario time unit)
softening_m = 1000000.0

[physics.barnes_hut]
enabled = false
theta = 0.7             # opening angle; smaller = more accurate
```

CLI override: `graviton run … --barnes-hut --theta 0.5` enables Barnes–Hut even if the file has it off.

## Render section

```toml
[render]
meters_per_cell = 5000000000.0
trail_points = 2048
follow_center_of_mass = true
heatmap_enabled = true
heatmap_sample_divisor = 2
show_com_marker = false

[render.kitty]
enabled = false
mode = "density-panel"
```

## Procedural asteroid belt

For stress tests without listing hundreds of bodies by hand:

```toml
[asteroid_belt]
count = 128
inner_au = 2.2
outer_au = 3.3
seed = 42
```

Include a central `[[bodies]]` entry (typically the Sun). Bodies are generated at load time with deterministic pseudo-random orbits.

## Bundled scenarios

| File | Description |
|------|-------------|
| `earth-moon.toml` | Two-body bound system |
| `figure-eight.toml` | Three-body figure-eight choreography |
| `solar-system.toml` | HORIZONS barycentric snapshot (11 bodies) |
| `asteroid-belt.toml` | Sun + 128 procedural asteroids, Barnes–Hut on |

List scenarios:

```bash
cargo run -- list-scenarios
```

## Sharing custom scenarios

1. Keep `schema_version = 1`.
2. Run `graviton validate your.toml`.
3. Prefer SI or `au`/`day` for astronomical scales.
4. Document assumptions in `description`.
5. Do not commit large binary caches; HORIZONS raw JSON belongs in `~/.cache/graviton/`.
