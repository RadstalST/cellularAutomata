// src/shader.wgsl

// Define constants and buffer layout
struct Particle {
    position: vec2<f32>,
    velocity: vec2<f32>,
    acceleration: vec2<f32>,
    mass: f32,
    temperature: f32,
    phase: u32,
}

struct SimParams {
    dt: f32,
    width: u32,
    height: u32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<uniform> params: SimParams;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let idx = GlobalInvocationID.x;
    if (idx >= arrayLength(&particles)) {
        return;
    }

    var p = particles[idx];

    let gravity = vec2<f32>(0.0, 9.8);

    if (p.phase == 2u || p.phase == 3u) {
        // Gas or Plasma: Buoyant behavior
        p.acceleration = -gravity;
    } else {
        // Solid or Liquid: Apply gravity
        p.acceleration = gravity;
    }

    // Semi-implicit Euler
    p.velocity += p.acceleration * params.dt;
    p.position += p.velocity * params.dt;

    // Boundary containment
    p.position.x = clamp(p.position.x, 0.0, f32(params.width - 1));
    p.position.y = clamp(p.position.y, 0.0, f32(params.height - 1));

    // Write back
    particles[idx] = p;
}
