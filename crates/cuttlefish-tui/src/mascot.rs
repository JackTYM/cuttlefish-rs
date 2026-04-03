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

/// 16x16 pixel art grid for the cuttlefish mascot.
///
/// Each character maps to a color in the palette.
const GRID: [&str; 16] = [
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

/// Color palette mapping characters to RGB colors.
const PALETTE: &[(char, (u8, u8, u8))] = &[
    ('.', (10, 22, 40)),    // background (dark blue)
    ('1', (184, 106, 53)),  // outline
    ('2', (200, 121, 65)),  // body outer
    ('3', (212, 136, 78)),  // body mid
    ('4', (234, 170, 114)), // body inner
    ('H', (245, 195, 140)), // highlight
    ('E', (26, 26, 46)),    // eyes (dark)
    ('s', (160, 80, 20)),   // stripe
    ('a', (100, 58, 22)),   // fin dark
    ('b', (140, 82, 30)),   // fin light
    ('t', (180, 100, 40)),  // tentacle 1
    ('u', (140, 78, 28)),   // tentacle 2
    ('v', (95, 50, 15)),    // tentacle 3
];

/// Minimum columns required to display the mascot.
const MIN_COLS: u16 = 34;

/// Minimum rows required to display the mascot.
const MIN_ROWS: u16 = 18;

/// Look up a color from the palette by character.
///
/// Returns the background color if the character is not found.
fn color_for_char(c: char) -> Color {
    for &(palette_char, (r, g, b)) in PALETTE {
        if palette_char == c {
            return Color::Rgb(r, g, b);
        }
    }
    // Default to background color
    Color::Rgb(10, 22, 40)
}

/// A widget that renders the cuttlefish mascot.
///
/// The mascot scales to fill the available area while maintaining aspect ratio.
/// Each pixel renders as 2 characters wide × 1 line tall for a square appearance.
#[derive(Debug, Clone, Default)]
pub struct MascotWidget;

impl MascotWidget {
    /// Create a new mascot widget.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Calculate the scale factor based on available area.
    ///
    /// Returns `None` if the area is too small.
    fn calculate_scale(&self, area: Rect) -> Option<u16> {
        // Each pixel is 2 chars wide, 1 line tall
        // Grid is 16x16 pixels = 32 chars wide, 16 lines tall at scale 1
        let max_scale_by_width = area.width / 32;
        let max_scale_by_height = area.height / 16;

        let scale = max_scale_by_width.min(max_scale_by_height);

        if scale >= 1 { Some(scale) } else { None }
    }
}

impl Widget for MascotWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Check if terminal is too small
        if area.width < MIN_COLS || area.height < MIN_ROWS {
            let msg = "Terminal too small";
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
        let scaled_width = 32 * scale; // 16 pixels * 2 chars each
        let scaled_height = 16 * scale;

        let offset_x = (area.width.saturating_sub(scaled_width)) / 2;
        let offset_y = (area.height.saturating_sub(scaled_height)) / 2;

        // Render each pixel
        for (row_idx, row) in GRID.iter().enumerate() {
            for (col_idx, pixel_char) in row.chars().enumerate() {
                let color = color_for_char(pixel_char);

                // Calculate the top-left position of this scaled pixel
                let base_x = area.x + offset_x + (col_idx as u16 * 2 * scale);
                let base_y = area.y + offset_y + (row_idx as u16 * scale);

                // Fill the scaled pixel area
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
        let color = color_for_char('.');
        assert_eq!(color, Color::Rgb(10, 22, 40));
    }

    #[test]
    fn test_color_lookup_body() {
        let color = color_for_char('4');
        assert_eq!(color, Color::Rgb(234, 170, 114));
    }

    #[test]
    fn test_color_lookup_eyes() {
        let color = color_for_char('E');
        assert_eq!(color, Color::Rgb(26, 26, 46));
    }

    #[test]
    fn test_color_lookup_unknown() {
        // Unknown characters should return background color
        let color = color_for_char('X');
        assert_eq!(color, Color::Rgb(10, 22, 40));
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
