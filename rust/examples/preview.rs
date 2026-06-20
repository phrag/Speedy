//! Renders a strip of sample status-bar icons to a BMP so the glyph rendering
//! can be eyeballed off-device. Run with: `cargo run --example preview`.

use speedy_core::icon;
use std::fs::File;
use std::io::{BufWriter, Write};

const TILE: usize = 96;

fn main() {
    // Download-only values spanning the formatting ranges (what the status bar
    // shows now — one big number via render_single).
    let samples = ["2K", "85K", "1M", "120M"];
    let w = TILE * samples.len();
    let h = TILE;

    // Dark background so the white (transparent-backed) glyphs are visible.
    let mut rgb = vec![0u8; w * h * 3];
    for px in rgb.chunks_mut(3) {
        px.copy_from_slice(&[20, 22, 26]);
    }

    for (col, text) in samples.iter().enumerate() {
        let icon = icon::render_single(TILE, TILE, text);
        for y in 0..TILE {
            for x in 0..TILE {
                if (icon[y * TILE + x] >> 24) & 0xFF > 0 {
                    let idx = (y * w + col * TILE + x) * 3;
                    rgb[idx..idx + 3].copy_from_slice(&[255, 255, 255]);
                }
            }
        }
    }

    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "preview.bmp".into());
    write_bmp(&path, w, h, &rgb).expect("write bmp");
    println!("wrote {path} ({w}x{h})");
}

/// Write a 24-bit, bottom-up BMP from row-major RGB bytes.
fn write_bmp(path: &str, w: usize, h: usize, rgb: &[u8]) -> std::io::Result<()> {
    let row = (w * 3 + 3) & !3; // pad each row to a multiple of 4 bytes
    let pixels = row * h;
    let size = 54 + pixels;
    let mut f = BufWriter::new(File::create(path)?);

    // File header (14 bytes).
    f.write_all(b"BM")?;
    f.write_all(&(size as u32).to_le_bytes())?;
    f.write_all(&0u32.to_le_bytes())?;
    f.write_all(&54u32.to_le_bytes())?;
    // Info header (40 bytes).
    f.write_all(&40u32.to_le_bytes())?;
    f.write_all(&(w as i32).to_le_bytes())?;
    f.write_all(&(h as i32).to_le_bytes())?;
    f.write_all(&1u16.to_le_bytes())?;
    f.write_all(&24u16.to_le_bytes())?;
    f.write_all(&[0u8; 24])?; // compression + sizes + resolution + palette

    let pad = [0u8; 3];
    for y in (0..h).rev() {
        for x in 0..w {
            let i = (y * w + x) * 3;
            f.write_all(&[rgb[i + 2], rgb[i + 1], rgb[i]])?; // BGR
        }
        f.write_all(&pad[..row - w * 3])?;
    }
    Ok(())
}
