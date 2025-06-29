// src/util/color.rs
use rand::Rng;

pub fn noisy_color(base: u32, max_variation: u8) -> u32 {
    let mut rng = rand::thread_rng();

    let r = ((base >> 16) & 0xFF) as i32 + rng.gen_range(0..=max_variation as i32);
    let g = ((base >> 8) & 0xFF) as i32 + rng.gen_range(0..=max_variation as i32);
    let b = (base & 0xFF) as i32 + rng.gen_range(0..=max_variation as i32);

    let clamp = |v: i32| v.clamp(0, 255) as u32;

    let mut color = (clamp(r) << 16) | (clamp(g) << 8) | clamp(b);

    // 🎨 Random darken/lighten and vibrancy
    let factor = rng.gen_range(0.7..=1.3);
    let amount = rng.gen_range(0.0..=0.2);
    let vibrancy_factor = rng.gen_range(0.5..=1.5);

    color = vibrancy(color, vibrancy_factor);
    color = darken(color, factor);
    color = lighten(color, amount);

    color
}

pub fn phase_color(phase: crate::domain::particle::Phase) -> u32 {
    let base = match phase {
        crate::domain::particle::Phase::Solid  => 0x8B4513,
        crate::domain::particle::Phase::Liquid => 0x3399FF,
        crate::domain::particle::Phase::Gas    => 0xCCCCCC,
        crate::domain::particle::Phase::Plasma => 0xFF3399,
    };
    let noise = match phase {
        crate::domain::particle::Phase::Solid  => 8,
        crate::domain::particle::Phase::Liquid => 12,
        crate::domain::particle::Phase::Gas    => 20,
        crate::domain::particle::Phase::Plasma => 30,
    };
    noisy_color(base, noise)
}

pub fn darken(color: u32, factor: f32) -> u32 {
    let clamp = |v: f32| v.clamp(0.0, 255.0) as u32;

    let r = clamp(((color >> 16) & 0xFF) as f32 * factor);
    let g = clamp(((color >> 8) & 0xFF) as f32 * factor);
    let b = clamp((color & 0xFF) as f32 * factor);

    (r << 16) | (g << 8) | b
}

pub fn lighten(color: u32, amount: f32) -> u32 {
    let clamp = |v: f32| v.clamp(0.0, 255.0) as u32;

    let r = clamp(((color >> 16) & 0xFF) as f32 + (255.0 - (color >> 16 & 0xFF) as f32) * amount);
    let g = clamp(((color >> 8) & 0xFF) as f32 + (255.0 - (color >> 8 & 0xFF) as f32) * amount);
    let b = clamp((color & 0xFF) as f32 + (255.0 - (color & 0xFF) as f32) * amount);

    (r << 16) | (g << 8) | b
}

pub fn vibrancy(color: u32, factor: f32) -> u32 {
    let r = ((color >> 16) & 0xFF) as f32;
    let g = ((color >> 8) & 0xFF) as f32;
    let b = (color & 0xFF) as f32;
    let gray = (r + g + b) / 3.0;

    let enhance = |c: f32| (gray + (c - gray) * factor).clamp(0.0, 255.0) as u32;

    let r = enhance(r);
    let g = enhance(g);
    let b = enhance(b);

    (r << 16) | (g << 8) | b
}
