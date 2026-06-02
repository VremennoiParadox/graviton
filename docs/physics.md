# Physics model

orrery-tui simulates Newtonian gravitation in **3D** with a fixed time step. Internal state uses SI units everywhere.

## Gravity

Pairwise acceleration with Plummer softening:

```text
a_i = Σ_{j≠i}  G m_j r_ij / (|r_ij|² + ε²)^(3/2)
```

- `G` = 6.67430×10⁻¹¹ m³ kg⁻¹ s⁻²  
- `ε` = `softening_m` from the scenario (default 10⁶ m for Solar System scale)

### Direct summation

Default mode: O(n²) per acceleration evaluation. Accurate for small `n` (tens of bodies).

### Barnes–Hut

Optional octree approximation (O(n log n) typical). For each body, a tree node is used as a single mass when:

```text
s / d < θ
```

- `s` = width of the node’s cell  
- `d` = distance to the node’s center of mass  
- `θ` = `barnes_hut_theta` (default **0.7**; try **0.5** for accuracy, **1.0** for speed)

Enable in TOML or with `--barnes-hut`. Toggle at runtime with **`B`** in the TUI.

**Limitations:** Approximate; close encounters and extreme mass ratios can disagree with direct summation. The tree is rebuilt on each RK4 acceleration evaluation (simple, not yet pooled).

## Integration

**RK4** (fourth-order Runge–Kutta) with fixed `dt`:

- State: positions and velocities per body  
- Each step performs four acceleration evaluations  
- Simulation time advances by `dt` after a successful step

Non-finite positions or velocities abort the step with an error.

## Diagnostics

Each frame (or headless report) can show:

- Kinetic, potential, and total energy  
- Energy drift relative to initial total energy  
- Linear momentum and center of mass  
- Orbital estimates in the HUD (two-body approximation)

Energy is **not** strictly conserved (fixed `dt`, softening, Barnes–Hut error). Small relative drift on bound orbits is expected; large drift often means `dt` is too large or the system is escaping.

## Field heatmap (visualization)

The TUI heatmap shows |**g**| at sample points using the same softened summation as direct gravity. Values are mapped with **log₁₀** scaling so weak and strong regions are visible together. This is diagnostic only and does not affect the simulation.

## Validation tests

The repository includes:

- Two-body symmetry and 1/r² scaling tests  
- Earth–Moon radius stability over many steps  
- Barnes–Hut vs direct acceleration comparison  
- Short-run energy drift comparison (direct vs Barnes–Hut)

Run: `cargo test`

## Known limitations (honest list)

- Point masses only (no tides, spin, GR, radiation pressure)  
- No collision handling beyond softening  
- HORIZONS vectors are instantaneous; no automatic ephemeris update during simulation  
- Figure-eight and other chaotic systems need small `dt`  
- Barnes–Hut is not symplectic; long runs may differ from direct integration  
- Heatmap samples the projection plane (z = 0 in the active 2D view for XY)

For teaching and portfolio demos these tradeoffs are intentional; see [PLANNING.md](../PLANNING.md) for future work.
