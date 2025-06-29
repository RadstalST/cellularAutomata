// src/usecase/update.rs
use crate::domain::particle::{Particle, Phase};
use crate::domain::vec2::Vec2;
use crate::grid::grid::Grid;
use rand::random;

pub fn update_particles(particles: &mut [Particle], grid: &mut Grid, dt: f32) {
    grid.clear();

    // Pass 1: Liquids
    for (i, p) in particles.iter_mut().enumerate() {
        if matches!(p.phase, Phase::Liquid) {
            update_liquid_particle(p, i, grid);
        }
    }

    // Pass 2: Solids
    for i in 0..particles.len() {
        if matches!(particles[i].phase, Phase::Solid) {
            update_solid_particle(i, particles, grid);
        }
    }

    // Pass 3: Gas and Plasma
    for (i, p) in particles.iter_mut().enumerate() {
        if matches!(p.phase, Phase::Gas | Phase::Plasma) {
            update_gas_particle(p, i, grid, dt);
        }
    }
}
pub fn update_liquid_particle(p: &mut Particle, i: usize, grid: &mut Grid) {
    let x = p.position.x.floor() as usize;
    let y = p.position.y.floor() as usize;

    // 1. Try moving down
    if y + 1 < grid.height && !grid.is_occupied(x, y + 1) {
        p.position.y += 1.0;
        grid.set(x, y + 1, Some(i));
        return;
    }

    let directions = if random::<bool>() { [-1, 1] } else { [1, -1] };

    // 2. Try moving diagonally down-left or down-right
    for dx in directions {
        let new_x = x.wrapping_add_signed(dx);
        if new_x < grid.width && y + 1 < grid.height && !grid.is_occupied(new_x, y + 1) {
            p.position.x += dx as f32;
            p.position.y += 1.0;
            grid.set(new_x, y + 1, Some(i));
            return;
        }
    }

    // 3. Try moving left or right
    for dx in directions {
        let new_x = x.wrapping_add_signed(dx);
        if new_x < grid.width && !grid.is_occupied(new_x, y) {
            p.position.x += dx as f32;
            grid.set(new_x, y, Some(i));
            return;
        }
    }

    // 4. No move possible, stay in place
    grid.set(x, y, Some(i));
}

fn update_solid_particle(i: usize, particles: &mut [Particle], grid: &mut Grid) {
    let (x, y) = {
        let p = &particles[i];
        (p.position.x.floor() as usize, p.position.y.floor() as usize)
    };

    if y + 1 < grid.height {
        match grid.get(x, y + 1) {
            None => {
                particles[i].position.y += 1.0;
                grid.set(x, y + 1, Some(i));
                return;
            }
            Some(j) => {
                if i != j && j < particles.len() && particles[i].mass > particles[j].mass {
                    // Swap positions based on density
                    let temp = particles[i].position;
                    particles[i].position = particles[j].position;
                    particles[j].position = temp;

                    grid.set(x, y + 1, Some(i));
                    grid.set(x, y, Some(j));
                    return;
                }
            }
        }
    }

    let directions = if random::<bool>() { [-1, 1] } else { [1, -1] };
    for dx in directions {
        let nx = x.wrapping_add_signed(dx);
        let ny = y + 1;
        if nx < grid.width && ny < grid.height && !grid.is_occupied(nx, ny) {
            particles[i].position.x += dx as f32;
            particles[i].position.y += 1.0;
            grid.set(nx, ny, Some(i));
            return;
        }
    }

    for dx in directions {
        let nx = x.wrapping_add_signed(dx);
        if nx < grid.width && !grid.is_occupied(nx, y) && (y + 1 >= grid.height || !grid.is_occupied(nx, y + 1)) {
            particles[i].position.x += dx as f32;
            grid.set(nx, y, Some(i));
            return;
        }
    }

    grid.set(x, y, Some(i));
}

fn update_gas_particle(p: &mut Particle, i: usize, grid: &mut Grid, dt: f32) {
    let x = p.position.x.floor() as usize;
    let y = p.position.y.floor() as usize;

    let empty_above = y > 0 && !grid.is_occupied(x, y - 1);
    let mut force = 9.8 * p.mass;
    if empty_above {
        force = -9.8 * p.mass * 1.5;
    }

    p.apply_force(Vec2::new(0.0, force));
    p.integrate(dt, (grid.width, grid.height));

    let new_x = p.position.x.floor() as usize;
    let new_y = p.position.y.floor() as usize;
    if new_x < grid.width && new_y < grid.height {
        grid.set(new_x, new_y, Some(i));
    }
}
