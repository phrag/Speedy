//! Rasterize the two throughput rows into an ARGB_8888 status-bar icon.
//!
//! Layout: download on the top half, upload on the bottom half — direction is
//! conveyed by position (no arrow glyphs), which frees horizontal space so the
//! digits render larger in the tiny status-bar slot. The `↓`/`↑` arrows live in
//! the expanded notification label instead. Text is white on a transparent
//! background; Android tints/scales the bitmap for the status bar. The returned
//! buffer is `width * height` `0xAARRGGBB` pixels, row-major, ready for
//! `Bitmap.setPixels`.

use crate::font::{glyph, GLYPH_H, GLYPH_W};

const WHITE: u32 = 0xFFFF_FFFF;
/// Inter-glyph gap, in font pixels (scaled along with the glyphs).
const GLYPH_GAP: usize = 1;

/// Width of a string in unscaled font pixels (glyphs + inter-glyph gaps).
fn text_width_px(s: &str) -> usize {
    let n = s.chars().count();
    if n == 0 {
        return 0;
    }
    n * GLYPH_W + (n - 1) * GLYPH_GAP
}

/// Largest integer scale at which both rows fit inside `width × height`.
fn fit_scale(width: usize, height: usize, top: &str, bottom: &str) -> usize {
    let widest = text_width_px(top).max(text_width_px(bottom)).max(1);
    let by_w = width / widest;
    // Two stacked rows, each GLYPH_H tall, plus a one-pixel gap between them.
    let by_h = height / (2 * GLYPH_H + 1);
    by_w.min(by_h).max(1)
}

/// Draw `s` with independent per-axis scale (`sx`, `sy`), placing the top-left
/// of the first glyph at `origin` = (left_x, top_y).
fn draw_text(
    buf: &mut [u32],
    width: usize,
    height: usize,
    s: &str,
    sx: usize,
    sy: usize,
    origin: (usize, usize),
) {
    let (left_x, top_y) = origin;
    let mut x = left_x;
    for c in s.chars() {
        let Some(rows) = glyph(c) else { continue };
        for (ry, &row) in rows.iter().enumerate() {
            for cx in 0..GLYPH_W {
                // Bit 4 is the leftmost pixel.
                if (row >> (GLYPH_W - 1 - cx)) & 1 == 0 {
                    continue;
                }
                // Paint the sx×sy block for this font pixel.
                for dy in 0..sy {
                    for dx in 0..sx {
                        let px = x + cx * sx + dx;
                        let py = top_y + ry * sy + dy;
                        if px < width && py < height {
                            buf[py * width + px] = WHITE;
                        }
                    }
                }
            }
        }
        x += (GLYPH_W + GLYPH_GAP) * sx;
    }
}

/// Cap on how much taller-than-wide a glyph may be stretched when filling the
/// icon, so a single big number stays legible rather than looking smeared.
const MAX_STRETCH: usize = 3;

/// Render a single value (e.g. the download rate) as large as possible, filling
/// the icon's height and centering it. This is what the status-bar icon uses:
/// one big number is far more legible in the tiny square than two stacked rows.
pub fn render_single(width: usize, height: usize, text: &str) -> Vec<u32> {
    let mut buf = vec![0u32; width.saturating_mul(height)];
    if width == 0 || height == 0 || text.is_empty() {
        return buf;
    }

    let units_w = text_width_px(text).max(1);
    let sx = (width / units_w).max(1);
    // Fill the vertical space, but don't stretch past MAX_STRETCH × the width scale.
    let sy = (height / GLYPH_H).max(1).min(sx * MAX_STRETCH).max(sx);

    let total_w = (text_width_px(text) * sx).min(width);
    let glyph_h = (GLYPH_H * sy).min(height);
    let left_x = (width - total_w) / 2;
    let top_y = (height - glyph_h) / 2;

    draw_text(&mut buf, width, height, text, sx, sy, (left_x, top_y));
    embolden(&mut buf, width, height);
    buf
}

/// Render two stacked rows (download over upload), right-aligned. Kept for the
/// two-value layout; the app currently uses [`render_single`] instead.
pub fn render(width: usize, height: usize, down: &str, up: &str) -> Vec<u32> {
    let mut buf = vec![0u32; width.saturating_mul(height)];
    if width == 0 || height == 0 {
        return buf;
    }

    // No arrow prefixes: top row = download, bottom row = upload (by position).
    let scale = fit_scale(width, height, down, up);

    let glyph_px = GLYPH_H * scale;
    let gap = scale; // one font-pixel gap between the rows
    let block_h = 2 * glyph_px + gap;
    let top_y = (height - block_h) / 2;

    // Right-align each row.
    let down_x = width.saturating_sub(text_width_px(down) * scale);
    let up_x = width.saturating_sub(text_width_px(up) * scale);
    draw_text(&mut buf, width, height, down, scale, scale, (down_x, top_y));
    draw_text(
        &mut buf,
        width,
        height,
        up,
        scale,
        scale,
        (up_x, top_y + glyph_px + gap),
    );

    // Thicken strokes so the glyphs survive Android's heavy downscale + the
    // monochrome alpha-mask tinting it applies to status-bar icons.
    embolden(&mut buf, width, height);
    buf
}

/// One-pixel dilation: any transparent pixel orthogonally adjacent to a lit one
/// becomes lit. Doubles effective stroke weight without merging the digits.
fn embolden(buf: &mut [u32], width: usize, height: usize) {
    let src = buf.to_vec();
    for y in 0..height {
        for x in 0..width {
            if src[y * width + x] == WHITE {
                continue;
            }
            let lit_neighbor = (x > 0 && src[y * width + x - 1] == WHITE)
                || (x + 1 < width && src[y * width + x + 1] == WHITE)
                || (y > 0 && src[(y - 1) * width + x] == WHITE)
                || (y + 1 < height && src[(y + 1) * width + x] == WHITE);
            if lit_neighbor {
                buf[y * width + x] = WHITE;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffer_has_exact_dimensions() {
        let buf = render(48, 48, "1.2M", "340K");
        assert_eq!(buf.len(), 48 * 48);
    }

    #[test]
    fn draws_some_opaque_white_pixels() {
        let buf = render(64, 64, "1.2M", "340K");
        let lit = buf.iter().filter(|&&p| p == WHITE).count();
        assert!(lit > 0, "expected rendered glyph pixels");
        // Sanity: it shouldn't fill the whole icon.
        assert!(lit < buf.len());
    }

    #[test]
    fn background_is_transparent() {
        let buf = render(48, 48, "0", "0");
        assert!(buf.contains(&0), "expected transparent background pixels");
        // Every pixel is either transparent or opaque white — nothing else.
        assert!(buf.iter().all(|&p| p == 0 || p == WHITE));
    }

    #[test]
    fn zero_size_is_safe() {
        assert!(render(0, 0, "1M", "1M").is_empty());
    }

    #[test]
    fn single_value_fills_more_than_stacked() {
        // The single big number should light more pixels than the same value in
        // the two-row layout (it uses the full height).
        let single = render_single(72, 72, "2K");
        let stacked = render(72, 72, "2K", "1K");
        let lit = |b: &[u32]| b.iter().filter(|&&p| p == WHITE).count();
        assert!(lit(&single) > lit(&stacked), "single should be larger");
        assert_eq!(single.len(), 72 * 72);
    }

    #[test]
    fn single_value_stays_in_bounds() {
        for t in ["0", "2K", "120M", "999G"] {
            let buf = render_single(48, 48, t);
            assert_eq!(buf.len(), 48 * 48);
            assert!(buf.contains(&WHITE));
        }
        assert!(render_single(0, 0, "2K").is_empty());
        assert!(render_single(48, 48, "").iter().all(|&p| p == 0));
    }

    #[test]
    fn shorter_text_renders_larger() {
        // Dropping the arrow prefix shortens each row, which should let the
        // glyphs scale up in the same icon box.
        assert!(fit_scale(72, 72, "1M", "1M") > fit_scale(72, 72, "↓999G", "↑999G"));
    }

    #[test]
    fn long_strings_stay_in_bounds() {
        // Narrow icon with wide text must not panic or overflow the buffer.
        let buf = render(16, 16, "999G", "999G");
        assert_eq!(buf.len(), 16 * 16);
    }
}
