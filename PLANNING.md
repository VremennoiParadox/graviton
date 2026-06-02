# graviton Planning Document

## 1. Project Overview

### Elevator Pitch

`graviton` is a terminal-based N-body gravitational simulator written in Rust that turns orbital mechanics into an interactive, visually rich scientific instrument.
It lets users load real Solar System state vectors from NASA JPL HORIZONS, simulate gravitational interactions with a custom RK4 integrator, and inspect trajectories, energy drift, field intensity, and orbital properties directly inside the terminal.
For a university admissions reader, the project demonstrates applied physics, numerical methods, systems programming, data visualization, and open-source engineering in one coherent artifact.

### Name Recommendation

Keep the name `graviton`.

Reason:

- It is short, memorable, and clearly related to gravity.
- It works well as a CLI command: `graviton run scenarios/solar-system.toml`.
- It feels serious enough for a scientific portfolio project.
- It leaves room for future subcommands such as `graviton fetch`, `graviton validate`, and `graviton bench`.

Alternative names considered:

- `orbital`: clear but generic.
- `nbody`: accurate but dry.
- `aether`: atmospheric but less scientifically direct.
- `periapsis`: elegant but harder to spell and remember.

Decision:

- Use `graviton` for the repository, binary, and crate unless the crates.io name is unavailable.

### Key Design Principles

1. Correctness first.
2. Aesthetics second.
3. Performance third.
4. Everything observable.
5. Prefer explicit units over clever abstractions.
6. Prefer small, testable physics functions over UI-driven logic.
7. Keep the simulation deterministic when the same scenario and settings are used.
8. Treat the terminal as a serious rendering surface, not as a fallback UI.
9. Make failure states readable: invalid scenarios, bad API responses, unstable time steps, and parse failures should explain themselves.
10. Make the code teach the developer Rust and computational physics.

### Why Rust

Rust is the right language for this project because:

- It gives C/C++-level performance without garbage collection pauses.
- It makes ownership of simulation state explicit.
- It supports strong typing for vectors, bodies, units, and configuration.
- It has excellent CLI, TUI, serialization, and testing libraries.
- It is respected in systems, aerospace, simulation, and open-source communities.
- It forces useful discipline around data layout, borrowing, error handling, and API boundaries.

### Target User Experience

The finished application should feel like this:

```text
$ graviton run scenarios/solar-system.toml
```

Then the user sees:

- A zoomable Solar System in the terminal.
- Colored trails behind planets.
- A selected body panel.
- Time warp controls.
- Optional gravitational field heatmap.
- Energy and momentum diagnostics.
- Real initial conditions fetched from NASA JPL HORIZONS.

### Explicit Non-Goals

This project will not:

- Attempt general relativity.
- Attempt relativistic precession corrections in the first release.
- Replace professional ephemeris tools.
- Guarantee long-term Solar System predictions over thousands of years.
- Model atmospheric drag.
- Model non-spherical bodies.
- Model tidal deformation.
- Model radiation pressure in the first release.
- Render a full 3D engine in the terminal.
- Depend on a GUI framework.
- Hide numerical error from the user.
- Prioritize raw particle count over scientific clarity in early phases.
- Support multiplayer or networking.
- Try to be a game engine.
- Ship a web app as the main interface.
- Treat NASA data as live-required for every run.

### Scientific Scope

The first release should accurately model:

- Newtonian point-mass gravity.
- 2D projected views of 3D motion.
- Initial state vectors from HORIZONS.
- Small curated systems such as Earth-Moon, Pluto-Charon, and figure-8 three-body.
- Energy and momentum diagnostics.
- Approximate orbital elements for selected two-body-dominant cases.

The first release may approximate:

- Rendering scale.
- Trail brightness.
- Heatmap intensity.
- Orbital period estimates for perturbed systems.

The first release should clearly label:

- Simulation units.
- Time step.
- Integrator.
- Energy drift.
- Whether data is real HORIZONS data or a bundled scenario.

## 2. Architecture Overview

### High-Level Shape

`graviton` should be one Rust workspace with one main binary crate at first.

Start simple:

```text
graviton/
  Cargo.toml
  README.md
  PLANNING.md
  LICENSE
  scenarios/
  src/
```

Do not split into many crates in week 1.

Reason:

- A student will learn faster with one cohesive codebase.
- Premature workspace splitting makes refactors harder.
- Public crate extraction can happen after the APIs settle.

Possible later split:

```text
crates/
  graviton-core/
  graviton-horizons/
  graviton-tui/
  graviton-cli/
```

Only do this after Phase 5 if the boundaries are stable.

### Recommended Crates

Pin exact versions in `Cargo.toml` at project start, then update deliberately.

Initial choices:

- `ratatui = "0.29"` for terminal UI widgets and frame layout.
- `crossterm = "0.28"` for cross-platform terminal input and backend.
- `clap = { version = "4.5", features = ["derive"] }` for CLI subcommands.
- `serde = { version = "1.0", features = ["derive"] }` for config and API structs.
- `toml = "0.8"` for scenario files.
- `reqwest = { version = "0.12", features = ["json", "rustls-tls"], default-features = false }` for HTTPS HORIZONS requests.
- `tokio = { version = "1.40", features = ["rt-multi-thread", "macros", "time", "fs"] }` for async HTTP and cache I/O.
- `thiserror = "1.0"` for typed application errors.
- `anyhow = "1.0"` for top-level CLI error reporting only.
- `tracing = "0.1"` for structured logs.
- `tracing-subscriber = { version = "0.3", features = ["env-filter"] }` for log configuration.
- `directories = "5.0"` for platform-correct cache paths.
- `glam = { version = "0.29", features = ["serde"] }` for vector math.
- `ordered-float = "4.2"` for sortable floating-point diagnostics when needed.
- `insta = "1.40"` for snapshot tests of scenario parsing and HORIZONS response parsing.
- `criterion = "0.5"` for physics and Barnes-Hut benchmarks.
- `proptest = "1.5"` for property tests around conservation laws and parser behavior.

Experimental or watch-list crates:

- `ratatui` evolves quickly; expect minor API changes between releases.
- `reqwest` feature flags can change recommended TLS choices.
- `glam` is stable enough for this project, but unit typing is not built in.

Crate selection principle:

- Use crates for infrastructure.
- Write the physics yourself.
- Do not import a black-box orbital mechanics crate for the core simulation.

### Module Structure

Recommended initial source layout:

```text
src/
  main.rs
  app.rs
  cli.rs
  error.rs
  physics/
    mod.rs
    body.rs
    constants.rs
    integrator.rs
    rk4.rs
    diagnostics.rs
    barnes_hut.rs
    units.rs
  render/
    mod.rs
    camera.rs
    canvas.rs
    colors.rs
    heatmap.rs
    trails.rs
    hud.rs
    kitty.rs
  horizons/
    mod.rs
    client.rs
    parser.rs
    cache.rs
    ids.rs
  scenario/
    mod.rs
    schema.rs
    loader.rs
    validate.rs
  input/
    mod.rs
    keymap.rs
    mouse.rs
  time/
    mod.rs
    clock.rs
    scheduler.rs
```

### Module Responsibilities

`main.rs`:

- Initializes logging.
- Parses CLI arguments.
- Calls the selected subcommand.
- Converts errors into readable terminal output.

`cli.rs`:

- Defines `graviton run`, `graviton fetch`, `graviton validate`, and `graviton bench`.
- Owns `clap` structs.
- Does not contain simulation logic.

`app.rs`:

- Owns the interactive application loop.
- Holds current scenario state, camera, UI toggles, selected body, and timing.
- Coordinates input, physics stepping, and rendering.

`error.rs`:

- Defines typed error variants using `thiserror`.
- Keeps lower-level errors understandable.

`physics/body.rs`:

- Defines `Body`.
- Stores mass, radius, position, velocity, acceleration, color, trail, and metadata.
- Keeps rendering metadata separate where possible.

`physics/constants.rs`:

- Defines gravitational constant `G`.
- Defines astronomical unit `AU`.
- Defines day, hour, solar mass, Earth mass, and useful constants.

`physics/units.rs`:

- Documents the internal unit system.
- Converts between SI, AU, days, km/s, and display units.

`physics/integrator.rs`:

- Defines an `Integrator` trait.
- Lets RK4 and future integrators share an interface.

`physics/rk4.rs`:

- Implements the custom RK4 integrator.
- Contains tests against known two-body circular orbits.

`physics/diagnostics.rs`:

- Computes total energy.
- Computes total linear momentum.
- Computes center of mass.
- Computes angular momentum.
- Computes energy drift from initial state.

`physics/barnes_hut.rs`:

- Implements the Barnes-Hut quadtree or octree later.
- Starts empty or behind a feature flag until Phase 5.

`render/camera.rs`:

- Maps world coordinates into terminal coordinates.
- Owns zoom, pan, projection plane, and viewport dimensions.

`render/canvas.rs`:

- Owns the terminal-cell drawing buffer.
- Supports plotting Unicode blocks, braille, or half-block characters.

`render/colors.rs`:

- Converts body classes and velocity/field intensity into ANSI 24-bit colors.

`render/trails.rs`:

- Maintains trail ring buffers.
- Calculates fading brightness.

`render/heatmap.rs`:

- Samples gravitational field intensity over the visible viewport.
- Converts intensity to color bands.

`render/hud.rs`:

- Draws selected body information.
- Draws controls, diagnostics, and warnings.

`render/kitty.rs`:

- Optional.
- Encodes high-resolution images using Kitty graphics protocol only when useful.
- Should not be required for core functionality.

`horizons/client.rs`:

- Builds API URLs.
- Performs HTTP requests.
- Handles retries and timeouts.

`horizons/parser.rs`:

- Parses HORIZONS vector text inside the JSON response.
- Converts rows into internal body state.

`horizons/cache.rs`:

- Stores fetched responses and parsed state vectors.
- Allows offline re-use.

`horizons/ids.rs`:

- Maps common names to HORIZONS command IDs.
- Example: Earth is `399`, Sun is `10`, Moon is `301`.

`scenario/schema.rs`:

- Defines serde-compatible TOML structs.

`scenario/loader.rs`:

- Reads TOML files.
- Supports local files and bundled scenarios.

`scenario/validate.rs`:

- Checks mass, radius, finite vectors, duplicate IDs, and unit declarations.

`input/keymap.rs`:

- Maps keyboard events into semantic commands.

`input/mouse.rs`:

- Converts mouse events into selection, pan, and zoom commands.

`time/clock.rs`:

- Tracks real time, simulation time, pause state, and time warp.

`time/scheduler.rs`:

- Decouples physics ticks from render frames.

### Data Flow Diagram

```text
User input
  |
  v
input::keymap / input::mouse
  |
  v
AppCommand
  |
  v
app::App state update
  |
  +--------------------+
  |                    |
  v                    v
time::scheduler    render toggles / camera / selected body
  |
  v
physics tick accumulator
  |
  v
physics::rk4::step(system, dt)
  |
  v
diagnostics::compute(system)
  |
  v
render::camera maps world to screen
  |
  v
render::canvas draws bodies, trails, heatmap, HUD
  |
  v
ratatui frame
  |
  v
crossterm backend
  |
  v
terminal
```

### State Management Across Frames

Use one central `App` struct:

```rust
struct App {
    system: SystemState,
    initial_diagnostics: Diagnostics,
    current_diagnostics: Diagnostics,
    camera: Camera,
    clock: SimulationClock,
    selected_body: Option<BodyId>,
    render_settings: RenderSettings,
    input_state: InputState,
    status_messages: Vec<StatusMessage>,
    should_quit: bool,
}
```

`SystemState` should own:

- `Vec<Body>`.
- Current simulation time.
- Integrator settings.
- Scenario metadata.
- Optional Barnes-Hut settings.

Rendering should borrow state immutably.

Physics should mutate only `SystemState`.

Input should mutate `App`, not `Body` directly, except through explicit commands.

Rule:

- Do not let renderer code update physics.
- Do not let physics code read keyboard state.
- Do not let HORIZONS code know about terminal layout.

### Frame Loop Shape

Use a fixed physics time step with an accumulator:

```rust
loop {
    let real_dt = clock.elapsed_real_time();
    app.handle_pending_input();
    app.clock.accumulate(real_dt);

    while app.clock.should_step_physics() {
        app.system.step(fixed_sim_dt);
        app.clock.consume_step();
    }

    terminal.draw(|frame| render(frame, &app))?;

    if app.should_quit {
        break;
    }
}
```

Reason:

- Physics stays deterministic.
- Rendering can run at a different rate.
- Slow frames do not directly corrupt integration.

### Error Handling Strategy

Use `thiserror` inside modules:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScenarioError {
    #[error("body `{0}` has non-positive mass")]
    NonPositiveMass(String),
}
```

Use `anyhow` only at CLI boundaries:

```rust
fn main() -> anyhow::Result<()> {
    cli::run()
}
```

Reason:

- Typed errors are better for tests.
- `anyhow` is convenient for user-facing binaries.
- Mixing both everywhere makes errors vague.

## 3. Physics Design

### The N-Body Gravitational Problem

The N-body problem asks:

Given `N` bodies with masses, positions, and velocities, how do their positions and velocities evolve under mutual gravity?

For each pair of bodies:

```text
F = G * m1 * m2 / r^2
```

Where:

- `F` is gravitational force.
- `G` is the gravitational constant.
- `m1` and `m2` are masses.
- `r` is distance between the bodies.

In vector form, the acceleration of body `i` caused by body `j` is:

```text
r_ij = position_j - position_i
distance = |r_ij|
direction = r_ij / distance
a_i_from_j = G * mass_j * direction / distance^2
```

Equivalent compact form:

```text
a_i_from_j = G * m_j * r_ij / |r_ij|^3
```

Total acceleration:

```text
a_i = sum over j != i of G * m_j * (r_j - r_i) / |r_j - r_i|^3
```

This is hard because:

- Every body affects every other body.
- The direct algorithm costs `O(n^2)` per step.
- Most systems have no closed-form solution.
- Small time-step errors accumulate.
- Close encounters can create huge accelerations.
- Chaotic systems amplify tiny numerical differences.
- Visually stable orbits can still have drifting energy.

### State Variables

For each body:

```text
position: r = (x, y, z)
velocity: v = (vx, vy, vz)
mass: m
```

The derivative of state is:

```text
dr/dt = v
dv/dt = a(r)
```

The integrator advances:

```text
state(t) -> state(t + dt)
```

### Force Calculation

The simplest direct acceleration function:

```rust
// For every body i:
//   acceleration[i] = 0
//   For every body j:
//     if i == j: continue
//     r = position[j] - position[i]
//     dist_sq = dot(r, r) + epsilon^2
//     inv_dist = 1 / sqrt(dist_sq)
//     inv_dist3 = inv_dist * inv_dist * inv_dist
//     acceleration[i] += G * mass[j] * r * inv_dist3
```

Use softening:

```text
dist_sq_softened = dot(r, r) + ε^2
```

This prevents singular acceleration when two bodies get extremely close.

### RK4 Integrator Overview

Use RK4 as the first serious integrator.

Reason:

- It is accurate enough for visually convincing orbital simulations.
- It is educational and explainable.
- It demonstrates numerical methods better than Euler.
- It is easier to implement correctly than symplectic higher-order schemes.

Warning:

- RK4 is not symplectic.
- Over very long simulations, energy can drift.
- The app should show energy drift so users learn this.

### RK4 Equations

For a differential equation:

```text
dy/dt = f(t, y)
```

RK4 computes:

```text
k1 = f(t, y)
k2 = f(t + dt/2, y + dt*k1/2)
k3 = f(t + dt/2, y + dt*k2/2)
k4 = f(t + dt,   y + dt*k3)

y_next = y + dt * (k1 + 2*k2 + 2*k3 + k4) / 6
```

For N-body simulation:

```text
y = all positions and velocities
f(y) = all velocities and accelerations
```

For each body:

```text
dr/dt = v
dv/dt = a(r)
```

### RK4 Step Pseudocode

```rust
// Input:
//   bodies: current bodies
//   dt: simulation time step in seconds
//
// State for each body:
//   r = position
//   v = velocity
//
// Derivative for each body:
//   dr = velocity
//   dv = acceleration_from_gravity(all_positions)
//
// k1:
//   k1.dr = v(t)
//   k1.dv = a(r(t))
//
// k2 temporary state:
//   r2 = r + 0.5 * dt * k1.dr
//   v2 = v + 0.5 * dt * k1.dv
//
// k2:
//   k2.dr = v2
//   k2.dv = a(r2)
//
// k3 temporary state:
//   r3 = r + 0.5 * dt * k2.dr
//   v3 = v + 0.5 * dt * k2.dv
//
// k3:
//   k3.dr = v3
//   k3.dv = a(r3)
//
// k4 temporary state:
//   r4 = r + dt * k3.dr
//   v4 = v + dt * k3.dv
//
// k4:
//   k4.dr = v4
//   k4.dv = a(r4)
//
// Final update:
//   r_next = r + dt/6 * (k1.dr + 2*k2.dr + 2*k3.dr + k4.dr)
//   v_next = v + dt/6 * (k1.dv + 2*k2.dv + 2*k3.dv + k4.dv)
```

### RK4 Implementation Notes

Do not mutate bodies while computing intermediate stages.

Use temporary arrays:

```rust
struct Derivative {
    dr: Vec3,
    dv: Vec3,
}
```

Use helper functions:

```rust
fn accelerations(bodies: &[Body], positions: &[Vec3], settings: &PhysicsSettings) -> Vec<Vec3>
fn evaluate(bodies: &[Body], positions: &[Vec3], velocities: &[Vec3]) -> Vec<Derivative>
fn apply_rk4(bodies: &mut [Body], k1: &[Derivative], k2: &[Derivative], k3: &[Derivative], k4: &[Derivative], dt: f64)
```

Prefer `f64` for physics.

Reason:

- Orbital mechanics spans huge distance and mass ranges.
- `f32` is too coarse for Solar System work.
- Rendering can downcast later if needed.

### Barnes-Hut Algorithm

Barnes-Hut approximates distant groups of bodies as one aggregate mass.

Direct method:

```text
for every body i:
  for every body j:
    compute pairwise influence
```

Cost:

```text
O(n^2)
```

Barnes-Hut method:

```text
1. Build a spatial tree.
2. Store total mass and center of mass in each node.
3. For each body, traverse the tree.
4. If a node is far enough away, approximate the whole node as one mass.
5. If not far enough, recurse into children.
```

Approximate cost:

```text
O(n log n)
```

For a 2D terminal projection, start with a quadtree.

For physically correct 3D simulation, later use an octree.

Recommended choice:

- Keep physics 3D internally.
- Start Barnes-Hut with an octree in Phase 5.
- Render a 2D projection of 3D coordinates.

### Barnes-Hut θ Parameter

The opening angle parameter `θ` controls accuracy vs speed.

Definitions:

```text
s = width of tree node
d = distance from body to node center of mass
```

Approximation rule:

```text
if s / d < θ:
    use node center of mass
else:
    inspect child nodes
```

Recommended defaults:

```text
θ = 0.5 for accurate mode
θ = 0.7 for balanced mode
θ = 1.0 for fast visual mode
```

Expose this in scenario or runtime settings:

```toml
[physics.barnes_hut]
enabled = false
theta = 0.7
```

### Numerical Stability Considerations

Main stability problems:

- Time step too large.
- Bodies too close together.
- Extreme mass ratios.
- Floating-point precision loss.
- Long simulation duration.
- Highly eccentric orbits.
- User-injected bodies with unrealistic velocities.

### Softening Length ε

Use a softening length:

```text
a_i_from_j = G * m_j * r_ij / (|r_ij|^2 + ε^2)^(3/2)
```

Recommended default:

```text
ε = 1e6 meters for Solar System scale
```

For small artificial scenarios:

```text
ε = 1e-4 scenario_distance_units converted to meters
```

The scenario file should allow:

```toml
[physics]
softening_m = 1000000.0
```

Display warning:

```text
softening is a numerical stabilizer, not a physical collision model
```

### Time-Step Strategy

Start with fixed time steps.

Reason:

- Easier to test.
- Easier to reproduce.
- Easier to explain.
- Good enough for early simulations.

Default Solar System time step:

```text
dt = 3600 seconds
```

That is one simulated hour per physics tick.

For Earth-Moon:

```text
dt = 300 seconds
```

For chaotic close-encounter scenarios:

```text
dt = 1 to 10 seconds in normalized systems
```

Later adaptive rule:

```text
dt_next = clamp(dt_min, dt_max, eta * sqrt(softening / max_acceleration))
```

Where:

```text
eta = safety factor, e.g. 0.1
```

Do not implement adaptive time stepping until fixed-step RK4 is tested.

### Units Strategy

Use SI internally.

Internal units:

```text
distance: meters
time: seconds
mass: kilograms
velocity: meters per second
acceleration: meters per second squared
```

Reason:

- NASA HORIZONS vectors can be converted cleanly.
- The gravitational constant `G` is standard in SI.
- Physics equations stay recognizable.
- Scenario files can declare friendly units and convert on load.

Display units:

- Use AU for Solar System distances.
- Use km for moon-scale distances.
- Use km/s for velocities.
- Use days or years for orbital periods.

Scenario units:

```toml
[units]
distance = "au"
mass = "solar_mass"
time = "day"
```

Loader converts to SI before simulation.

### Conservation Diagnostics

Compute total kinetic energy:

```text
K = sum over i of 0.5 * m_i * |v_i|^2
```

Compute total potential energy:

```text
U = - sum over i<j of G * m_i * m_j / |r_i - r_j|
```

Total energy:

```text
E = K + U
```

Energy drift:

```text
drift = (E_current - E_initial) / abs(E_initial)
```

Momentum:

```text
P = sum over i of m_i * v_i
```

Center of mass:

```text
R_cm = sum(m_i * r_i) / sum(m_i)
```

These should appear in the HUD or diagnostics panel.

### Collision Handling

Phase 1 should not implement physical collisions.

If two bodies get too close:

- Continue with softening.
- Show a warning.
- Optionally pause if acceleration exceeds a configured threshold.

Later possible modes:

- Merge bodies.
- Bounce bodies.
- Mark collision event.

Default:

- No collision response.

Reason:

- Real collision physics is complex.
- Incorrect collision behavior would look scientific but be misleading.

## 4. NASA HORIZONS Integration

### What HORIZONS Is

NASA JPL HORIZONS is an authoritative ephemeris system that provides positions and velocities for Solar System bodies.

It can return:

- Observer tables.
- Vector tables.
- Orbital elements.
- Physical data.

For `graviton`, use vector tables.

Reason:

- The simulator needs initial position and velocity.
- HORIZONS can provide barycentric or heliocentric state vectors.
- Vector output maps naturally into `Body` structs.

### Endpoint

Use:

```text
https://ssd.jpl.nasa.gov/api/horizons.api
```

Method:

```text
GET
```

Response format:

```text
JSON wrapper containing text result
```

Important:

- HORIZONS often returns the actual vector table as text inside the JSON `result` field.
- Do not expect a clean array of vector objects.
- The parser must find `$$SOE` and `$$EOE`.

### Example Query

Earth state vector relative to the Solar System barycenter:

```text
https://ssd.jpl.nasa.gov/api/horizons.api
  ?format=json
  &COMMAND='399'
  &EPHEM_TYPE=VECTORS
  &CENTER='500@0'
  &START_TIME='2026-Jun-01'
  &STOP_TIME='2026-Jun-02'
  &STEP_SIZE='1 d'
  &VEC_TABLE='2'
  &CSV_FORMAT='YES'
  &OBJ_DATA='YES'
```

Use URL encoding in code.

Key parameters:

- `format=json`: wraps response in JSON.
- `COMMAND='399'`: Earth body ID.
- `EPHEM_TYPE=VECTORS`: request state vectors.
- `CENTER='500@0'`: Solar System barycenter.
- `START_TIME`: start date.
- `STOP_TIME`: end date.
- `STEP_SIZE`: output interval.
- `VEC_TABLE='2'`: position and velocity table.
- `CSV_FORMAT='YES'`: easier parsing.
- `OBJ_DATA='YES'`: include body metadata.

Alternative center:

```text
CENTER='500@10'
```

This means relative to the Sun.

Recommended default:

- Use `CENTER='500@0'` for internally consistent Solar System barycentric states.

Reason:

- The Sun itself moves relative to the barycenter.
- Barycentric vectors are better for multi-body simulation.

### HORIZONS Body IDs

Use these defaults:

Sun:

- Sun: `10`

Inner planets:

- Mercury: `199`
- Venus: `299`
- Earth: `399`
- Mars: `499`

Outer planets:

- Jupiter: `599`
- Saturn: `699`
- Uranus: `799`
- Neptune: `899`

Dwarf planets:

- Pluto: `999`
- Ceres: `1`

Major moons:

- Moon: `301`
- Io: `501`
- Europa: `502`
- Ganymede: `503`
- Callisto: `504`
- Titan: `606`
- Enceladus: `602`
- Triton: `801`
- Charon: `901`

Notable asteroids:

- Vesta: `4`
- Pallas: `2`
- Hygiea: `10` is ambiguous with Sun in some asteroid contexts, so avoid using name-only lookup.
- Eros: `433`
- Bennu: `101955`
- Apophis: `99942`

Important:

- Some numeric IDs can be ambiguous.
- Prefer exact HORIZONS command strings tested in integration tests.
- Store known-good mappings in `horizons/ids.rs`.

### Fetch Strategy

Create a command:

```text
graviton fetch solar-system --date 2026-06-01
```

It should:

1. Build a list of body IDs.
2. Request one body at a time from HORIZONS.
3. Parse the first vector row per body.
4. Save a cache file.
5. Generate or update a scenario file.

Reason:

- One-body requests are easier to debug.
- HORIZONS errors are easier to associate with a body.
- Cache files can be inspected.

### Parsing Response into Body

Expected vector text contains:

```text
$$SOE
2460827.500000000, A.D. 2026-Jun-01 00:00:00.0000, X, Y, Z, VX, VY, VZ, ...
$$EOE
```

Parsing steps:

1. Deserialize outer JSON into:

```rust
struct HorizonsResponse {
    result: String,
    signature: Option<Signature>,
}
```

2. Find the substring between:

```text
$$SOE
$$EOE
```

3. Parse CSV rows.
4. Extract:

```text
X, Y, Z, VX, VY, VZ
```

5. Convert units.

HORIZONS vector units are commonly:

```text
position: au
velocity: au/day
```

The response header states the units.

Do not assume silently.

Parser rule:

- Detect units from response text.
- Fail if units are unknown.

Conversion:

```text
meters = au * 149597870700
m_per_s = au_per_day * 149597870700 / 86400
```

### Internal Body Struct

Recommended:

```rust
pub struct Body {
    pub id: BodyId,
    pub name: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub position_m: DVec3,
    pub velocity_mps: DVec3,
    pub color: Rgb,
    pub class: BodyClass,
    pub trail: Trail,
}
```

If using `glam`, prefer:

```rust
glam::DVec3
```

Reason:

- `DVec3` is double precision.
- Physics should use `f64`.

### Mass and Radius Data

HORIZONS vector tables are not the only source of mass and radius.

Use local constants for common bodies:

```text
src/physics/constants.rs
```

For unknown HORIZONS bodies:

- Allow mass and radius override in scenario.
- If missing, refuse to simulate gravity for that body.
- Do not invent masses.

### Cache Strategy

Use the OS cache directory:

```text
~/.cache/graviton/horizons/
```

With `directories` crate:

```rust
ProjectDirs::from("dev", "graviton", "graviton")
```

Cache raw responses:

```text
cache/horizons/raw/399_2026-06-01_500-at-0.json
```

Cache parsed state:

```text
cache/horizons/parsed/solar-system_2026-06-01.toml
```

Cache metadata:

```toml
fetched_at = "2026-06-02T19:25:00Z"
horizons_endpoint = "https://ssd.jpl.nasa.gov/api/horizons.api"
center = "500@0"
start_time = "2026-Jun-01"
step_size = "1 d"
body_ids = ["10", "199", "299", "399"]
```

Offline behavior:

- If network fetch fails, check cache.
- If cache exists, load it and show a warning.
- If cache does not exist, fail with a clear message.

### Default Bodies to Include

Default `solar-system` scenario:

- Sun.
- Mercury.
- Venus.
- Earth.
- Moon.
- Mars.
- Jupiter.
- Saturn.
- Uranus.
- Neptune.
- Pluto.

Extended `solar-system-full` scenario:

- Sun.
- Mercury.
- Venus.
- Earth.
- Moon.
- Mars.
- Phobos.
- Deimos.
- Jupiter.
- Io.
- Europa.
- Ganymede.
- Callisto.
- Saturn.
- Titan.
- Enceladus.
- Uranus.
- Neptune.
- Triton.
- Pluto.
- Charon.
- Ceres.
- Vesta.
- Pallas.
- Eros.
- Bennu.
- Apophis.

Keep the default scenario readable.

Reason:

- Too many bodies makes the first user experience visually noisy.
- Extended scenarios can showcase scale later.

### API Failure Handling

Handle:

- HTTP timeout.
- Non-200 status.
- JSON parse error.
- HORIZONS error in `result`.
- Missing `$$SOE`.
- Missing `$$EOE`.
- Unknown units.
- Empty vector rows.
- Ambiguous body ID.

User-facing error example:

```text
Could not fetch Earth (399) from HORIZONS.
Reason: response did not contain a vector table between $$SOE and $$EOE.
Try: graviton fetch solar-system --date 2026-06-01 --verbose
```

## 5. Rendering Design

### Terminal Rendering Goal

The renderer should make the terminal feel like a scientific display.

Use:

- Unicode block characters.
- Braille dots for dense particles if useful.
- ANSI 24-bit color.
- Fading trails.
- Heatmap overlays.
- HUD panels.
- Clear selected-body highlighting.

### Coordinate Mapping

Simulation coordinates are in meters.

Camera maps world to screen:

```text
world_position = (x_m, y_m, z_m)
camera_center = (cx_m, cy_m)
zoom = meters_per_terminal_cell

screen_x = terminal_width / 2 + (x_m - cx_m) / zoom
screen_y = terminal_height / 2 - (y_m - cy_m) / zoom / aspect_correction
```

Use aspect correction because terminal cells are taller than they are wide.

Approximate:

```text
cell_aspect = 2.0
```

So:

```text
screen_y = center_y - (world_y - camera_y) / zoom / cell_aspect
```

### Projection

Start with XY projection.

Later allow:

- XY plane.
- XZ plane.
- YZ plane.
- Camera rotation around Z.

Do not implement full 3D perspective in Phase 2.

Reason:

- Orbital mechanics clarity matters more than fake depth.
- Terminal UI benefits from stable 2D maps.

### Zoom and Pan

Camera state:

```rust
struct Camera {
    center_m: DVec2,
    meters_per_cell: f64,
    projection: Projection,
}
```

Zoom:

```text
zoom_in: meters_per_cell *= 0.8
zoom_out: meters_per_cell *= 1.25
```

Pan:

```text
center.x += dx_cells * meters_per_cell
center.y += dy_cells * meters_per_cell * cell_aspect
```

Follow selected body:

```text
camera.center = selected_body.position.xy()
```

### Trail Rendering

Each body owns or references a trail ring buffer:

```rust
struct Trail {
    points: VecDeque<DVec3>,
    capacity: usize,
}
```

Add one trail point every `trail_sample_interval`.

Do not add every physics tick if time warp is high.

Trail rendering:

```text
old point -> dim color
new point -> bright color
```

Ring buffer settings:

```toml
[render.trails]
enabled = true
max_points_per_body = 2048
sample_every_ticks = 4
fade = "linear"
```

Characters:

- `·` for faint trail points if Unicode width is reliable.
- `.` for ASCII fallback.
- `•` for brighter points.
- `█` for selected body or star core.
- `░`, `▒`, `▓` for heatmap.

ASCII fallback:

- Use `.`, `o`, `O`, `*`.

### Color Scheme

Use ANSI 24-bit color through ratatui styles.

Stars by stellar classification:

- O: blue-white, `#9bbcff`
- B: blue-white, `#aabfff`
- A: white, `#cad7ff`
- F: yellow-white, `#f8f7ff`
- G: yellow, `#fff4ea`
- K: orange, `#ffd2a1`
- M: red-orange, `#ffcc6f`

Sun:

- Use G-class color `#fff4ea`.
- Add warm halo with dim yellow/orange.

Planet palette:

- Mercury: gray-brown `#8c7f70`
- Venus: pale gold `#d9c27f`
- Earth: blue-green `#3d8bfd`
- Moon: silver gray `#c0c0c0`
- Mars: rust red `#c1440e`
- Jupiter: cream-orange `#d8a45f`
- Saturn: pale amber `#e3c16f`
- Uranus: cyan `#7fdbff`
- Neptune: deep blue `#4169e1`
- Pluto: muted tan `#b89b72`

Rationale:

- Colors should be recognizable.
- Scientific readability beats photorealism.
- Trails should remain visible on dark terminals.

### Gravitational Field Heatmap

Field intensity at a point:

```text
g(point) = |sum over bodies of G * m_i * (r_i - point) / |r_i - point|^3|
```

For each sampled terminal cell:

```rust
world_point = camera.screen_to_world(cell)
field = compute_gravitational_field(world_point, bodies)
intensity = log10(field + floor)
color = heatmap_color(intensity)
```

Use logarithmic scaling.

Reason:

- Gravity varies massively near bodies.
- Linear scale would saturate.

Heatmap display:

- Low: dark blue.
- Medium: violet.
- High: orange.
- Extreme: white/yellow.

Characters:

- ` ` for none.
- `░` for low.
- `▒` for medium.
- `▓` for high.
- `█` for extreme.

Performance rule:

- Sample heatmap at half or quarter terminal resolution.
- Recompute every few render frames.
- Disable automatically if body count is high and frame rate drops.

### Rendering Order

Recommended order:

1. Clear canvas.
2. Draw heatmap if enabled.
3. Draw trails.
4. Draw bodies.
5. Draw selected body marker.
6. Draw HUD.
7. Draw warnings.

Reason:

- Bodies should appear above trails.
- HUD must remain readable.
- Heatmap should be background information.

### Frame Rate Target

Target:

```text
60 FPS render when possible
30 FPS acceptable
```

Physics:

```text
fixed simulation tick independent from render frame
```

Example:

```text
render_dt = 1/60 second real time
physics_dt = 3600 seconds simulated time
time_warp = 24 physics ticks per real second
```

Decoupling:

- Render loop consumes current state.
- Physics accumulator runs zero or more steps before render.
- If physics falls behind, cap catch-up steps per frame.

Cap:

```text
max_physics_steps_per_frame = 8
```

If exceeded:

- Drop visual catch-up.
- Show warning: `simulation overloaded; reducing time warp recommended`.

### Kitty Graphics Protocol

Use Kitty graphics only for optional high-resolution elements.

Good uses:

- Exported screenshot preview.
- High-resolution density map panel.
- Static background starfield.
- Future image-based minimap.

Avoid using Kitty graphics for:

- Core body rendering.
- Every-frame full-screen animation in early versions.
- Essential UI.

Reason:

- Unicode rendering is portable and fast.
- Kitty graphics support is excellent in the user's environment but not universal.
- The app should still work in other terminals.

Implementation approach:

- Put Kitty support behind a config flag.
- Detect `TERM` and `KITTY_WINDOW_ID`.
- Keep a crossterm-only renderer as default.

```toml
[render.kitty]
enabled = false
mode = "density-panel"
```

## 6. Interactivity Design

### Keybinding Map

Global:

```text
q / Esc        quit
?              toggle help
Space          pause / resume
r              reset scenario
R              reload scenario file
```

Camera:

```text
+ / =          zoom in
-              zoom out
0              reset zoom
Arrow keys     pan
h/j/k/l        pan left/down/up/right
f              follow selected body
F              frame all bodies
1              XY projection
2              XZ projection
3              YZ projection
```

Time:

```text
.              increase time warp
,              decrease time warp
]              increase physics dt
[              decrease physics dt
t              toggle time controls panel
```

Selection:

```text
Tab            select next body
Shift+Tab      select previous body
/              search body by name
Enter          lock/unlock selected body
```

Overlays:

```text
o              cycle overlays
g              toggle gravitational field heatmap
e              toggle energy diagnostics
p              toggle momentum diagnostics
c              toggle center of mass marker
T              toggle trails
H              toggle HUD
```

Scenario:

```text
s              open scenario switcher
v              validate current scenario
i              insert custom body mode
x              remove selected custom body
```

Display:

```text
b              toggle Barnes-Hut debug tree after Phase 5
m              toggle minimap
u              cycle display units
```

### Mouse Support

Enable mouse capture in crossterm.

Interactions:

- Left click on body: select nearest visible body.
- Left drag: pan camera.
- Scroll up: zoom in at cursor.
- Scroll down: zoom out at cursor.
- Right click: open context panel if supported.

Selection algorithm:

```text
for each body:
    project body position to screen
    compute screen distance to click
choose nearest body within threshold
```

Threshold:

```text
2 terminal cells
```

Zoom-at-cursor:

```text
world_before = camera.screen_to_world(mouse_cell)
apply_zoom()
world_after = camera.screen_to_world(mouse_cell)
camera.center += world_before - world_after
```

Reason:

- Zoom feels anchored under the mouse.

### HUD Design

HUD layout:

```text
+------------------------------------------------------+
| graviton | paused | warp 24x | dt 3600s | RK4        |
+---------------------------+--------------------------+
| simulation viewport       | selected body            |
|                           | name: Earth              |
|                           | mass: 5.972e24 kg        |
|                           | speed: 29.78 km/s        |
|                           | r: 1.000 AU              |
|                           | period: 365.25 d         |
|                           | periapsis: ...           |
|                           | apoapsis: ...            |
+---------------------------+--------------------------+
| energy drift: 2.1e-8 | bodies: 11 | fps: 60          |
+------------------------------------------------------+
```

Selected body panel:

- Name.
- Class.
- Mass.
- Radius.
- Position.
- Velocity.
- Speed.
- Distance from primary.
- Estimated orbital period.
- Apoapsis.
- Periapsis.
- Trail length.
- HORIZONS ID if applicable.

### Orbital Period Estimate

For approximately elliptic two-body cases:

```text
T = 2π * sqrt(a^3 / μ)
```

Where:

```text
μ = G * (M_primary + m_body)
a = semi-major axis
```

Estimate specific orbital energy:

```text
ε_orbit = v^2 / 2 - μ / r
```

Semi-major axis:

```text
a = - μ / (2 * ε_orbit)
```

Only show apoapsis/periapsis if:

- A primary is selected or inferred.
- Orbit is bound.
- Eccentricity is less than a reasonable threshold.

Otherwise show:

```text
orbit: unbound or strongly perturbed
```

### Add Custom Body Mode

The student-friendly version should support a guided input panel:

Fields:

- Name.
- Mass.
- Radius.
- Position.
- Velocity.
- Color.

Shortcut:

```text
i
```

Recommended defaults:

- Position at mouse cursor.
- Velocity equal to selected body plus user-adjustable delta.
- Mass small enough not to destroy the whole system unless confirmed.

Confirmation:

```text
Add body `Probe` with mass 1000 kg? [y/N]
```

If mass is huge:

```text
Warning: this body will significantly perturb the system.
```

## 7. Scenario System

### Scenario Goals

Scenario files should be:

- Human-readable.
- Versioned.
- Validated before simulation.
- Friendly to contributors.
- Capable of representing real and fictional systems.

Use TOML.

Reason:

- TOML is readable.
- Rust has strong TOML support through `serde`.
- It fits configuration better than JSON.
- It works well in git diffs.

### Scenario Schema

Top-level:

```toml
schema_version = 1
name = "Earth-Moon"
description = "A simple Earth-Moon system."
author = "graviton team"

[units]
distance = "m"
mass = "kg"
time = "s"

[physics]
integrator = "rk4"
dt = 300.0
softening_m = 1000.0

[render]
meters_per_cell = 10000000.0
trail_points = 1024

[[bodies]]
id = "earth"
name = "Earth"
class = "planet"
mass = 5.97219e24
radius = 6371000.0
position = [0.0, 0.0, 0.0]
velocity = [0.0, 0.0, 0.0]
color = "#3d8bfd"
```

### Body Fields

Required:

- `id`
- `name`
- `class`
- `mass`
- `radius`
- `position`
- `velocity`

Optional:

- `color`
- `horizons_id`
- `primary`
- `trail_enabled`
- `notes`

Classes:

- `star`
- `planet`
- `moon`
- `dwarf_planet`
- `asteroid`
- `comet`
- `spacecraft`
- `custom`

### Worked Example: Figure-8 Three-Body Orbit

Use normalized units in the scenario file, then convert to SI-like internal units with declared scale.

```toml
schema_version = 1
name = "Figure-8 Three-Body"
description = "A classic equal-mass three-body periodic orbit."
author = "graviton team"

[units]
distance = "custom"
distance_scale_m = 1.0e9
mass = "custom"
mass_scale_kg = 1.0e24
time = "custom"
time_scale_s = 86400.0

[physics]
integrator = "rk4"
dt = 0.001
dt_unit = "time"
softening_m = 1000.0

[render]
meters_per_cell = 2.0e7
trail_points = 4096
follow_center_of_mass = true

[[bodies]]
id = "body-a"
name = "A"
class = "custom"
mass = 1.0
radius = 0.02
position = [-0.97000436, 0.24308753, 0.0]
velocity = [0.4662036850, 0.4323657300, 0.0]
color = "#ff6b6b"

[[bodies]]
id = "body-b"
name = "B"
class = "custom"
mass = 1.0
radius = 0.02
position = [0.97000436, -0.24308753, 0.0]
velocity = [0.4662036850, 0.4323657300, 0.0]
color = "#4dabf7"

[[bodies]]
id = "body-c"
name = "C"
class = "custom"
mass = 1.0
radius = 0.02
position = [0.0, 0.0, 0.0]
velocity = [-0.93240737, -0.86473146, 0.0]
color = "#51cf66"
```

Validation note:

- Custom unit scenarios must define scale factors.
- Loader converts all values to SI before simulation.

### Bundled Scenarios

Ship these files:

```text
scenarios/solar-system.toml
scenarios/earth-moon.toml
scenarios/figure-eight.toml
scenarios/chaotic-five-body.toml
scenarios/pluto-charon.toml
scenarios/rogue-body-injection.toml
```

### Solar System Scenario

Purpose:

- Demonstrate real data.
- Show HORIZONS integration.
- Make the project scientifically credible.

Includes:

- Sun.
- Mercury.
- Venus.
- Earth.
- Moon.
- Mars.
- Jupiter.
- Saturn.
- Uranus.
- Neptune.
- Pluto.

Initial data:

- Generated from HORIZONS cache.
- Date pinned in the scenario metadata.

### Earth-Moon Scenario

Purpose:

- Easier first physics validation.
- Shows orbital period and distance clearly.

Includes:

- Earth.
- Moon.

Recommended time step:

```text
dt = 300 seconds
```

### Figure-8 Three-Body Scenario

Purpose:

- Shows numerical methods.
- Demonstrates non-trivial periodic orbit.
- Looks visually impressive.

### Chaotic Five-Body Scenario

Purpose:

- Shows sensitive dependence on initial conditions.
- Demonstrates why diagnostics matter.

Design:

- Five equal-mass bodies.
- Slightly asymmetric initial velocities.
- Short time step.

### Pluto-Charon Binary Scenario

Purpose:

- Shows barycentric motion.
- Great for explaining binary systems.

Includes:

- Pluto.
- Charon.
- Optional small moons later.

### Rogue Body Injection Demo

Purpose:

- Shows interactivity.
- Demonstrates perturbation.
- Lets users inject a massive body through the inner Solar System.

Design:

- Start with a stable small system.
- Spawn a rogue body with high eccentricity.
- Show energy and trajectory changes.

### Runtime Loading and Validation

Loading steps:

1. Read TOML file.
2. Deserialize into raw schema structs.
3. Validate schema version.
4. Validate units.
5. Validate body IDs are unique.
6. Validate masses are positive.
7. Validate radii are non-negative.
8. Validate positions and velocities are finite.
9. Convert all values to SI.
10. Build `SystemState`.
11. Compute initial diagnostics.
12. Start simulation.

Validation failures should list all obvious issues at once.

Example:

```text
Invalid scenario:
- body `earth` has duplicate id
- body `probe` mass must be positive
- physics.dt must be greater than zero
```

## 8. Development Phases

### Phase 0 – Scaffold & Tooling (Week 1)

#### Goals

- Create the Rust project.
- Establish formatting, linting, testing, and CLI shape.
- Make the repository feel professional from day one.

#### Specific Tasks

- Initialize `Cargo.toml`.
- Add initial dependencies.
- Create source module skeleton.
- Add `rustfmt.toml` if needed.
- Add `.gitignore`.
- Add `README.md` stub.
- Add `LICENSE`.
- Add `scenarios/` directory.
- Add `cargo fmt` and `cargo clippy` checks.
- Add first smoke test.
- Add `graviton --help`.

#### Files and Modules Created

```text
Cargo.toml
README.md
LICENSE
.gitignore
src/main.rs
src/cli.rs
src/error.rs
src/app.rs
src/physics/mod.rs
src/render/mod.rs
src/scenario/mod.rs
src/horizons/mod.rs
src/input/mod.rs
src/time/mod.rs
scenarios/.gitkeep
```

#### Entry Criteria

- Project directory exists.
- Rust toolchain is installed.
- Developer can run `cargo --version`.

#### Exit Criteria

- `cargo check` passes.
- `cargo fmt --check` passes.
- `cargo clippy --all-targets -- -D warnings` passes.
- `graviton --help` prints useful commands.

#### Done Looks Like

The repository builds cleanly and has a clear skeleton for the rest of the project.

### Phase 1 – Core Physics Engine (Weeks 1–2)

#### Goals

- Implement Newtonian gravity.
- Implement custom RK4.
- Validate against simple known systems.
- Build confidence before rendering complexity.

#### Specific Tasks

- Define `Body`.
- Define `SystemState`.
- Define constants.
- Implement acceleration calculation.
- Implement RK4 stepping.
- Implement energy diagnostics.
- Implement momentum diagnostics.
- Implement center-of-mass calculation.
- Add Earth-Moon scenario manually.
- Add figure-8 scenario manually.
- Write unit tests for two-body acceleration symmetry.
- Write test for center-of-mass stability.
- Write test for approximate circular orbit stability.

#### Files and Modules Created

```text
src/physics/body.rs
src/physics/constants.rs
src/physics/units.rs
src/physics/integrator.rs
src/physics/rk4.rs
src/physics/diagnostics.rs
src/scenario/schema.rs
src/scenario/loader.rs
src/scenario/validate.rs
scenarios/earth-moon.toml
scenarios/figure-eight.toml
```

#### Entry Criteria

- Phase 0 complete.
- CLI can load a subcommand.

#### Exit Criteria

- Direct acceleration works.
- RK4 advances bodies.
- Energy drift is measurable.
- Scenario loader converts to SI.
- Tests pass.

#### Done Looks Like

Running a non-interactive command can simulate Earth-Moon for a fixed number of steps and print final diagnostics.

Example:

```text
graviton run scenarios/earth-moon.toml --headless --steps 1000
```

### Phase 2 – Terminal Renderer & Basic Interaction (Weeks 2–3)

#### Goals

- Build the first interactive TUI.
- Render bodies and trails.
- Add camera controls.
- Make the simulation enjoyable to watch.

#### Specific Tasks

- Initialize ratatui with crossterm backend.
- Build terminal event loop.
- Add `App` state.
- Add fixed-step scheduler.
- Implement camera mapping.
- Implement canvas buffer.
- Render bodies.
- Render trails.
- Add pause/resume.
- Add zoom and pan.
- Add body selection with keyboard.
- Add basic HUD.
- Add FPS counter.

#### Files and Modules Created

```text
src/app.rs
src/render/camera.rs
src/render/canvas.rs
src/render/colors.rs
src/render/trails.rs
src/render/hud.rs
src/input/keymap.rs
src/time/clock.rs
src/time/scheduler.rs
```

#### Entry Criteria

- Physics can advance independently.
- At least two scenario files load.

#### Exit Criteria

- `graviton run scenarios/earth-moon.toml` opens a TUI.
- User can pause.
- User can zoom.
- User can pan.
- User can select bodies.
- Trails appear.
- HUD shows selected body.

#### Done Looks Like

The Earth-Moon scenario is visually understandable in the terminal and can be controlled without restarting.

### Phase 3 – NASA HORIZONS Integration & Real Data (Weeks 3–4)

#### Goals

- Fetch real Solar System initial conditions.
- Cache HORIZONS responses.
- Generate a real-data scenario.
- Make the project scientifically credible.

#### Specific Tasks

- Implement HORIZONS client.
- Implement URL builder.
- Implement response structs.
- Implement vector table parser.
- Implement unit detection.
- Implement cache directory handling.
- Implement `graviton fetch`.
- Implement body ID mapping.
- Add local mass/radius constants.
- Generate `scenarios/solar-system.toml`.
- Add parser snapshot tests.
- Add offline-cache fallback.

#### Files and Modules Created

```text
src/horizons/client.rs
src/horizons/parser.rs
src/horizons/cache.rs
src/horizons/ids.rs
src/physics/constants.rs
scenarios/solar-system.toml
tests/horizons_parser.rs
```

#### Entry Criteria

- Scenario system works.
- TUI can run bundled scenarios.

#### Exit Criteria

- `graviton fetch solar-system --date YYYY-MM-DD` works.
- Raw HORIZONS response is cached.
- Parsed scenario is cached or written.
- App can run from cached data without network.
- Parser tests cover a saved HORIZONS sample.

#### Done Looks Like

The user can fetch real planetary vectors, then launch a Solar System simulation from those vectors.

### Phase 4 – Visual Polish & Advanced Overlays (Weeks 4–6)

#### Goals

- Make the app visually stunning.
- Add field heatmap.
- Improve HUD.
- Add mouse support.
- Prepare demo material.

#### Specific Tasks

- Add gravitational field heatmap.
- Add color gradients.
- Add body glow effect.
- Add selected-body marker.
- Add center-of-mass marker.
- Add energy drift graph or sparkline.
- Add mouse click selection.
- Add mouse wheel zoom.
- Add scenario switcher.
- Add help overlay.
- Add warning overlay.
- Add optional Kitty graphics module stub.
- Add render settings to scenario.

#### Files and Modules Created

```text
src/render/heatmap.rs
src/render/kitty.rs
src/input/mouse.rs
src/render/hud.rs
src/render/colors.rs
src/render/canvas.rs
```

#### Entry Criteria

- Interactive renderer works.
- HORIZONS scenario exists.

#### Exit Criteria

- Heatmap can be toggled.
- Mouse selection works.
- HUD is polished.
- Colors are consistent.
- Help screen documents controls.
- Demo recording looks impressive.

#### Done Looks Like

A viewer can understand the simulation, inspect bodies, and see gravitational structure without reading the source code.

### Phase 5 – Barnes-Hut Optimisation & Stress Testing (Weeks 6–7)

#### Goals

- Add scalable force approximation.
- Benchmark direct vs Barnes-Hut.
- Stress test with many bodies.
- Teach algorithmic performance tradeoffs.

#### Specific Tasks

- Design octree node structure.
- Implement bounding cube.
- Insert bodies into tree.
- Compute node mass and center of mass.
- Implement Barnes-Hut acceleration.
- Add `theta` setting.
- Add debug tree visualization if useful.
- Add benchmark suite.
- Add random asteroid-belt scenario.
- Compare energy drift direct vs Barnes-Hut.
- Document limitations.

#### Files and Modules Created

```text
src/physics/barnes_hut.rs
benches/direct_vs_barnes_hut.rs
scenarios/asteroid-belt.toml
```

#### Entry Criteria

- Direct `O(n^2)` physics is correct.
- Diagnostics are available.
- Benchmark tooling is in place.

#### Exit Criteria

- Barnes-Hut can be enabled.
- Direct and Barnes-Hut modes produce similar short-term results for test systems.
- Benchmarks show improvement for large `n`.
- User can tune `theta`.
- Stress scenario remains interactive.

#### Done Looks Like

The project can explain and demonstrate the difference between `O(n^2)` and `O(n log n)` gravitational simulation.

### Phase 6 – Open-Source Release Prep (Week 8)

#### Goals

- Make the project presentable.
- Make it easy for others to install, run, and contribute.
- Package it as a portfolio artifact.

#### Specific Tasks

- Write polished README.
- Add architecture diagram.
- Add demo GIF.
- Add asciinema recording.
- Add scenario documentation.
- Add contribution guide.
- Add issue templates.
- Add GitHub Actions CI.
- Add release profile optimization.
- Add crates.io metadata.
- Add `cargo install` instructions.
- Add screenshots.
- Add roadmap.
- Add limitations section.
- Tag `v0.1.0`.

#### Files and Modules Created

```text
README.md
CONTRIBUTING.md
CODE_OF_CONDUCT.md
.github/workflows/ci.yml
.github/ISSUE_TEMPLATE/bug_report.md
.github/ISSUE_TEMPLATE/feature_request.md
docs/scenarios.md
docs/physics.md
docs/horizons.md
assets/demo.gif
assets/demo.cast
```

#### Entry Criteria

- Core app works.
- Real-data scenario works.
- Visual polish is acceptable.
- Tests and benchmarks exist.

#### Exit Criteria

- Fresh clone can build.
- README explains the project in under 60 seconds.
- Demo GIF exists.
- CI passes.
- License exists.
- Release tag exists.

#### Done Looks Like

The repository is ready to send to mentors, admissions readers, open-source contributors, and friends.

## 9. Open-Source Release Checklist

### README Structure

Use this README order:

1. Project title and one-line pitch.
2. Demo GIF.
3. Badges.
4. Why this exists.
5. Features.
6. Installation.
7. Quick start.
8. Controls.
9. Scenarios.
10. NASA HORIZONS data.
11. Physics model.
12. Screenshots.
13. Architecture.
14. Development setup.
15. Testing.
16. Benchmarks.
17. Roadmap.
18. Contributing.
19. License.
20. Acknowledgements.

Badges:

```text
build passing
license MIT OR Apache-2.0
rust stable
crates.io version after publish
```

### Install Instructions

Source install:

```text
git clone https://github.com/<user>/graviton
cd graviton
cargo run --release -- run scenarios/earth-moon.toml
```

Future crates.io install:

```text
cargo install graviton
graviton run scenarios/solar-system.toml
```

### Usage Examples

```text
graviton run scenarios/earth-moon.toml
graviton run scenarios/figure-eight.toml
graviton fetch solar-system --date 2026-06-01
graviton validate scenarios/solar-system.toml
graviton bench
```

### Scenario Documentation

Create:

```text
docs/scenarios.md
```

Include:

- Schema.
- Units.
- Body fields.
- Validation rules.
- Worked examples.
- How to share custom scenarios.

### Physics Documentation

Create:

```text
docs/physics.md
```

Include:

- Newtonian gravity.
- RK4.
- Softening.
- Energy drift.
- Barnes-Hut.
- Known limitations.

### HORIZONS Documentation

Create:

```text
docs/horizons.md
```

Include:

- Endpoint.
- Parameters.
- Body IDs.
- Cache behavior.
- Offline behavior.
- Reproducibility notes.

### Crates to Publish

For `v0.1.0`, publish only one crate if possible:

```text
graviton
```

Do not publish internal crates yet.

Reason:

- Public APIs are not stable.
- A single crate is easier to maintain.
- Portfolio impact comes from the app, not crate count.

Possible future crates:

```text
graviton-core
graviton-horizons
graviton-tui
```

Publish future crates only if external users need them.

### CI/CD Pipeline

Use GitHub Actions:

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo test --all-features
      - run: cargo build --release
```

Add later:

- `cargo deny`.
- `cargo audit`.
- Cross-platform builds.
- Release artifact builds.

### Demo Recording Strategy

Use `asciinema` for terminal-native recording:

```text
asciinema rec assets/demo.cast
```

Use `vhs` for repeatable GIF generation:

```text
vhs assets/demo.tape
```

Recommendation:

- Use `vhs` for the README GIF.
- Use `asciinema` for interactive terminal playback.

Reason:

- GIFs are easy for GitHub visitors.
- asciinema preserves terminal behavior.
- VHS makes the demo reproducible.

Demo scenes:

1. Launch Earth-Moon.
2. Zoom out.
3. Select Moon.
4. Toggle trails.
5. Toggle heatmap.
6. Switch to Solar System.
7. Increase time warp.
8. Show energy drift.

### Licensing Recommendation

Use dual license:

```text
MIT OR Apache-2.0
```

Reason:

- Standard in the Rust ecosystem.
- Friendly for open-source contributors.
- Compatible with many downstream uses.
- Admissions readers will recognize it as professional.

Add:

```text
LICENSE-MIT
LICENSE-APACHE
```

Cargo metadata:

```toml
license = "MIT OR Apache-2.0"
```

### Contribution Guide

`CONTRIBUTING.md` should include:

- How to run the app.
- How to run tests.
- How to add scenarios.
- How to add body constants.
- Code style.
- Commit style.
- Good first issues.
- Scientific correctness expectations.

Good first issues:

- Add a scenario.
- Add a body color.
- Improve docs.
- Add parser fixture.
- Add HUD field.

Hard issues:

- Barnes-Hut.
- Adaptive timestep.
- Orbital elements.
- Kitty graphics density panel.

### Release Criteria for v0.1.0

Required:

- Runs on Linux terminal.
- At least five bundled scenarios.
- RK4 integrator.
- HORIZONS fetch and cache.
- Trails.
- HUD.
- Heatmap.
- CI.
- README demo.
- License.

Not required:

- Perfect performance.
- Full 3D camera.
- Windows support.
- crates.io publish.
- Adaptive time stepping.

## 10. Learning Checkpoints

### Phase 0 Learning Checkpoints

By the end of Phase 0, the developer should understand:

- How a Rust binary project is structured.
- How `cargo check`, `cargo fmt`, `cargo clippy`, and `cargo test` fit together.
- How `clap` turns structs into a real CLI.

Reflection questions:

- What does `cargo check` verify without producing a final binary?
- Why is `clippy -D warnings` useful before release?
- What belongs in `main.rs` versus a module?

### Phase 1 Learning Checkpoints

By the end of Phase 1, the developer should understand:

- Newtonian vector gravity.
- RK4 derivation and implementation.
- Rust ownership patterns for mutable simulation state.

Reflection questions:

- Why is Euler integration not good enough for orbits?
- What does `dr/dt = v` mean in code?
- Why should RK4 intermediate states avoid mutating the real bodies?

### Phase 2 Learning Checkpoints

By the end of Phase 2, the developer should understand:

- Terminal rendering with ratatui and crossterm.
- Event loops and input handling.
- Decoupling physics ticks from render frames.

Reflection questions:

- Why should rendering borrow state immutably?
- Why does fixed-step physics improve reproducibility?
- How does camera zoom map meters to terminal cells?

### Phase 3 Learning Checkpoints

By the end of Phase 3, the developer should understand:

- REST API requests with `reqwest`.
- JSON and text parsing with `serde`.
- Scientific reproducibility through cached input data.

Reflection questions:

- Why does HORIZONS return text inside a JSON wrapper?
- Why is barycentric data better for the full Solar System?
- What information must be cached to reproduce a run later?

### Phase 4 Learning Checkpoints

By the end of Phase 4, the developer should understand:

- Visual encoding of scientific data.
- Logarithmic scaling for field intensity.
- Mouse interaction in terminal applications.

Reflection questions:

- Why does the heatmap need logarithmic scaling?
- What makes a color palette scientifically readable?
- How can a HUD show uncertainty or numerical warnings honestly?

### Phase 5 Learning Checkpoints

By the end of Phase 5, the developer should understand:

- Barnes-Hut tree approximation.
- Algorithmic complexity from `O(n^2)` to `O(n log n)`.
- Benchmarking Rust code with Criterion.

Reflection questions:

- What does the `θ` parameter trade off?
- When is a distant group safe to approximate as one mass?
- Why can a faster algorithm be less accurate?

### Phase 6 Learning Checkpoints

By the end of Phase 6, the developer should understand:

- Open-source release hygiene.
- CI with GitHub Actions.
- Technical communication for scientific software.

Reflection questions:

- Can a new user run the project from the README alone?
- Does the demo show the science and the engineering?
- Are the limitations honest and easy to find?

## Suggested Implementation Order Inside Each Work Session

Use this rhythm:

1. Pick one small task.
2. Write or update a test if practical.
3. Implement the minimum code.
4. Run `cargo check`.
5. Run focused tests.
6. Refactor only after it works.
7. Update docs if behavior changed.
8. Commit with a clear message.

This keeps the project from becoming a pile of half-finished impressive ideas.

## Scientific Validation Plan

### Two-Body Circular Orbit Test

Create a small test system:

- One massive central body.
- One lighter orbiting body.
- Circular velocity:

```text
v = sqrt(G * M / r)
```

Expected:

- Radius remains approximately constant.
- Energy drift remains small.
- Orbital period roughly matches:

```text
T = 2π * sqrt(r^3 / (G * M))
```

### Earth-Moon Sanity Check

Expected:

- Moon remains bound to Earth.
- Period is near 27.3 days in simplified model.
- Earth moves around the barycenter, not perfectly fixed if both bodies are free.

### Solar System Sanity Check

Expected:

- Inner planets orbit faster than outer planets.
- Jupiter visibly dominates outer Solar System perturbations.
- Energy drift remains visible but small over short demo times.

### Figure-8 Sanity Check

Expected:

- Bodies trace the known figure-8 for a reasonable duration.
- Too-large `dt` destroys the orbit.
- This makes a good educational demo of numerical sensitivity.

## Testing Strategy

### Unit Tests

Test:

- Vector acceleration symmetry.
- Softening avoids infinite acceleration.
- Energy calculation for simple systems.
- Momentum calculation.
- Unit conversion.
- Scenario validation.
- HORIZONS parser fixtures.

### Integration Tests

Test:

- Load every bundled scenario.
- Run each scenario for 10 steps.
- Validate no NaN positions.
- Validate no infinite velocities.
- Validate CLI help.
- Validate `graviton validate`.

### Property Tests

Use `proptest` for:

- Random valid masses remain finite.
- Unit conversions round-trip within tolerance.
- Scenario validation rejects non-finite values.
- Center of mass is translation invariant.

### Snapshot Tests

Use `insta` for:

- Parser output from saved HORIZONS response.
- Scenario validation error formatting.
- CLI help output if stable enough.

### Benchmarks

Use `criterion` for:

- Direct acceleration with 10, 100, 1000 bodies.
- Barnes-Hut acceleration with 100, 1000, 10000 bodies.
- Heatmap sampling cost.
- Trail rendering cost.

## Performance Budget

### Early Version

Target direct simulation:

- 10 bodies: easy.
- 100 bodies: should be smooth.
- 1000 bodies: may be slow with direct `O(n^2)`.

### After Barnes-Hut

Target:

- 1000 bodies: smooth enough.
- 10000 bodies: benchmark/demo mode.

### Rendering Budget

Per frame:

- Project visible bodies.
- Draw trails.
- Draw HUD.
- Recompute heatmap only periodically.

Avoid:

- Allocating large buffers every frame.
- Formatting many strings every frame.
- Recomputing colors for unchanged trails unnecessarily.

## Configuration Strategy

Have separate config layers:

1. Scenario file.
2. User config file.
3. CLI overrides.
4. Runtime key changes.

Priority:

```text
runtime input > CLI flags > user config > scenario defaults
```

Possible user config path:

```text
~/.config/graviton/config.toml
```

Example:

```toml
[render]
true_color = true
unicode = true
mouse = true

[controls]
vim_keys = true

[cache]
offline = false
```

Do not implement global config until after scenario config works.

## CLI Design

Commands:

```text
graviton run <scenario>
graviton fetch solar-system --date <date>
graviton validate <scenario>
graviton list-scenarios
graviton bench
```

`run` flags:

```text
--headless
--steps <n>
--dt <seconds>
--integrator rk4
--barnes-hut
--theta <value>
--no-heatmap
--no-trails
```

`fetch` flags:

```text
--date YYYY-MM-DD
--center 500@0
--output scenarios/solar-system.toml
--offline
--force
```

`validate`:

```text
graviton validate scenarios/*.toml
```

## Documentation Standards

Every major module should start with a short module-level doc comment.

Example:

```rust
//! RK4 integration for Newtonian N-body systems.
//!
//! This module advances position and velocity using a fixed time step.
//! It intentionally exposes energy drift through diagnostics instead of
//! pretending the integrator is exact.
```

Public functions should explain:

- Inputs.
- Units.
- Panics if any.
- Error behavior if any.

Avoid:

- Over-commenting simple Rust syntax.
- Hiding equations in prose only.
- Using unexplained magic constants.

## Risk Register

### Risk: Physics Looks Stable But Is Wrong

Mitigation:

- Add diagnostics early.
- Test against simple known systems.
- Document limitations.

### Risk: HORIZONS Parsing Is Fragile

Mitigation:

- Save raw fixtures.
- Parse only known vector table format first.
- Show clear errors.

### Risk: TUI Complexity Sprawls

Mitigation:

- Keep renderer pure.
- Keep app state explicit.
- Avoid mixing input and physics.

### Risk: Scope Too Large for 8 Weeks

Mitigation:

- Treat Barnes-Hut as Phase 5, not Phase 1.
- Treat Kitty graphics as optional.
- Release with direct solver if needed.

### Risk: Performance Disappoints

Mitigation:

- Benchmark.
- Add Barnes-Hut.
- Reduce heatmap frequency.
- Cap trail points.

### Risk: Admissions Reader Cannot Understand It Quickly

Mitigation:

- Excellent README.
- Demo GIF.
- Short scientific explanation.
- Clear architecture diagram.
- Honest limitations.

## Final Definition of Success

The project succeeds if:

- A user can install and run it.
- The simulation is visually impressive in a terminal.
- The physics is explained and tested.
- Real NASA HORIZONS data is used reproducibly.
- The code is idiomatic enough to show serious Rust learning.
- The README tells a compelling story.
- The student can explain every major design choice in an interview.

The project does not need to be perfect.

It needs to be honest, beautiful, educational, and working.
