# NASA JPL HORIZONS integration

orrery-tui can download real Solar System state vectors from the [JPL Horizons](https://ssd.jpl.nasa.gov/horizons/) API and write a scenario TOML.

## Quick usage

```bash
cargo run --release -- fetch solar-system --date 2026-06-01
cargo run --release -- validate scenarios/solar-system.toml
cargo run --release -- run scenarios/solar-system.toml
```

## API request

The client calls `https://ssd.jpl.nasa.gov/api/horizons.api` with:

| Parameter | Typical value |
|-----------|----------------|
| `format` | `json` |
| `EPHEM_TYPE` | `VECTORS` |
| `COMMAND` | Body ID (e.g. `'399'` for Earth) |
| `CENTER` | `'500@0'` (Solar System barycenter) |
| `START_TIME` | e.g. `2026-Jun-01` |
| `STOP_TIME` | Next calendar day (required: start < stop) |
| `STEP_SIZE` | `1 d` |
| `VEC_TABLE` | `2` (position + velocity) |
| `CSV_FORMAT` | `YES` |
| `OBJ_DATA` | `YES` |

The vector table appears as text inside the JSON `result` field, between `$$SOE` and `$$EOE`.

## Units in responses

HORIZONS often returns **KM-S** (km and km/s). The parser detects the `Output units` line and converts to **AU** and **AU/day** for scenario export. Positions are converted km → m → AU; velocities km/s → m/s → AU/day.

## Body IDs (preset)

| Body | COMMAND |
|------|---------|
| Sun | `10` |
| Mercury | `199` |
| Venus | `299` |
| Earth | `399` |
| Moon | `301` |
| Mars | `499` |
| Jupiter | `599` |
| Saturn | `699` |
| Uranus | `799` |
| Neptune | `899` |
| Pluto | `999` |

Masses and radii in the generated TOML come from built-in catalog values, not from HORIZONS object metadata alone.

## Cache layout

Raw JSON is stored under:

```text
~/.cache/orrery-tui/horizons/raw/{id}_{date}_{center}.json
```

Example: `399_2026-Jun-01_500_at_0.json`

### Offline mode

After a successful fetch:

```bash
cargo run -- fetch solar-system --date 2026-06-01 --offline
```

Rebuilds `scenarios/solar-system.toml` from cache only (no network).

### Force refresh

```bash
cargo run -- fetch solar-system --date 2026-06-01 --force
```

## CLI options

```text
orrery-tui fetch solar-system [--date YYYY-MM-DD] [--center 500@0] [--output path] [--offline] [--force]
```

## Reproducibility

- Same date, center, and cache files produce the same scenario.  
- HORIZONS ephemeris updates may change results if cache is refreshed.  
- Commit `scenarios/solar-system.toml` for reproducible demos without network; document the fetch date in `description`.

## Errors

Common failures:

- **Bad dates** — `STOP_TIME` must be after `START_TIME`  
- **Offline without cache** — fetch once online first  
- **Network / HTTP** — check connectivity and API status  

The fixture `tests/fixtures/horizons_earth.json` is used in unit tests for the parser.

## Alternative centers

Default `500@0` is the Solar System barycenter (recommended for a consistent system). Heliocentric centers (e.g. `500@10`) are supported via `--center` but change interpretation of positions.
