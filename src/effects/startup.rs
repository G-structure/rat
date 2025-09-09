use ratatui::{
    buffer::Buffer,
    layout::{Position, Rect},
    style::{Color, Style},
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
    // y position per column (i32 to allow -1 starting)
    drops: Vec<i32>,
    // ms accumulator for step updates
    acc_ms: u32,
}

impl RainState {
    fn new(area: Rect) -> Self {
        let width = area.width.max(1);
        let height = area.height.max(1);
        // stagger starts above the top edge for nicer entrance
        let drops = (0..width)
            .map(|x| -((x as i32) % 20))
            .collect::<Vec<_>>();
        Self {
            width,
            height,
            drops,
            acc_ms: 0,
        }
    }
}

/// Matrix-like digital rain that morphs into the target buffer content.
/// Renders rain for ~1.6s, then progressively reveals the target by replacing
/// cells based on the animation alpha.
pub fn matrix_rain_morph_with_duration(target: RefCount<Buffer>, duration_ms: u64) -> Effect {
    let timer = EffectTimer::from_ms(duration_ms as u32, Interpolation::QuadOut);

    fx::effect_fn_buf(RainState::new(Rect::new(0, 0, 1, 1)), timer, move |state, ctx, buf| {
        // Initialize dimensions if they changed
        if state.width != buf.area.width || state.height != buf.area.height {
            *state = RainState::new(buf.area);
        }

        // Step speed ~ every 30ms
        let step_ms = 30u32;
        let last_ms: u32 = ctx.last_tick.as_millis() as _;
        state.acc_ms = state.acc_ms.saturating_add(last_ms);
        let steps = state.acc_ms / step_ms;
        state.acc_ms %= step_ms;

        // Advance drops
        for _ in 0..steps {
            for d in &mut state.drops {
                *d += 1;
                if *d >= state.height as i32 + 8 {
                    *d = -8; // reset slightly above the top for trailing effect
                }
            }
        }

        // Draw background
        let bg = Color::from_u32(0x0b0f14);
        for y in 0..state.height {
            for x in 0..state.width {
                let c = buf.cell_mut(Position::new(buf.area.x + x, buf.area.y + y)).unwrap();
                c.set_char(' ');
                c.set_bg(bg);
                // keep fg as is; will set when drawing rain/target
            }
        }

        // Draw rain heads and trails
        let green = Color::from_u32(0x00ff84);
        let dim = Color::from_u32(0x007a48);
        for x in 0..state.width {
            let head_y = state.drops[x as usize];
            // head
            if (0..state.height as i32).contains(&head_y) {
                let cell = buf
                    .cell_mut(Position::new(buf.area.x + x, buf.area.y + head_y as u16))
                    .unwrap();
                cell.set_char(random_glyph((x as i32 * 31 + head_y) as u64));
                cell.set_fg(green);
            }
            // a few trailing cells
            for t in 1..4 {
                let y = head_y - t;
                if (0..state.height as i32).contains(&y) {
                    let cell = buf
                        .cell_mut(Position::new(buf.area.x + x, buf.area.y + y as u16))
                        .unwrap();
                    cell.set_char(random_glyph((x as i32 * 131 + y) as u64));
                    cell.set_fg(dim);
                }
            }
        }

        // Morph: progressively reveal target content based on alpha
        let alpha = ctx.alpha().clamp(0.0, 1.0);
        if alpha > 0.0 {
            let t = target.borrow();
            // Clamp copy region to smallest common area to avoid out-of-bounds
            let copy_w = state.width.min(t.area.width).min(buf.area.width);
            let copy_h = state.height.min(t.area.height).min(buf.area.height);

            // proportion of rows revealed this frame
            let reveal_rows = ((alpha * copy_h as f32) as u16).min(copy_h);
            for y in 0..reveal_rows {
                for x in 0..copy_w {
                    if let Some(src) = t.cell(Position::new(t.area.x + x, t.area.y + y)).cloned() {
                        if let Some(dst) = buf.cell_mut(Position::new(buf.area.x + x, buf.area.y + y)) {
                            *dst = src;
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

fn random_glyph(seed: u64) -> char {
    // simple LCG-ish mapping (deterministic, no rng deps)
    let v = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    let k = (v >> 32) as u8;
    // pick from a small set of glyphs
    match k % 7 {
        0 => '0',
        1 => '1',
        2 => '∷',
        3 => '≣',
        4 => '╳',
        5 => '░',
        _ => '▒',
    }
}
