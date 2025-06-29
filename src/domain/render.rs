/// Darkens a color by a factor (0.0 = black, 1.0 = unchanged)
pub fn darken(color: u32, factor: f32) -> u32 {
    let clamp = |v: f32| v.clamp(0.0, 255.0) as u32;

    let r = clamp(((color >> 16) & 0xFF) as f32 * factor);
    let g = clamp(((color >> 8) & 0xFF) as f32 * factor);
    let b = clamp((color & 0xFF) as f32 * factor);

    (r << 16) | (g << 8) | b
}

/// Lightens a color by interpolating toward white (1.0 = white, 0.0 = unchanged)
pub fn lighten(color: u32, amount: f32) -> u32 {
    let clamp = |v: f32| v.clamp(0.0, 255.0) as u32;

    let r = clamp(((color >> 16) & 0xFF) as f32 + (255.0 - (color >> 16 & 0xFF) as f32) * amount);
    let g = clamp(((color >> 8) & 0xFF) as f32 + (255.0 - (color >> 8 & 0xFF) as f32) * amount);
    let b = clamp((color & 0xFF) as f32 + (255.0 - (color & 0xFF) as f32) * amount);

    (r << 16) | (g << 8) | b
}