use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::Color,
};
use tachyonfx::{
    fx,
    Duration,
    Effect,
    EffectTimer,
    Interpolation,
    RefCount,
};

#[derive(Clone)]
struct RainState {
    width: u16,
    height: u16,
    cols: Vec<Column>,
    frame: u64,
}

#[derive(Clone, Copy)]
struct Column {
    y: f32,
    speed_cps: f32, // cells per second
    tail: u16,
    seed: u64,
}

impl RainState {
    fn new(area: Rect) -> Self {
        let width = area.width.max(1);
        let height = area.height.max(1);
        let cols = (0..width)
            .map(|i| new_col(i as u64, height))
            .collect();
        Self {
            width,
            height,
            cols,
            frame: 0,
        }
    }
}

/// Matrix-like digital rain that morphs into the target buffer content.
/// Renders rain for ~1.6s, then progressively reveals the target by replacing
/// cells based on the animation alpha.
pub fn matrix_rain_morph_with_duration(target: RefCount<Buffer>, duration_ms: u64) -> Effect {
    let timer = EffectTimer::from_ms(duration_ms as u32, Interpolation::QuadOut);

    fx::effect_fn_buf(RainState::new(Rect::new(0, 0, 1, 1)), timer, move |state, ctx, buf| {
        // Skip processing if buffer area is too small
        if buf.area.width < 1 || buf.area.height < 1 {
            return;
        }
        // Resize/adapt state
        if state.width != buf.area.width || state.height != buf.area.height || state.cols.len() != buf.area.width as usize {
            *state = RainState::new(buf.area);
        }

        let dt = ctx.last_tick.as_secs_f32().max(0.0);
        state.frame = state.frame.wrapping_add(1);

        // Background
        let bg = Color::Rgb(2, 6, 8);
        for y in 0..state.height {
            for x in 0..state.width {
                if let Some(c) = buf.cell_mut(Position::new(buf.area.x + x, buf.area.y + y)) {
                    c.set_char(' ');
                    c.set_bg(bg);
                }
            }
        }

        // Advance columns
        for (x, col) in state.cols.iter_mut().enumerate() {
            col.y += col.speed_cps * dt;
            let max_y = state.height as f32 + col.tail as f32 + 2.0;
            if col.y > max_y {
                *col = new_col(col.seed.wrapping_add(0x9E37), state.height);
                // start above the screen by up to half-height for staggered entry
                let off = (mix_u32(col.seed ^ 0xA5A5_4242) % (state.height as u32 / 2 + 1)) as i32;
                col.y = -(off as f32);
            }

            let head_y = col.y as i32;
            let tail = col.tail as i32;

            // Draw head and trail
            for t in 0..=tail {
                let yy = head_y - t;
                if yy < 0 || yy as u16 >= state.height {
                    continue;
                }
                let pos = Position::new(buf.area.x + x as u16, buf.area.y + yy as u16);
                if let Some(cell) = buf.cell_mut(pos) {
                    let intensity = 1.0 - (t as f32 / (tail as f32 + 0.001)).min(1.0);
                    let ch = matrix_glyph(x as u16, yy as u16, state.frame, col.seed, t as u16);
                    cell.set_char(ch);

                    let (r, g, b) = if t == 0 {
                        // Bright white-green head
                        (180, 255, 200)
                    } else {
                        // Trail gradient: vivid → dim green
                        let g = (80.0 + 175.0 * intensity) as u8;
                        let r = (10.0 * intensity) as u8;
                        let b = (40.0 + 40.0 * intensity) as u8;
                        (r, g, b)
                    };
                    cell.set_fg(Color::Rgb(r, g, b));
                }
            }
        }

        // Morph/dissolve into target based on alpha
        let alpha = ctx.alpha().clamp(0.0, 1.0);
        if alpha > 0.0 {
            let tbuf = target.borrow();
            let copy_w = state.width.min(tbuf.area.width).min(buf.area.width);
            let copy_h = state.height.min(tbuf.area.height).min(buf.area.height);

            // Probability mask per cell → reveal target as alpha grows
            for y in 0..copy_h {
                for x in 0..copy_w {
                    let seed = ((x as u64) << 32) ^ (y as u64) ^ 0xD1B54A32D192ED03u64;
                    let r = (mix_u32(seed) as f32) / (u32::MAX as f32);
                    if r < alpha.powf(1.4) {
                        if let Some(src) = tbuf.cell(Position::new(tbuf.area.x + x, tbuf.area.y + y)).cloned() {
                            if let Some(dst) = buf.cell_mut(Position::new(buf.area.x + x, buf.area.y + y)) {
                                *dst = src;
                            }
                        }
                    } else {
                        // Slightly dim rain as we approach full reveal
                        if let Some(dst) = buf.cell_mut(Position::new(buf.area.x + x, buf.area.y + y)) {
                            if let Color::Rgb(r, g, b) = dst.fg { 
                                let fade = (1.0 - alpha).clamp(0.0, 1.0);
                                let (r2, g2, b2) = (
                                    ((r as f32) * fade) as u8,
                                    ((g as f32) * fade) as u8,
                                    ((b as f32) * fade) as u8,
                                );
                                dst.set_fg(Color::Rgb(r2, g2, b2));
                            }
                        }
                    }
                }
            }
        }
    })
}

pub fn matrix_rain_morph(target: RefCount<Buffer>) -> Effect {
    matrix_rain_morph_with_duration(target, 1800)
}

fn new_col(seed: u64, height: u16) -> Column {
    // speed ~ 8..24 cps
    let speed = 8.0 + (mix_u32(seed ^ 0xB5297A4D) % 17) as f32;
    // tail ~ 8..22
    let tail = 8 + (mix_u32(seed ^ 0x68E31DA4) % 15) as u16;
    let mut y = -((mix_u32(seed ^ 0x1B56C4E9) % (height as u32 / 2 + 1)) as i32);
    if y > 0 {
        y = -y;
    }
    Column {
        y: y as f32,
        speed_cps: speed,
        tail,
        seed,
    }
}

fn mix_u32(seed: u64) -> u32 {
    // xorshift-style mixer (deterministic, no deps)
    let mut x = seed ^ 0x9E3779B97F4A7C15u64;
    x ^= x << 7;
    x ^= x >> 9;
    x ^= x << 8;
    (x as u32).wrapping_mul(2654435761)
}

fn matrix_glyph(x: u16, y: u16, frame: u64, seed: u64, trail_idx: u16) -> char {
    // Change a bit over time for shimmer
    let s = (seed
        ^ ((x as u64) << 32)
        ^ ((y as u64) << 16)
        ^ (frame.wrapping_mul(0x9E37)) as u64
        ^ (trail_idx as u64) * 0xA3)
        as u64;
    let r = mix_u32(s) as usize;
    KATAKANA[r % KATAKANA.len()]
}

// A compact set of half/full-width katakana and symbols reminiscent of the film
static KATAKANA: &[char] = &[
    'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ',
    'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ', 'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ',
    'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ',
    'ﾏ', 'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    'ｧ', 'ｨ', 'ｩ', 'ｪ', 'ｫ', 'ｯ', 'ｬ', 'ｭ', 'ｮ',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    '・', '･', '＝', '≡', '∵', '∴', '◇', '◆', '○', '●',
];
