// --- domain/particle.rs ---
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParticleType {
    Sand,
    Water,
    Fire,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub kind: ParticleType,
    pub color: u32,
}

impl Particle {
    pub fn update(&mut self, width: usize, height: usize) {
        match self.kind {
            ParticleType::Sand => self.vy += 0.1, // Sand falls faster than water
            ParticleType::Water => self.vy += 0.05, // Water falls slower than sand
            ParticleType::Fire => {
                self.vy -= 0.02;
                self.vx += rand::random::<f32>() * 0.2 - 0.1;
            }
        }

        self.x += self.vx;
        self.y += self.vy;

        if self.y >= (height - 1) as f32 {
            self.y = (height - 1) as f32;
            self.vy *= -0.6;
        }
        if self.x < 0.0 {
            self.x = 0.0;
            self.vx *= -1.0;
        }
        if self.x >= (width - 1) as f32 {
            self.x = (width - 1) as f32;
            self.vx *= -1.0;
        }
    }

    pub fn draw(&self, buffer: &mut [u32], width: usize, height: usize) {
        let x = self.x as usize;
        let y = self.y as usize;
        if x < width && y < height {
            let index = y * width + x;
            buffer[index] = self.color;
        }
    }
}