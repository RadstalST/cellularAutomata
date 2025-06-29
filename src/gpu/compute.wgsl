// compute.wgsl

struct Vec2 {
    x: f32,
    y: f32,
};

struct Particle {
    position: Vec2,
    velocity: Vec2,
    acceleration: Vec2,
    mass: f32,
    radius: f32,
    temperature: f32,
    phase: u32,
    color: u32,
};

@group(0) @binding(0)
var<storage, read_write> particles: array<Particle>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let index = id.x;

    if index >= arrayLength(&particles) {
        return;
    }

    var p = &particles[index];

    // Apply gravity to all phases for now (simplified)
    let gravity = vec2<f32>(0.0, 9.8);
    p.acceleration = gravity;

    // Integrate velocity and position
    let dt = 0.016;
    p.velocity = p.velocity + p.acceleration * dt;
    p.position = p.position + p.velocity * dt;

    // Reset acceleration for next frame
    p.acceleration = vec2<f32>(0.0, 0.0);
}
