mod domain;
mod usecase;

use domain::particle::{Particle, ParticleType};
use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use std::collections::HashMap;
use usecase::collision::{resolve_collisions, CellCoord};

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const NUM_PARTICLES: usize = 1000;
const RADIUS: f32 = 3.0;
const CELL_SIZE: usize = 8;

fn main() {
    let mut buffer = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Pixel Sim", WIDTH, HEIGHT, WindowOptions::default()).unwrap();

    let mut rng = rand::thread_rng();
fn initialize_particles() -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    (0..NUM_PARTICLES)
        .map(|i| {
            let kind = match i % 3 {
                0 => ParticleType::Sand,
                1 => ParticleType::Water,
                _ => ParticleType::Fire,
            };
            let color = match kind {
                ParticleType::Sand => 0xC2B280,
                ParticleType::Water => 0x3399FF,
                ParticleType::Fire => 0xFF6600,
            };

            Particle {
                x: rng.gen_range(100.0..200.0),
                y: rng.gen_range(0.0..10.0),
                vx: rng.gen_range(-1.0..1.0),
                vy: rng.gen_range(0.0..2.0),
                kind,
                color,
            }
        })
        .collect()
}

let mut particles = initialize_particles();

    use std::time::{Duration, Instant};

let mut last_reinit = Instant::now();

while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0);

// Debounce logic for "R" key
if window.is_key_down(Key::R) && last_reinit.elapsed() > Duration::from_millis(300) {
    particles = initialize_particles();
    last_reinit = Instant::now();
}

        let mut grid: HashMap<CellCoord, Vec<usize>> = HashMap::new();
        for (i, p) in particles.iter().enumerate() {
            let cx = (p.x / CELL_SIZE as f32).floor() as i32;
            let cy = (p.y / CELL_SIZE as f32).floor() as i32;
            grid.entry((cx, cy)).or_default().push(i);
        }

        resolve_collisions(&mut particles, &grid, RADIUS);

        for p in particles.iter_mut() {
            p.update(WIDTH, HEIGHT);
            p.draw(&mut buffer, WIDTH, HEIGHT);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
