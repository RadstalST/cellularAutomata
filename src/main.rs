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

use domain::particle::Particle;
use domain::vec2::Vec2;
use domain::material::{Material, SAND, WATER};
use grid::grid::Grid;
use usecase::update::update_particles;
use minifb::{Key, Window, WindowOptions};
use utils::colors::phase_color;
use rand::Rng;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const NUM_PARTICLES: usize = 10000;
const DT: f32 = 0.0; 

pub fn spawn_particles(
    materials: &[Material],
    count: usize,
    width: usize,
    height: usize,
    grid: &mut Grid,
) -> Vec<Particle> {
    let mut rng = rand::thread_rng();
    let mut particles = Vec::with_capacity(count);

    let mut attempts = 0;
    let max_attempts = count * 10;

    for i in 0..count {
        let material = materials[i % materials.len()];
        let mut placed = false;

        while !placed && attempts < max_attempts {
            let x = rng.gen_range(50..(width - 50));
            let y = rng.gen_range(0..100);
            if !grid.is_occupied(x, y) {
                grid.set(x, y, Some(i));
                particles.push(Particle {
                    position: Vec2::new(x as f32, y as f32),
                    velocity: Vec2::zero(),
                    acceleration: Vec2::zero(),
                    mass: material.mass,
                    radius: 1.0,
                    temperature: 20.0,
                    phase: material.phase,
                    color: phase_color(material.base_color, material.phase),
                });
                placed = true;
            }
            attempts += 1;
        }
    }

    particles
}

fn main() {
    let mut buffer = vec![0x000000; WIDTH * HEIGHT];
    let mut window = Window::new("Physics Particle Sim", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Failed to create window");

    let materials = [SAND,WATER,WATER,WATER,WATER,WATER];
    let mut grid = Grid::new(WIDTH, HEIGHT);
    let mut particles = spawn_particles(&materials, NUM_PARTICLES, WIDTH, HEIGHT, &mut grid);


    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(0x000000);
        update_particles(&mut particles, &mut grid, DT);

        for p in particles.iter_mut() {
            p.position.x = p.position.x.clamp(0.0, (WIDTH - 1) as f32);
            p.position.y = p.position.y.clamp(0.0, (HEIGHT - 1) as f32);

            let x = p.position.x as usize;
            let y = p.position.y as usize;
            if x < WIDTH && y < HEIGHT {
                buffer[y * WIDTH + x] = p.color;
            }
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
