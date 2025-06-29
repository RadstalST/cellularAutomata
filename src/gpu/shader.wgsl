struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
    mass: f32,
    radius: f32,
    temperature: f32,
    phase: u32,
    color: u32,
};

struct SimParams {
    dt: f32,
    width: u32,
    height: u32,
};

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: SimParams;
@group(0) @binding(2) var<storage, read_write> occupancy: array<u32>;

fn get_index(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

fn is_occupied(x: u32, y: u32) -> bool {
    let idx = get_index(x, y, params.width);
    return occupancy[idx] != 0u;
}

fn set_occupied(x: u32, y: u32, particle_id: u32) {
    let idx = get_index(x, y, params.width);
    occupancy[idx] = particle_id + 1u;
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if (i >= arrayLength(&particles)) {
        return;
    }

    var p = particles[i];
    var x = u32(floor(p.position.x));
    var y = u32(floor(p.position.y));

    if (x >= params.width || y >= params.height) {
        return;
    }

    var moved = false;

    // Solid (e.g., SAND)
    if (p.phase == 0u) {
        if (y + 1u < params.height && !is_occupied(x, y + 1u)) {
            y = y + 1u;
            p.position.y += 1.0;
            moved = true;
        }

        // Diagonal left
        if (!moved) {
            let dx = -1;
            let nx = i32(x) + dx;
            let ny = i32(y) + 1;
            if (nx >= 0 && u32(nx) < params.width && u32(ny) < params.height &&
                !is_occupied(u32(nx), u32(ny))) {
                p.position.x += f32(dx);
                p.position.y += 1.0;
                x = u32(nx);
                y = u32(ny);
                moved = true;
            }
        }

        // Diagonal right
        if (!moved) {
            let dx = 1;
            let nx = i32(x) + dx;
            let ny = i32(y) + 1;
            if (nx >= 0 && u32(nx) < params.width && u32(ny) < params.height &&
                !is_occupied(u32(nx), u32(ny))) {
                p.position.x += f32(dx);
                p.position.y += 1.0;
                x = u32(nx);
                y = u32(ny);
                moved = true;
            }
        }
    }

    // Liquid (e.g., WATER)
    if (p.phase == 1u) {
        if (y + 1u < params.height && !is_occupied(x, y + 1u)) {
            y = y + 1u;
            p.position.y += 1.0;
            moved = true;
        }

        // Diagonal left
        if (!moved) {
            let dx = -1;
            let nx = i32(x) + dx;
            let ny = i32(y) + 1;
            if (nx >= 0 && u32(nx) < params.width && u32(ny) < params.height &&
                !is_occupied(u32(nx), u32(ny))) {
                p.position.x += f32(dx);
                p.position.y += 1.0;
                x = u32(nx);
                y = u32(ny);
                moved = true;
            }
        }

        // Diagonal right
        if (!moved) {
            let dx = 1;
            let nx = i32(x) + dx;
            let ny = i32(y) + 1;
            if (nx >= 0 && u32(nx) < params.width && u32(ny) < params.height &&
                !is_occupied(u32(nx), u32(ny))) {
                p.position.x += f32(dx);
                p.position.y += 1.0;
                x = u32(nx);
                y = u32(ny);
                moved = true;
            }
        }

        // Horizontal left
        if (!moved) {
            let dx = -1;
            let nx = i32(x) + dx;
            if (nx >= 0 && u32(nx) < params.width &&
                !is_occupied(u32(nx), y)) {
                p.position.x += f32(dx);
                x = u32(nx);
                moved = true;
            }
        }

        // Horizontal right
        if (!moved) {
            let dx = 1;
            let nx = i32(x) + dx;
            if (nx >= 0 && u32(nx) < params.width &&
                !is_occupied(u32(nx), y)) {
                p.position.x += f32(dx);
                x = u32(nx);
                moved = true;
            }
        }
    }

    // Gas / Plasma
    if (p.phase == 2u || p.phase == 3u) {
        let gravity = vec2<f32>(0.0, -9.8);
        p.acceleration = gravity;
        p.velocity += p.acceleration * params.dt;
        p.position += p.velocity * params.dt;

        p.position.x = clamp(p.position.x, 0.0, f32(params.width - 1u));
        p.position.y = clamp(p.position.y, 0.0, f32(params.height - 1u));

        x = u32(floor(p.position.x));
        y = u32(floor(p.position.y));
    }

    if (x < params.width && y < params.height) {
        set_occupied(x, y, i);
    }

    particles[i] = p;
}
