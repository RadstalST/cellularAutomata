// src/gpu/shader.wgsl

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
// occupancy grid of atomic<u32>, 0 = free, >0 = particle_id+1
@group(0) @binding(2) var<storage, read_write> occupancy: array<atomic<u32>>;

// a very cheap pseudo-random bit from the particle index
fn rand_bit(i: u32) -> bool {
    // linear congruential step, then pick bit 16
    return (((i * 1103515245u + 12345u) >> 16u) & 1u) == 1u;
}

fn get_index(x: u32, y: u32, width: u32) -> u32 {
    return y * width + x;
}

// try to atomically claim (x+dx, y+dy). On success advance *pos, return true.
fn try_move(
    x: u32, y: u32,
    dx: i32, dy: i32,
    id: u32,
    pos: ptr<function, vec2<f32>>
) -> bool {
    let nx_i = i32(x) + dx;
    let ny_i = i32(y) + dy;
    if (nx_i < 0 || ny_i < 0) {
        return false;
    }
    let nx = u32(nx_i);
    let ny = u32(ny_i);
    // Fallback: Force movement if grid cell is stuck
    if (nx < params.width && ny < params.height) {
        (*pos).x = f32(nx);
        (*pos).y = f32(ny);
        return true;
    }
    if (nx >= params.width || ny >= params.height) {
        return false;
    }
    let idx = get_index(nx, ny, params.width);
    // if grid was zero, atomicMin returns old=0 => we claim it
    if (atomicMin(&occupancy[idx], id + 1u) == 0u) {
        (*pos).x += f32(dx);
        (*pos).y += f32(dy);
        return true;
    }
    return false;
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

    // Debugging: Log particle position and phase
    // Uncomment the following line for debugging
    // Debugging: Log particle position and phase (removed unsupported print statement)

    // clear occupancy for this old cell if needed?
    // (we assume CPU cleared entire occupancy buffer before dispatch)

    let bias = rand_bit(i);
    var moved = false;

    // --- SOLID (SAND) behavior ---
    if (p.phase == 0u) {
        // 1) straight down
        moved = try_move(x, y, 0, 1, i, &p.position);
        // 2) one diagonal, biased
        if (!moved) {
            moved = try_move(x, y, 1, 1, i, &p.position) || try_move(x, y, -1, 1, i, &p.position);
        }
    }

    // --- LIQUID (WATER) behavior ---
    if (p.phase == 1u) {
        // down
        moved = try_move(x, y, 0, 1, i, &p.position);
        // diag biased
        if (!moved) {
            moved = try_move(x, y, 1, 1, i, &p.position) || try_move(x, y, -1, 1, i, &p.position);
        }
        // left/right
        if (!moved) {
            if (bias) {
                moved = try_move(x, y, 1, 0, i, &p.position);
            } else {
                moved = try_move(x, y, -1, 0, i, &p.position);
            }
        }
        if (!moved) {
            if (bias) {
                moved = try_move(x, y, -1, 0, i, &p.position);
            } else {
                moved = try_move(x, y, 1, 0, i, &p.position);
            }
        }
    }

    // --- GAS / PLASMA behavior ---
    if (p.phase == 2u || p.phase == 3u) {
        let gravity = vec2<f32>(0.0, -9.8);
        p.acceleration = gravity;
        p.velocity += p.acceleration * params.dt;
        p.position += p.velocity * params.dt;
        p.velocity *= 0.99; // Reduce damping for smoother movement
        p.position.x = clamp(p.position.x, 0.0, f32(params.width - 1u));
        p.position.y = clamp(p.position.y, 0.0, f32(params.height - 1u));
    }

    // Write our new position into the occupancy grid
    let new_x = u32(floor(p.position.x));
    let new_y = u32(floor(p.position.y));
    if (new_x < params.width && new_y < params.height) {
        let idx = get_index(new_x, new_y, params.width);
        if (p.phase == 0u || p.phase == 1u) {
            atomicStore(&occupancy[idx], i + 1u);
        }
    }

    // Write debug information to the debug buffer
    let debug_idx = i * 3u;
    atomicStore(&occupancy[debug_idx], new_x);
    atomicStore(&occupancy[debug_idx + 1u], new_y);
    atomicStore(&occupancy[debug_idx + 2u], p.phase);

    particles[i] = p;
}
