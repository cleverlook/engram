use ratatui::style::Color;

/// Catppuccin Mocha palette
/// https://catppuccin.com/palette
#[allow(dead_code)]
pub mod catppuccin {
    use ratatui::style::Color;

    pub const BASE: Color = Color::Rgb(30, 30, 46); // #1e1e2e
    pub const TEXT: Color = Color::Rgb(205, 214, 244); // #cdd6f4
    pub const SUBTEXT: Color = Color::Rgb(166, 173, 200); // #a6adc8
    pub const OVERLAY: Color = Color::Rgb(108, 112, 134); // #6c7086 (dim)
    pub const SURFACE: Color = Color::Rgb(49, 50, 68); // #313244 (selected bg)

    pub const TEAL: Color = Color::Rgb(148, 226, 213); // #94e2d5 (cyan replacement)
    pub const GREEN: Color = Color::Rgb(166, 227, 161); // #a6e3a1
    pub const YELLOW: Color = Color::Rgb(249, 226, 175); // #f9e2af
    pub const PEACH: Color = Color::Rgb(250, 179, 135); // #fab387 (warm orange)
    pub const RED: Color = Color::Rgb(243, 139, 168); // #f38ba8
    pub const MAUVE: Color = Color::Rgb(203, 166, 247); // #cba6f7 (purple)
    pub const BLUE: Color = Color::Rgb(137, 180, 250); // #89b4fa
    pub const SAPPHIRE: Color = Color::Rgb(116, 199, 236); // #74c7ec
}

/// Semantic color mapping for the app
pub struct Theme;

#[allow(dead_code)]
impl Theme {
    // Text
    pub const TEXT: Color = catppuccin::TEXT;
    pub const DIM: Color = catppuccin::OVERLAY;
    pub const SUBTLE: Color = catppuccin::SUBTEXT;

    // Accents
    pub const ACCENT: Color = catppuccin::MAUVE; // primary accent (node IDs, branches)
    pub const HIGHLIGHT: Color = catppuccin::YELLOW; // selected / active
    pub const SECONDARY: Color = catppuccin::TEAL; // edges, links
    pub const HEADER: Color = catppuccin::PEACH; // section headers

    // Status
    pub const SUCCESS: Color = catppuccin::GREEN;
    pub const WARNING: Color = catppuccin::YELLOW;
    pub const ERROR: Color = catppuccin::RED;
    pub const STALE: Color = catppuccin::PEACH;

    // Backgrounds
    pub const BASE_BG: Color = catppuccin::BASE;
    pub const SELECTED_BG: Color = catppuccin::SURFACE;
}

pub fn status_color(status: &crate::models::node::NodeStatus) -> Color {
    use crate::models::node::NodeStatus;
    match status {
        NodeStatus::Active => Theme::SUCCESS,
        NodeStatus::Dirty => Theme::WARNING,
        NodeStatus::Stale => Theme::STALE,
        NodeStatus::Deprecated => Theme::ERROR,
    }
}

pub fn status_icon(status: &crate::models::node::NodeStatus) -> &'static str {
    use crate::models::node::NodeStatus;
    match status {
        NodeStatus::Active => "●",
        NodeStatus::Dirty => "◐",
        NodeStatus::Stale => "○",
        NodeStatus::Deprecated => "✕",
    }
}
