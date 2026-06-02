//! Barnes–Hut octree acceleration (3D, softened Newtonian gravity).

use glam::DVec3;

use crate::physics::body::Body;
use crate::physics::constants::G;

const MIN_HALF_WIDTH: f64 = 1.0;
const MAX_DEPTH: usize = 32;

/// Statistics from a built octree (debug HUD).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeStats {
    pub nodes: usize,
    pub max_depth: usize,
}

/// Compute accelerations with Barnes–Hut for all bodies at `positions`.
#[must_use]
pub fn accelerations(
    bodies: &[Body],
    positions: &[DVec3],
    softening_m: f64,
    theta: f64,
) -> Vec<DVec3> {
    let n = bodies.len();
    if n == 0 {
        return Vec::new();
    }
    let theta = theta.clamp(0.0, 2.0);
    let eps_sq = softening_m * softening_m;
    let masses: Vec<f64> = bodies.iter().map(|b| b.mass_kg).collect();
    let (root, _) = build_tree(positions, &masses);
    (0..n)
        .map(|i| root.acceleration(i, positions[i], positions, bodies, eps_sq, theta))
        .collect()
}

/// Build an octree and return root + stats (for diagnostics).
#[must_use]
pub fn build_tree(positions: &[DVec3], masses: &[f64]) -> (OctNode, TreeStats) {
    let (center, half_width) = bounding_cube(positions);
    let mut root = OctNode::empty(center, half_width);
    let mut stats = TreeStats::default();

    for (i, &pos) in positions.iter().enumerate() {
        root.insert(i, pos, masses[i], positions, masses, 0, &mut stats);
    }

    stats.nodes = stats.nodes.max(1);
    (root, stats)
}

/// Octree node (public for optional debug visualization).
#[derive(Debug, Clone)]
pub struct OctNode {
    center: DVec3,
    half_width: f64,
    mass: f64,
    com: DVec3,
    body: Option<usize>,
    children: Option<Box<[OctNode; 8]>>,
}

impl OctNode {
    fn empty(center: DVec3, half_width: f64) -> Self {
        Self {
            center,
            half_width,
            mass: 0.0,
            com: DVec3::ZERO,
            body: None,
            children: None,
        }
    }

    fn is_empty(&self) -> bool {
        self.mass == 0.0 && self.body.is_none() && self.children.is_none()
    }

    #[allow(clippy::too_many_arguments)]
    fn insert(
        &mut self,
        body_idx: usize,
        pos: DVec3,
        mass: f64,
        positions: &[DVec3],
        masses: &[f64],
        depth: usize,
        stats: &mut TreeStats,
    ) {
        stats.nodes += 1;
        stats.max_depth = stats.max_depth.max(depth);

        if self.is_empty() {
            self.body = Some(body_idx);
            self.mass = mass;
            self.com = pos;
            return;
        }

        if self.body.is_some() && self.children.is_none() {
            if depth >= MAX_DEPTH {
                self.merge_leaf(body_idx, pos, mass, positions, masses);
                return;
            }
            let old_idx = self.body.take().expect("leaf");
            let old_pos = positions[old_idx];
            let old_mass = masses[old_idx];
            self.subdivide();
            self.insert(
                old_idx,
                old_pos,
                old_mass,
                positions,
                masses,
                depth + 1,
                stats,
            );
        }

        if let Some(children) = self.children.as_mut() {
            let octant = child_octant(self.center, pos);
            children[octant].insert(body_idx, pos, mass, positions, masses, depth + 1, stats);
            self.aggregate_from_children();
        }
    }

    fn merge_leaf(
        &mut self,
        body_idx: usize,
        pos: DVec3,
        mass: f64,
        positions: &[DVec3],
        masses: &[f64],
    ) {
        if let Some(existing) = self.body {
            let m0 = self.mass;
            let m1 = mass;
            let new_mass = m0 + m1;
            self.com = (self.com * m0 + pos * m1) / new_mass;
            self.mass = new_mass;
            let _ = (existing, body_idx, positions, masses);
            self.body = None;
        } else {
            self.body = Some(body_idx);
            self.mass = mass;
            self.com = pos;
        }
    }

    fn subdivide(&mut self) {
        let hw = self.half_width * 0.5;
        let mut children = Vec::with_capacity(8);
        for oz in 0..2 {
            for oy in 0..2 {
                for ox in 0..2 {
                    let offset = DVec3::new(
                        if ox == 0 { -hw } else { hw },
                        if oy == 0 { -hw } else { hw },
                        if oz == 0 { -hw } else { hw },
                    );
                    children.push(OctNode::empty(self.center + offset, hw));
                }
            }
        }
        let arr: [OctNode; 8] = children.try_into().expect("8 children");
        self.children = Some(Box::new(arr));
        self.body = None;
    }

    fn aggregate_from_children(&mut self) {
        if let Some(children) = &self.children {
            self.mass = 0.0;
            self.com = DVec3::ZERO;
            for child in children.iter() {
                if child.mass > 0.0 {
                    self.com += child.com * child.mass;
                    self.mass += child.mass;
                }
            }
            if self.mass > 0.0 {
                self.com /= self.mass;
            }
        }
    }

    fn acceleration(
        &self,
        body_idx: usize,
        position: DVec3,
        positions: &[DVec3],
        bodies: &[Body],
        eps_sq: f64,
        theta: f64,
    ) -> DVec3 {
        if self.mass <= 0.0 {
            return DVec3::ZERO;
        }

        let r = self.com - position;
        let dist = r.length();
        let size = self.half_width * 2.0;

        if self.children.is_none() {
            if let Some(other) = self.body {
                if other == body_idx {
                    return DVec3::ZERO;
                }
                let r_ij = positions[other] - position;
                let dist_sq = r_ij.length_squared() + eps_sq;
                let inv_dist3 = 1.0 / (dist_sq * dist_sq.sqrt());
                return G * bodies[other].mass_kg * r_ij * inv_dist3;
            }
            return DVec3::ZERO;
        }

        if dist > 0.0 && size / dist < theta {
            let dist_sq = dist * dist + eps_sq;
            let inv_dist3 = 1.0 / (dist_sq * dist_sq.sqrt());
            return G * self.mass * r * inv_dist3;
        }

        let mut acc = DVec3::ZERO;
        if let Some(children) = &self.children {
            for child in children.iter() {
                if child.mass > 0.0 {
                    acc += child.acceleration(body_idx, position, positions, bodies, eps_sq, theta);
                }
            }
        }
        acc
    }
}

fn child_octant(center: DVec3, pos: DVec3) -> usize {
    let mut idx = 0;
    if pos.x >= center.x {
        idx |= 1;
    }
    if pos.y >= center.y {
        idx |= 2;
    }
    if pos.z >= center.z {
        idx |= 4;
    }
    idx
}

fn bounding_cube(positions: &[DVec3]) -> (DVec3, f64) {
    let mut min = positions[0];
    let mut max = positions[0];
    for &p in positions.iter().skip(1) {
        min = min.min(p);
        max = max.max(p);
    }
    let center = (min + max) * 0.5;
    let span = max - min;
    let mut half = (span.x.max(span.y).max(span.z) * 0.5).max(MIN_HALF_WIDTH);
    if !half.is_finite() || half <= 0.0 {
        half = MIN_HALF_WIDTH;
    }
    (center, half)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::physics::body::{Body, BodyClass};
    use crate::physics::constants::M_EARTH;
    use crate::physics::gravity::direct_accelerations as direct;

    fn body(id: &str, mass: f64, pos: DVec3) -> Body {
        Body {
            id: id.into(),
            name: id.into(),
            mass_kg: mass,
            radius_m: 1.0,
            position_m: pos,
            velocity_mps: DVec3::ZERO,
            class: BodyClass::Planet,
            color_rgb: None,
        }
    }

    #[test]
    fn matches_direct_for_two_bodies() {
        let softening = 1.0;
        let bodies = vec![
            body("a", M_EARTH, DVec3::ZERO),
            body("b", M_EARTH, DVec3::new(3.84e8, 0.0, 0.0)),
        ];
        let positions: Vec<_> = bodies.iter().map(|b| b.position_m).collect();
        let direct_acc = direct(&bodies, &positions, softening);
        let bh_acc = accelerations(&bodies, &positions, softening, 0.3);
        for i in 0..2 {
            let diff = (direct_acc[i] - bh_acc[i]).length();
            let scale = direct_acc[i].length().max(1e-12);
            assert!(
                diff / scale < 0.02,
                "body {i}: diff ratio {} direct {:?} bh {:?}",
                diff / scale,
                direct_acc[i],
                bh_acc[i]
            );
        }
    }

    #[test]
    fn matches_direct_for_random_cluster() {
        let softening = 1000.0;
        let mut bodies = Vec::new();
        let mut positions = Vec::new();
        for i in 0..24 {
            let angle = i as f64 * 0.7;
            let r = 1.0e9 + (i as f64) * 4.0e7;
            let pos = DVec3::new(r * angle.cos(), r * angle.sin(), (i as f64 - 12.0) * 1.0e7);
            let mass = 1.0e22 + (i as f64) * 1.0e20;
            bodies.push(body(&format!("b{i}"), mass, pos));
            positions.push(pos);
        }
        let direct_acc = direct(&bodies, &positions, softening);
        let bh_acc = accelerations(&bodies, &positions, softening, 0.5);
        let mut max_rel = 0.0_f64;
        for i in 0..bodies.len() {
            let diff = (direct_acc[i] - bh_acc[i]).length();
            let scale = direct_acc[i].length().max(1e-12);
            max_rel = max_rel.max(diff / scale);
        }
        assert!(max_rel < 0.05, "max relative acceleration error {max_rel}");
    }

    #[test]
    fn theta_zero_matches_direct_closely() {
        let softening = 100.0;
        let bodies = vec![
            body("a", 1.0e24, DVec3::new(-1.0e8, 0.0, 0.0)),
            body("b", 1.0e24, DVec3::new(1.0e8, 0.0, 0.0)),
            body("c", 1.0e23, DVec3::new(0.0, 1.0e8, 0.0)),
        ];
        let positions: Vec<_> = bodies.iter().map(|b| b.position_m).collect();
        let direct_acc = direct(&bodies, &positions, softening);
        let bh_acc = accelerations(&bodies, &positions, softening, 0.01);
        for i in 0..bodies.len() {
            let rel = (direct_acc[i] - bh_acc[i]).length() / direct_acc[i].length().max(1e-12);
            assert!(rel < 0.01, "theta~0 body {i} rel err {rel}");
        }
    }
}
