use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use std::collections::HashMap;

const WIDTH: usize = 300;
const HEIGHT: usize = 300;
const NUM_PARTICLES: usize = 1000;
const RADIUS: f32 = 3.0;
const RADIUS_SQ: f32 = RADIUS * RADIUS;
const CELL_SIZE: usize = 8;

type CellCoord = (i32, i32);


#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

fn resolve_collisions(particles: &mut Vec<Particle>, grid: &HashMap<CellCoord, Vec<usize>>) {
    let neighbor_offsets = [-1, 0, 1];

    for ((cell_x, cell_y), indices) in grid.iter() {
        for &i in indices {
            for dx in &neighbor_offsets {
                for dy in &neighbor_offsets {
                    let neighbor_cell = (cell_x + dx, cell_y + dy);
                    if let Some(neighbors) = grid.get(&neighbor_cell) {
                        for &j in neighbors {
                            if i >= j {
                                continue; // avoid double checking
                            }

                            let (p1, p2) = {
                                let (head, tail) = particles.split_at_mut(j.max(i));
                                if i < j {
                                    (&mut head[i], &mut tail[0])
                                } else {
                                    (&mut tail[0], &mut head[j])
                                }
                            };

                            let dx = p2.x - p1.x;
                            let dy = p2.y - p1.y;
                            let dist_sq = dx * dx + dy * dy;

                            if dist_sq < RADIUS * RADIUS && dist_sq > 0.0 {
                                let dist = dist_sq.sqrt();
                                let overlap = 0.5 * (RADIUS - dist);
                                let nx = dx / dist;
                                let ny = dy / dist;

                                p1.x -= overlap * nx;
                                p1.y -= overlap * ny;
                                p2.x += overlap * nx;
                                p2.y += overlap * ny;

                                let dvx = p2.vx - p1.vx;
                                let dvy = p2.vy - p1.vy;
                                let impact = dvx * nx + dvy * ny;

                                if impact < 0.0 {
                                    let impulse = 0.5 * impact;
                                    p1.vx += impulse * nx;
                                    p1.vy += impulse * ny;
                                    p2.vx -= impulse * nx;
                                    p2.vy -= impulse * ny;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


impl Particle {
    fn update(&mut self) {
        self.vy += 0.1; // gravity
        self.x += self.vx;
        self.y += self.vy;

        // bounce off the floor
        if self.y >= (HEIGHT - 1) as f32 {
            self.y = (HEIGHT - 1) as f32;
            self.vy *= -0.6; // lose energy on bounce
        }

        // bounce off walls
        if self.x < 0.0 {
            self.x = 0.0;
            self.vx *= -1.0;
        }
        if self.x >= (WIDTH - 1) as f32 {
            self.x = (WIDTH - 1) as f32;
            self.vx *= -1.0;
        }
    }

    fn draw(&self, buffer: &mut [u32]) {
        let x = self.x as usize;
        let y = self.y as usize;
        if x < WIDTH && y < HEIGHT {
            let index = y * WIDTH + x;
            buffer[index] = 0xFFFFFF; // white pixel
        }
    }
}

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Pixel Particles", WIDTH, HEIGHT, WindowOptions::default())
        .expect("Unable to create window");

    // create particles
    let mut rng = rand::thread_rng();
    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| Particle {
            x: rng.gen_range(100.0..200.0),
            y: rng.gen_range(0.0..10.0),
            vx: rng.gen_range(-1.0..1.0),
            vy: rng.gen_range(0.0..2.0),
        })
        .collect();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // clear screen (black)
        buffer.fill(0x000000);
        let mut spatial_map: HashMap<CellCoord, Vec<usize>> = HashMap::new();
        for (i, p) in particles.iter().enumerate() {
            let cell_x = (p.x / CELL_SIZE as f32).floor() as i32;
            let cell_y = (p.y / CELL_SIZE as f32).floor() as i32;
            spatial_map.entry((cell_x, cell_y)).or_default().push(i);
        }
        resolve_collisions(&mut particles, &spatial_map);

        // update + draw each particle
        for p in particles.iter_mut() {
            p.update();
            p.draw(&mut buffer);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
