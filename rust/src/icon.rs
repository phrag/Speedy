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

/// Draw `s` at scale `scale`, its right edge at `right_x`, top at `top_y`.
fn draw_text(
    buf: &mut [u32],
    width: usize,
    height: usize,
    s: &str,
    scale: usize,
    right_x: usize,
    top_y: usize,
) {
    let total_w = text_width_px(s) * scale;
    let mut x = right_x.saturating_sub(total_w);
    for c in s.chars() {
        let Some(rows) = glyph(c) else { continue };
        for (ry, &row) in rows.iter().enumerate() {
            for cx in 0..GLYPH_W {
                // Bit 4 is the leftmost pixel.
                if (row >> (GLYPH_W - 1 - cx)) & 1 == 0 {
                    continue;
                }
                // Paint the scale×scale block for this font pixel.
                for sy in 0..scale {
                    for sx in 0..scale {
                        let px = x + cx * scale + sx;
                        let py = top_y + ry * scale + sy;
                        if px < width && py < height {
                            buf[py * width + px] = WHITE;
                        }
                    }
                }
            }
        }
        x += (GLYPH_W + GLYPH_GAP) * scale;
    }
}

/// Render the icon. `down`/`up` are the formatted rate strings (e.g. `"1.2M"`).
/// Returns `width * height` ARGB_8888 pixels.
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
    let right_x = width; // right-align

    draw_text(&mut buf, width, height, down, scale, right_x, top_y);
    draw_text(
        &mut buf,
        width,
        height,
        up,
        scale,
        right_x,
        top_y + glyph_px + gap,
    );
    buf
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
