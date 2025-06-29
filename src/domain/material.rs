// src/domain/material.rs
use crate::domain::particle::Phase;

#[derive(Clone, Copy)]
pub struct Material {
    pub name: &'static str,
    pub phase: Phase,
    pub base_color: u32,
    pub mass: f32,
}


// src/domain/material.rs (continued)
pub const SAND: Material = Material {
    name: "silica_sand",
    phase: Phase::Solid,
    base_color: 0xC2B280,
    mass: 2.65, // g/cm³ equivalent
};

pub const WATER: Material = Material {
    name: "h2o",
    phase: Phase::Liquid,
    base_color: 0x3399FF,
    mass: 1.0,
};
