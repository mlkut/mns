//! 48-bit avatars, rendered from [crate::Name]'s internal value.
//!
//! The layout mirrors the original 40-bit trick, grown up:
//!   1. 48 bits = 6 source columns × 8 rows.
//!   2. The 6 columns mirror to 11 columns (6·2−1).
//!   3. An 11-row symmetric circle silhouette consumes the 48 bits row-major:
//!      rounded tips top and bottom, fullest in the middle — so the pixels
//!      that would sit in a 6×8 block's corners are redistributed into the
//!      three extra bottom rows.
use crate::name::Name;

const ROWS: usize = 11;
const COLS: usize = 11;
const MARGIN: f32 = 16.0;
const CELL: f32 = 26.0;
const BOARD: f32 = MARGIN * 2.0 + COLS as f32 * CELL; // 318

// Circle silhouette: which source columns appear in each of the 11 rows.
// A centered disk of radius 5 over the 11×11 grid, rounded tips on top and
// bottom and the fullest rows in the middle (sum == 48 bits).
const ROW_COLS: [&[usize]; ROWS] = [
    &[4, 5],
    &[2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[0, 1, 2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[1, 2, 3, 4, 5],
    &[2, 3, 4, 5],
    &[4, 5],
];

impl Name {
    /// Renders the value as an 11×11 mirrored pixel grid SVG (circular).
    pub fn render_avatar_svg(&self) -> String {
        self.render_avatar_svg_with_color("currentColor")
    }

    /// Same as [`Name::render_avatar_svg`] but with a custom fill color.
    pub fn render_avatar_svg_with_color(&self, fill: &str) -> String {
        let value = self.as_u64();

        let mut lit = [[false; COLS]; ROWS];
        let mut bit: usize = 0;
        for (r, cols) in ROW_COLS.iter().enumerate() {
            for &c in *cols {
                if (value >> bit) & 1 == 1 {
                    lit[r][c] = true;
                    lit[r][COLS - 1 - c] = true;
                }
                bit += 1;
            }
        }

        let mut rects = String::new();
        for r in 0..ROWS {
            for c in 0..COLS {
                if lit[r][c] {
                    let x = MARGIN + c as f32 * CELL + 0.5;
                    let y = MARGIN + r as f32 * CELL + 0.5;
                    rects.push_str(&format!(
                        r#"<rect class="mns-avatar-pixel" x="{x}" y="{y}" width="25" height="25"/>"#,
                    ));
                }
            }
        }

        format!(
            r#"<svg class="mns-avatar" width="100%" height="100%" viewBox="0 0 {BOARD} {BOARD}" xmlns="http://www.w3.org/2000/svg"><style>.mns-avatar-pixel{{fill:{fill}}}</style>{rects}</svg>"#,
            fill = fill,
        )
    }
}