// src/main.rs
mod domain {
    pub mod vec2;
    pub mod particle;
    pub mod material;
}

mod grid {
    pub mod grid;
}

mod usecase {
    pub mod update;
}
mod utils {
    pub mod colors;
}

use domain::particle::{Particle, Phase};
use domain::vec2::Vec2;
use grid::grid::Grid;
use usecase::update::update_particles;
use minifb::{Key, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const NUM_PARTICLES: usize = 10000;
const DT: f32 = 0.016* 0.5; // 30 FPSs

use utils::colors::{noisy_color, darken, lighten, phase_color};


use domain::material::{SAND, WATER, Material};





fn main() {
    let mut buffer = vec![0x000000; WIDTH * HEIGHT];
    let mut window = Window::new("Physics Particle Sim", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Failed to create window");

    let mut rng = rand::thread_rng();
    let materials = [SAND, WATER];

    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|i| {
            let material = materials[i % materials.len()];
            Particle {
                position: Vec2::new(rng.gen_range(50.0..250.0), rng.gen_range(0.0..100.0)),
                velocity: Vec2::zero(),
                acceleration: Vec2::zero(),
                mass: material.mass,
                radius: 1.0,
                temperature: 20.0,
                phase: material.phase,
                color: phase_color(material.base_color, material.phase),
            }
        })
        .collect();

    let mut grid = Grid::new(WIDTH, HEIGHT);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0x000000);
        update_particles(&mut particles, &mut grid, DT);

        for p in particles.iter() {
            let x = p.position.x as usize;
            let y = p.position.y as usize;
            if x < WIDTH && y < HEIGHT {
                buffer[y * WIDTH + x] = p.color;
            }
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}