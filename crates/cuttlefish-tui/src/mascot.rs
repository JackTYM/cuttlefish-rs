//! Cuttlefish mascot widget with 16x16 pixel art.
//!
//! Renders a cute cuttlefish mascot that scales to fill available space.
//! Each "pixel" renders as 2 characters wide × 1 line tall for a square appearance.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::Widget,
};

/// 16x16 pixel art grid for the cuttlefish mascot (mouth closed).
///
/// Each character maps to a color in the palette.
const GRID_CLOSED: [&str; 16] = [
    "...1222222221...", // 0 - top of head
    "..123444444321..", // 1
    "..12E444444E21..", // 2 - eyes
    "..12344HH44321..", // 3 - highlight
    ".b12ss2222ss21b.", // 4 - fins + stripe
    ".ab2344444432ba.", // 5 - fins
    "..123444444321..", // 6
    "...1233333321...", // 7 - taper
    "................", // 8 - gap
    "...t.t.tt.t.t...", // 9 - tentacles
    "...t.t.tt.t.t...", // 10
    "...u.u.tt.u.u...", // 11
    ".....u.uu.u.....", // 12
    ".....v.uu.v.....", // 13
    ".......vv.......", // 14
    ".......vv.......", // 15
];

/// 16x16 pixel art grid for the cuttlefish mascot (mouth open).
const GRID_OPEN: [&str; 16] = [
    "...1222222221...", // 0 - top of head
    "..123444444321..", // 1
    "..12E444444E21..", // 2 - eyes
    "..12344HH44321..", // 3 - highlight (smile)
    ".b12ss2MM2ss21b.", // 4 - mouth OPEN in stripe area
    ".ab2344444432ba.", // 5 - fins
    "..123444444321..", // 6
    "...1233333321...", // 7 - taper
    "................", // 8 - gap
    "...t.t.tt.t.t...", // 9 - tentacles
    "...t.t.tt.t.t...", // 10
    "...u.u.tt.u.u...", // 11
    ".....u.uu.u.....", // 12
    ".....v.uu.v.....", // 13
    ".......vv.......", // 14
    ".......vv.......", // 15
];

/// Alias for tests
#[cfg(test)]
const GRID: [&str; 16] = GRID_CLOSED;

/// Color palette mapping characters to RGB colors.
const PALETTE: &[(char, (u8, u8, u8))] = &[
    ('.', (10, 22, 40)),    // background (dark blue)
    ('1', (184, 106, 53)),  // outline
    ('2', (200, 121, 65)),  // body outer
    ('3', (212, 136, 78)),  // body mid
    ('4', (234, 170, 114)), // body inner
    ('H', (245, 195, 140)), // highlight
    ('E', (26, 26, 46)),    // eyes (dark)
    ('M', (60, 30, 30)),    // mouth (dark red/maroon)
    ('s', (160, 80, 20)),   // stripe
    ('a', (100, 58, 22)),   // fin dark
    ('b', (140, 82, 30)),   // fin light
    ('t', (180, 100, 40)),  // tentacle 1
    ('u', (140, 78, 28)),   // tentacle 2
    ('v', (95, 50, 15)),    // tentacle 3
];

/// Background character (transparent).
const BG_CHAR: char = '.';

/// Look up a color from the palette by character.
///
/// Returns None for background/transparent pixels.
fn color_for_char(c: char) -> Option<Color> {
    if c == BG_CHAR {
        return None;
    }
    for &(palette_char, (r, g, b)) in PALETTE {
        if palette_char == c {
            return Some(Color::Rgb(r, g, b));
        }
    }
    None
}

/// Rendering mode for the mascot.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum MascotMode {
    /// Standard mode: 2 chars wide × 1 char tall per pixel (32x16 chars for 16x16 grid).
    #[default]
    Standard,
    /// Compact mode using half-block characters: 1 char = 2 vertical pixels (16x8 chars).
    HalfBlock,
}

/// A widget that renders the cuttlefish mascot.
///
/// The mascot scales to fill the available area while maintaining aspect ratio.
#[derive(Debug, Clone)]
pub struct MascotWidget {
    mode: MascotMode,
    mouth_open: bool,
}

impl Default for MascotWidget {
    fn default() -> Self {
        Self {
            mode: MascotMode::Standard,
            mouth_open: false,
        }
    }
}

impl MascotWidget {
    /// Create a new mascot widget (standard mode).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a compact mascot widget using half-block characters.
    /// This renders 16x16 pixels in just 16x8 character cells.
    #[must_use]
    pub fn compact() -> Self {
        Self {
            mode: MascotMode::HalfBlock,
            mouth_open: false,
        }
    }

    /// Set mouth state (open/closed) for animation.
    #[must_use]
    pub fn with_mouth_open(mut self, open: bool) -> Self {
        self.mouth_open = open;
        self
    }

    /// Get the grid to use based on mouth state.
    fn grid(&self) -> &'static [&'static str; 16] {
        if self.mouth_open {
            &GRID_OPEN
        } else {
            &GRID_CLOSED
        }
    }

    /// Get base dimensions at scale 1.
    fn base_dimensions(&self) -> (u16, u16) {
        match self.mode {
            MascotMode::Standard => (32, 16), // 2 chars wide per pixel
            MascotMode::HalfBlock => (16, 8), // 1 char wide, 2 pixels per char height
        }
    }

    /// Calculate the scale factor based on available area.
    fn calculate_scale(&self, area: Rect) -> Option<u16> {
        let (base_w, base_h) = self.base_dimensions();
        let max_scale_by_width = area.width / base_w;
        let max_scale_by_height = area.height / base_h;

        let scale = max_scale_by_width.min(max_scale_by_height);
        if scale >= 1 { Some(scale) } else { None }
    }
}

impl Widget for MascotWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let (base_w, base_h) = self.base_dimensions();

        // Check if terminal is too small
        if area.width < base_w || area.height < base_h {
            let msg = "Too small";
            let msg_len = msg.len() as u16;
            let x = area.x + (area.width.saturating_sub(msg_len) / 2);
            let y = area.y + (area.height / 2);

            if x < area.right() && y < area.bottom() {
                let line = Line::styled(msg, Style::default().fg(Color::Yellow));
                line.render(Rect::new(x, y, msg_len.min(area.width), 1), buf);
            }
            return;
        }

        // Calculate scale
        let scale = self.calculate_scale(area).unwrap_or(1);

        // Calculate centered position
        let scaled_width = base_w * scale;
        let scaled_height = base_h * scale;

        let offset_x = (area.width.saturating_sub(scaled_width)) / 2;
        let offset_y = (area.height.saturating_sub(scaled_height)) / 2;

        let grid = self.grid();
        match self.mode {
            MascotMode::Standard => {
                // Standard mode: 2 chars wide × 1 char tall per pixel
                for (row_idx, row) in grid.iter().enumerate() {
                    for (col_idx, pixel_char) in row.chars().enumerate() {
                        // Skip background/transparent pixels
                        let Some(color) = color_for_char(pixel_char) else {
                            continue;
                        };
                        let base_x = area.x + offset_x + (col_idx as u16 * 2 * scale);
                        let base_y = area.y + offset_y + (row_idx as u16 * scale);

                        for dy in 0..scale {
                            for dx in 0..(2 * scale) {
                                let x = base_x + dx;
                                let y = base_y + dy;
                                if x < area.right() && y < area.bottom() {
                                    let cell = &mut buf[(x, y)];
                                    cell.set_char(' ');
                                    cell.set_bg(color);
                                }
                            }
                        }
                    }
                }
            }
            MascotMode::HalfBlock => {
                // Half-block mode: use ▀/▄ to pack 2 vertical pixels per char
                for row_pair in 0..8 {
                    let top_row = &grid[row_pair * 2];
                    let bottom_row = &grid[row_pair * 2 + 1];

                    for col_idx in 0..16 {
                        let top_char = top_row.chars().nth(col_idx).unwrap_or(BG_CHAR);
                        let bottom_char = bottom_row.chars().nth(col_idx).unwrap_or(BG_CHAR);

                        let top_color = color_for_char(top_char);
                        let bottom_color = color_for_char(bottom_char);

                        // Skip if both pixels are transparent
                        if top_color.is_none() && bottom_color.is_none() {
                            continue;
                        }

                        let base_x = area.x + offset_x + (col_idx as u16 * scale);
                        let base_y = area.y + offset_y + (row_pair as u16 * scale);

                        for dy in 0..scale {
                            for dx in 0..scale {
                                let x = base_x + dx;
                                let y = base_y + dy;
                                if x < area.right() && y < area.bottom() {
                                    let cell = &mut buf[(x, y)];

                                    match (top_color, bottom_color) {
                                        (Some(top), Some(bot)) => {
                                            // Both pixels colored: ▀ with fg=top, bg=bottom
                                            cell.set_char('▀');
                                            cell.set_fg(top);
                                            cell.set_bg(bot);
                                        }
                                        (Some(top), None) => {
                                            // Only top pixel: ▀ with fg=top, default bg
                                            cell.set_char('▀');
                                            cell.set_fg(top);
                                        }
                                        (None, Some(bot)) => {
                                            // Only bottom pixel: ▄ with fg=bottom, default bg
                                            cell.set_char('▄');
                                            cell.set_fg(bot);
                                        }
                                        (None, None) => {
                                            // Already handled above
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Render the mascot widget to a frame.
///
/// This is a convenience function for rendering the mascot directly.
#[allow(dead_code)]
pub fn render_mascot(frame: &mut ratatui::Frame, area: Rect) {
    let widget = MascotWidget::new();
    frame.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_lookup_background() {
        // Background character returns None (transparent)
        let color = color_for_char('.');
        assert_eq!(color, None);
    }

    #[test]
    fn test_color_lookup_body() {
        let color = color_for_char('4');
        assert_eq!(color, Some(Color::Rgb(234, 170, 114)));
    }

    #[test]
    fn test_color_lookup_eyes() {
        let color = color_for_char('E');
        assert_eq!(color, Some(Color::Rgb(26, 26, 46)));
    }

    #[test]
    fn test_color_lookup_unknown() {
        // Unknown characters should return None (transparent)
        let color = color_for_char('X');
        assert_eq!(color, None);
    }

    #[test]
    fn test_scale_calculation_exact_fit() {
        let widget = MascotWidget::new();
        // Exactly 32 cols, 16 rows = scale 1
        let area = Rect::new(0, 0, 32, 16);
        assert_eq!(widget.calculate_scale(area), Some(1));
    }

    #[test]
    fn test_scale_calculation_double() {
        let widget = MascotWidget::new();
        // 64 cols, 32 rows = scale 2
        let area = Rect::new(0, 0, 64, 32);
        assert_eq!(widget.calculate_scale(area), Some(2));
    }

    #[test]
    fn test_scale_calculation_limited_by_height() {
        let widget = MascotWidget::new();
        // 96 cols (scale 3 by width), 32 rows (scale 2 by height) = scale 2
        let area = Rect::new(0, 0, 96, 32);
        assert_eq!(widget.calculate_scale(area), Some(2));
    }

    #[test]
    fn test_scale_calculation_limited_by_width() {
        let widget = MascotWidget::new();
        // 48 cols (scale 1 by width), 48 rows (scale 3 by height) = scale 1
        let area = Rect::new(0, 0, 48, 48);
        assert_eq!(widget.calculate_scale(area), Some(1));
    }

    #[test]
    fn test_scale_calculation_too_small() {
        let widget = MascotWidget::new();
        // Less than 32 cols
        let area = Rect::new(0, 0, 20, 16);
        assert_eq!(widget.calculate_scale(area), None);
    }

    #[test]
    fn test_grid_dimensions() {
        // Verify grid is exactly 16 rows
        assert_eq!(GRID.len(), 16);

        // Verify each row is exactly 16 characters
        for (idx, row) in GRID.iter().enumerate() {
            assert_eq!(
                row.len(),
                16,
                "Row {} has wrong length: expected 16, got {}",
                idx,
                row.len()
            );
        }
    }

    #[test]
    fn test_palette_contains_all_grid_chars() {
        // Collect all unique characters from the grid
        let grid_chars: std::collections::HashSet<char> =
            GRID.iter().flat_map(|row| row.chars()).collect();

        // Collect all palette characters
        let palette_chars: std::collections::HashSet<char> =
            PALETTE.iter().map(|&(c, _)| c).collect();

        // Every grid character should be in the palette
        for c in &grid_chars {
            assert!(
                palette_chars.contains(c),
                "Character '{}' is in grid but not in palette",
                c
            );
        }
    }

    #[test]
    fn test_widget_default() {
        let widget = MascotWidget::default();
        let area = Rect::new(0, 0, 40, 20);
        // Should not panic
        assert_eq!(widget.calculate_scale(area), Some(1));
    }
}
