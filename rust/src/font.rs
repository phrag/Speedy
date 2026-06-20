//! A minimal 5×7 bitmap font covering only the glyphs the status-bar icon needs:
//! digits, `.`, the unit letters `B K M G`, and the `↓`/`↑` arrows.
//!
//! Each glyph is 7 rows tall and 5 pixels wide. A row is stored in the low 5
//! bits of a `u8`; bit 4 is the leftmost pixel.

pub const GLYPH_W: usize = 5;
pub const GLYPH_H: usize = 7;

/// Returns the 7-row bitmap for `c`, or `None` if the glyph isn't in the font.
pub fn glyph(c: char) -> Option<&'static [u8; 7]> {
    let g: &'static [u8; 7] = match c {
        '0' => &[
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => &[
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => &[
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => &[
            0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110,
        ],
        '4' => &[
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => &[
            0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
        ],
        '6' => &[
            0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => &[
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => &[
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => &[
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100,
        ],
        '.' => &[
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00110, 0b00110,
        ],
        'B' => &[
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'K' => &[
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'M' => &[
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'G' => &[
            0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        '↓' => &[
            0b00100, 0b00100, 0b00100, 0b00100, 0b10101, 0b01110, 0b00100,
        ],
        '↑' => &[
            0b00100, 0b01110, 0b10101, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        _ => return None,
    };
    Some(g)
}

/// True if every character in `s` has a glyph in this font.
pub fn covers(s: &str) -> bool {
    s.chars().all(|c| glyph(c).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_expected_charset() {
        assert!(covers("0123456789.BKMG↓↑"));
    }

    #[test]
    fn missing_glyph_detected() {
        assert!(glyph('Z').is_none());
        assert!(!covers("1.2X"));
    }

    #[test]
    fn glyphs_fit_five_bits() {
        for c in "0123456789.BKMG↓↑".chars() {
            for &row in glyph(c).unwrap() {
                assert!(row < 0b100000, "glyph {c} row {row:#07b} exceeds 5 bits");
            }
        }
    }
}
