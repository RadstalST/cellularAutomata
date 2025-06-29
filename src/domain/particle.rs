use crate::domain::vec2::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Solid,
    Liquid,
    Gas,
    Plasma,
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