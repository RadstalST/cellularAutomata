// src/util/color.rs
use rand::Rng;

/// Applies color noise and optional visual enhancements.
#[allow(clippy::too_many_arguments)]
pub fn noisy_color(
    base: u32,
    max_variation_r: Option<u8>,
    max_variation_g: Option<u8>,
    max_variation_b: Option<u8>,
    darken_factor: Option<f32>,
    lighten_amount: Option<f32>,
    vibrancy_factor: Option<f32>,
    rand_range_darken: Option<(f32, f32)>,
    rand_range_lighten: Option<(f32, f32)>,
    rand_range_vibrancy: Option<(f32, f32)>,
) -> u32 {
    let mut rng = rand::thread_rng();

    let vr = max_variation_r.unwrap_or(12) as i32;
    let vg = max_variation_g.unwrap_or(10) as i32;
    let vb = max_variation_b.unwrap_or(8) as i32;

    let r = ((base >> 16) & 0xFF) as i32 + rng.gen_range(0..=vr);
    let g = ((base >> 8) & 0xFF) as i32 + rng.gen_range(0..=vg);
    let b = (base & 0xFF) as i32 + rng.gen_range(0..=vb);

    let clamp = |v: i32| v.clamp(0, 255) as u32;
    let mut color = (clamp(r) << 16) | (clamp(g) << 8) | clamp(b);

    // 🎨 Choose enhancement values
    let darken_f = darken_factor.unwrap_or_else(|| {
        rand_range_darken
            .map(|(min, max)| rng.gen_range(min..=max))
            .unwrap_or(0.9)
    });

    let lighten_a = lighten_amount.unwrap_or_else(|| {
        rand_range_lighten
            .map(|(min, max)| rng.gen_range(min..=max))
            .unwrap_or(0.1)
    });

    let vibrancy_f = vibrancy_factor.unwrap_or_else(|| {
        rand_range_vibrancy
            .map(|(min, max)| rng.gen_range(min..=max))
            .unwrap_or(1.2)
    });

    color = vibrancy(color, vibrancy_f);
    color = darken(color, darken_f);
    color = lighten(color, lighten_a);

    color
}

pub fn phase_color(base: u32, phase: crate::domain::particle::Phase) -> u32 {
    let noise = match phase {
        crate::domain::particle::Phase::Solid  => (Some(12), Some(10), Some(8)),
        crate::domain::particle::Phase::Liquid => (Some(16), Some(14), Some(12)),
        crate::domain::particle::Phase::Gas    => (Some(22), Some(20), Some(18)),
        crate::domain::particle::Phase::Plasma => (Some(32), Some(30), Some(28)),
    };
    noisy_color(
        base,
        noise.0,
        noise.1,
        noise.2,
        None,
        None,
        None,
        Some((0.8, 1.2)),
        Some((0.05, 0.2)),
        Some((0.5, 1.6))
    )
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
