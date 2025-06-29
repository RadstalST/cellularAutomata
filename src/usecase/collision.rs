use crate::domain::particle::Particle;
use std::collections::HashMap;

pub type CellCoord = (i32, i32);

pub fn resolve_collisions(particles: &mut Vec<Particle>, grid: &HashMap<CellCoord, Vec<usize>>, radius: f32) {
    let neighbor_offsets = [-1, 0, 1];

    for ((cell_x, cell_y), indices) in grid.iter() {
        for &i in indices {
            for dx in &neighbor_offsets {
                for dy in &neighbor_offsets {
                    let neighbor_cell = (cell_x + dx, cell_y + dy);
                    if let Some(neighbors) = grid.get(&neighbor_cell) {
                        for &j in neighbors {
                            if i >= j {
                                continue;
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
                            let radius_sq = radius * radius;

                            if dist_sq < radius_sq && dist_sq > 0.0 {
                                let dist = dist_sq.sqrt();
                                let overlap = 0.5 * (radius - dist);
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
let damping = 0.9; // Add damping factor
let impulse = damping * 0.5 * impact;
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
