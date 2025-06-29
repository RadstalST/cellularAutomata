mod domain {
    pub mod material;
    pub mod particle;
    pub mod render;
    pub mod vec2;
}

mod grid {
    pub mod grid;
    pub mod occupancy;
}

mod gpu {
    pub mod compute;
}

mod usecase {
    pub mod update;
}

mod utils {
    pub mod colors;
}

use domain::material::{Material, SAND, WATER};
use domain::particle::Particle;
use domain::vec2::Vec2;
use grid::grid::Grid;
use gpu::compute::{initialize_gpu, dispatch_particles};
use minifb::{Key, Window, WindowOptions};
use utils::colors::phase_color;

use rand::Rng;
use wgpu::util::DeviceExt;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const NUM_PARTICLES: usize = 10000;
const DT: f32 = 0.016;

fn spawn_particles(materials: &[Material], count: usize, width: usize, height: usize, grid: &mut Grid) -> Vec<Particle> {
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
            if x < width && y < height && !grid.is_occupied(x, y) {
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

fn render_particles(buffer: &mut [u32], particles: &[Particle]) {
    buffer.fill(0x000000);
    for p in particles.iter() {
        let x = p.position.x as usize;
        let y = p.position.y as usize;
        if x < WIDTH && y < HEIGHT {
            buffer[y * WIDTH + x] = p.color;
        }
    }
}

fn main() {
    // Set up window and buffer
    let mut buffer = vec![0x000000; WIDTH * HEIGHT];
    let mut window = Window::new("Physics Particle Sim (GPU)", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Failed to create window");

    // Spawn initial particles
    let materials = [SAND, WATER, WATER, WATER, SAND];
    let mut grid = Grid::new(WIDTH, HEIGHT);
    let mut particles = spawn_particles(&materials, NUM_PARTICLES, WIDTH, HEIGHT, &mut grid);
    let mut occupancy = vec![0u32; WIDTH * HEIGHT];

    // Initialize GPU context (block on async)
    let (device, queue, shader) = pollster::block_on(initialize_gpu());

    // Main simulation loop
    while window.is_open() && !window.is_key_down(Key::Escape) {
        let (updated_particles, updated_occupancy) = pollster::block_on(dispatch_particles(
            &device,
            &queue,
            &shader,
            &particles,
            &occupancy,
        ));
        particles = updated_particles;
        occupancy = updated_occupancy;
        render_particles(&mut buffer, &particles);
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
