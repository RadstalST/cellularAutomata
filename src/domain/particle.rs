use crate::domain::vec2::Vec2;
use bytemuck::{Pod, Zeroable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Solid,
    Liquid,
    Gas,
    Plasma,
}

impl From<u32> for Phase {
    fn from(value: u32) -> Self {
        match value {
            0 => Phase::Solid,
            1 => Phase::Liquid,
            2 => Phase::Gas,
            3 => Phase::Plasma,
            _ => Phase::Solid,
        }
    }
}

impl From<Phase> for u32 {
    fn from(phase: Phase) -> Self {
        match phase {
            Phase::Solid => 0,
            Phase::Liquid => 1,
            Phase::Gas => 2,
            Phase::Plasma => 3,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Particle {
    pub position: Vec2,
    pub velocity: Vec2,
    pub acceleration: Vec2,
    pub mass: f32,
    pub radius: f32,
    pub temperature: f32,
    pub phase: Phase,
    pub color: u32,
}

/// GPU-safe version (must match WGSL layout exactly)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct GpuParticle {
    pub position: [f32; 2],
    pub velocity: [f32; 2],
    pub acceleration: [f32; 2],
    pub mass: f32,
    pub radius: f32,
    pub temperature: f32,
    pub phase: u32,
    pub color: u32,
    _padding: u32, // <-- 4-byte padding for 16-byte alignment
}

impl From<Particle> for GpuParticle {
    fn from(p: Particle) -> Self {
        Self {
            position: [p.position.x, p.position.y],
            velocity: [p.velocity.x, p.velocity.y],
            acceleration: [p.acceleration.x, p.acceleration.y],
            mass: p.mass,
            radius: p.radius,
            temperature: p.temperature,
            phase: p.phase.into(),
            color: p.color,
            _padding: 0,
        }
    }
}

impl From<GpuParticle> for Particle {
    fn from(gp: GpuParticle) -> Self {
        Self {
            position: Vec2::new(gp.position[0], gp.position[1]),
            velocity: Vec2::new(gp.velocity[0], gp.velocity[1]),
            acceleration: Vec2::new(gp.acceleration[0], gp.acceleration[1]),
            mass: gp.mass,
            radius: gp.radius,
            temperature: gp.temperature,
            phase: gp.phase.into(),
            color: gp.color,
        }
    }
}

impl Particle {
    pub fn apply_force(&mut self, force: Vec2) {
        let acc = force.scale(1.0 / self.mass);
        self.acceleration = self.acceleration.add(acc);
    }

    pub fn integrate(&mut self, dt: f32, bounds: (usize, usize)) {
        self.velocity = self.velocity.add(self.acceleration.scale(dt));
        self.position = self.position.add(self.velocity.scale(dt));
        self.acceleration = Vec2::zero();

        let (max_x, max_y) = (bounds.0 as f32, bounds.1 as f32);

        if self.position.x < 0.0 {
            self.position.x = 0.0;
            self.velocity.x *= -0.5;
        } else if self.position.x >= max_x {
            self.position.x = max_x - 1.0;
            self.velocity.x *= -0.5;
        }

        if self.position.y < 0.0 {
            self.position.y = 0.0;
            self.velocity.y *= -0.5;
        } else if self.position.y >= max_y {
            self.position.y = max_y - 1.0;
            self.velocity.y *= -0.5;
        }
    }
}
