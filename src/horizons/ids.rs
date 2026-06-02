//! NASA JPL HORIZONS command IDs and local physical properties.

/// Catalog entry for a gravitating body.
#[derive(Debug, Clone, Copy)]
pub struct HorizonsBody {
    /// HORIZONS `COMMAND` ID (e.g. `399` for Earth).
    pub horizons_id: &'static str,
    pub id: &'static str,
    pub name: &'static str,
    pub class: &'static str,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub color: &'static str,
}

/// Default Solar System preset from PLANNING.md (inner + outer planets, Moon, Pluto).
pub fn solar_system_default() -> &'static [HorizonsBody] {
    &[
        SUN, MERCURY, VENUS, EARTH, MOON, MARS, JUPITER, SATURN, URANUS, NEPTUNE, PLUTO,
    ]
}

pub const SUN: HorizonsBody = HorizonsBody {
    horizons_id: "10",
    id: "sun",
    name: "Sun",
    class: "star",
    mass_kg: 1.988_92e30,
    radius_m: 696_340_000.0,
    color: "#fff4ea",
};

pub const MERCURY: HorizonsBody = HorizonsBody {
    horizons_id: "199",
    id: "mercury",
    name: "Mercury",
    class: "planet",
    mass_kg: 3.3011e23,
    radius_m: 2_439_700.0,
    color: "#8c7f70",
};

pub const VENUS: HorizonsBody = HorizonsBody {
    horizons_id: "299",
    id: "venus",
    name: "Venus",
    class: "planet",
    mass_kg: 4.8675e24,
    radius_m: 6_051_800.0,
    color: "#d9c27f",
};

pub const EARTH: HorizonsBody = HorizonsBody {
    horizons_id: "399",
    id: "earth",
    name: "Earth",
    class: "planet",
    mass_kg: 5.972_19e24,
    radius_m: 6_371_000.0,
    color: "#3d8bfd",
};

pub const MOON: HorizonsBody = HorizonsBody {
    horizons_id: "301",
    id: "moon",
    name: "Moon",
    class: "moon",
    mass_kg: 7.342e22,
    radius_m: 1_737_400.0,
    color: "#c0c0c0",
};

pub const MARS: HorizonsBody = HorizonsBody {
    horizons_id: "499",
    id: "mars",
    name: "Mars",
    class: "planet",
    mass_kg: 6.4171e23,
    radius_m: 3_389_500.0,
    color: "#c1440e",
};

pub const JUPITER: HorizonsBody = HorizonsBody {
    horizons_id: "599",
    id: "jupiter",
    name: "Jupiter",
    class: "planet",
    mass_kg: 1.8982e27,
    radius_m: 69_911_000.0,
    color: "#d8a45f",
};

pub const SATURN: HorizonsBody = HorizonsBody {
    horizons_id: "699",
    id: "saturn",
    name: "Saturn",
    class: "planet",
    mass_kg: 5.6834e26,
    radius_m: 58_232_000.0,
    color: "#e3c16f",
};

pub const URANUS: HorizonsBody = HorizonsBody {
    horizons_id: "799",
    id: "uranus",
    name: "Uranus",
    class: "planet",
    mass_kg: 8.6810e25,
    radius_m: 25_362_000.0,
    color: "#7fdbff",
};

pub const NEPTUNE: HorizonsBody = HorizonsBody {
    horizons_id: "899",
    id: "neptune",
    name: "Neptune",
    class: "planet",
    mass_kg: 1.02413e26,
    radius_m: 24_622_000.0,
    color: "#4169e1",
};

pub const PLUTO: HorizonsBody = HorizonsBody {
    horizons_id: "999",
    id: "pluto",
    name: "Pluto",
    class: "dwarf_planet",
    mass_kg: 1.303e22,
    radius_m: 1_188_300.0,
    color: "#b89b72",
};
