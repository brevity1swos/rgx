use ratatui::style::Color;

pub const CAPTURE_COLORS: &[Color] = &[
    Color::Rgb(166, 227, 161), // green
    Color::Rgb(249, 226, 175), // yellow
    Color::Rgb(137, 180, 250), // blue
    Color::Rgb(245, 194, 231), // pink
    Color::Rgb(180, 190, 254), // lavender
    Color::Rgb(148, 226, 213), // teal
    Color::Rgb(250, 179, 135), // peach
    Color::Rgb(203, 166, 247), // mauve
    Color::Rgb(116, 199, 236), // sapphire
    Color::Rgb(242, 205, 205), // flamingo
    Color::Rgb(243, 139, 168), // red
    Color::Rgb(137, 220, 235), // sky
];

pub const MATCH_BG: Color = Color::Rgb(69, 71, 90);
pub const SURFACE0: Color = Color::Rgb(49, 50, 68);
pub const SURFACE1: Color = Color::Rgb(69, 71, 90);
pub const TEXT: Color = Color::Rgb(205, 214, 244);
pub const SUBTEXT: Color = Color::Rgb(166, 173, 200);
pub const OVERLAY: Color = Color::Rgb(108, 112, 134);
pub const RED: Color = Color::Rgb(243, 139, 168);
pub const GREEN: Color = Color::Rgb(166, 227, 161);
pub const BLUE: Color = Color::Rgb(137, 180, 250);
pub const BASE: Color = Color::Rgb(30, 30, 46);

pub fn capture_color(index: usize) -> Color {
    CAPTURE_COLORS[index % CAPTURE_COLORS.len()]
}
