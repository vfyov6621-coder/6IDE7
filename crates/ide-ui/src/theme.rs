//! Theme and color definitions for 6IDE7
//!
//! Modern dark theme inspired by popular IDEs like VS Code and IntelliJ.

use druid::{Color, Env, Key, theme};

// Color palette keys
pub const BACKGROUND: Key<Color> = Key::new("ide.background");
pub const SURFACE: Key<Color> = Key::new("ide.surface");
pub const SURFACE_HOVER: Key<Color> = Key::new("ide.surface-hover");
pub const BORDER: Key<Color> = Key::new("ide.border");
pub const TEXT: Key<Color> = Key::new("ide.text");
pub const TEXT_SECONDARY: Key<Color> = Key::new("ide.text-secondary");
pub const TEXT_MUTED: Key<Color> = Key::new("ide.text-muted");
pub const ACCENT: Key<Color> = Key::new("ide.accent");
pub const ACCENT_HOVER: Key<Color> = Key::new("ide.accent-hover");
pub const SUCCESS: Key<Color> = Key::new("ide.success");
pub const WARNING: Key<Color> = Key::new("ide.warning");
pub const ERROR: Key<Color> = Key::new("ide.error");
pub const INFO: Key<Color> = Key::new("ide.info");

// Block colors
pub const BLOCK_IO: Key<Color> = Key::new("ide.block-io");
pub const BLOCK_DATA: Key<Color> = Key::new("ide.block-data");
pub const BLOCK_CONTROL: Key<Color> = Key::new("ide.block-control");
pub const BLOCK_FUNCTION: Key<Color> = Key::new("ide.block-function");
pub const BLOCK_MATH: Key<Color> = Key::new("ide.block-math");
pub const BLOCK_STRING: Key<Color> = Key::new("ide.block-string");

// Sizing keys
pub const BORDER_RADIUS: Key<f64> = Key::new("ide.border-radius");
pub const SPACING_UNIT: Key<f64> = Key::new("ide.spacing-unit");

/// Apply the 6IDE7 dark theme to the environment
pub fn apply_theme(env: &mut Env) {
    // Base colors - Modern dark theme
    env.set(BACKGROUND, Color::rgb8(0x1e, 0x1e, 0x2e));         // Dark blue-gray background
    env.set(SURFACE, Color::rgb8(0x25, 0x25, 0x3a));            // Slightly lighter surface
    env.set(SURFACE_HOVER, Color::rgb8(0x2d, 0x2d, 0x44));      // Hover state
    env.set(BORDER, Color::rgb8(0x3d, 0x3d, 0x5c));             // Subtle border
    env.set(TEXT, Color::rgb8(0xe0, 0xe0, 0xe8));               // Primary text
    env.set(TEXT_SECONDARY, Color::rgb8(0xb0, 0xb0, 0xc0));     // Secondary text
    env.set(TEXT_MUTED, Color::rgb8(0x70, 0x70, 0x88));         // Muted text
    
    // Accent colors - Vibrant purple-blue
    env.set(ACCENT, Color::rgb8(0x7c, 0x3a, 0xed));             // Purple accent
    env.set(ACCENT_HOVER, Color::rgb8(0x96, 0x4a, 0xff));       // Lighter purple
    
    // Status colors
    env.set(SUCCESS, Color::rgb8(0x22, 0xc5, 0x5e));            // Green
    env.set(WARNING, Color::rgb8(0xf5, 0x9e, 0x0b));            // Amber
    env.set(ERROR, Color::rgb8(0xef, 0x44, 0x44));              // Red
    env.set(INFO, Color::rgb8(0x3b, 0x82, 0xf6));               // Blue
    
    // Block category colors
    env.set(BLOCK_IO, Color::rgb8(0x3b, 0x82, 0xf6));           // Blue - I/O blocks
    env.set(BLOCK_DATA, Color::rgb8(0x22, 0xc5, 0x5e));         // Green - Data blocks
    env.set(BLOCK_CONTROL, Color::rgb8(0xf5, 0x9e, 0x0b));      // Amber - Control flow
    env.set(BLOCK_FUNCTION, Color::rgb8(0xa8, 0x55, 0xf7));     // Purple - Functions
    env.set(BLOCK_MATH, Color::rgb8(0xec, 0x48, 0x99));         // Pink - Math
    env.set(BLOCK_STRING, Color::rgb8(0x14, 0xb8, 0xa6));       // Teal - Strings
    
    // Sizing
    env.set(BORDER_RADIUS, 6.0);
    env.set(SPACING_UNIT, 8.0);
    
    // Druid theme overrides
    env.set(theme::WINDOW_BACKGROUND_COLOR, env.get(BACKGROUND));
    env.set(theme::LABEL_COLOR, env.get(TEXT));
    env.set(theme::BORDER_LIGHT, env.get(BORDER));
    env.set(theme::BORDER_DARK, env.get(BORDER));
    env.set(theme::BACKGROUND_LIGHT, env.get(SURFACE));
    env.set(theme::BACKGROUND_DARK, env.get(SURFACE_HOVER));
    env.set(theme::FOREGROUND_LIGHT, env.get(TEXT));
    env.set(theme::FOREGROUND_DARK, env.get(TEXT_SECONDARY));
    env.set(theme::PRIMARY_LIGHT, env.get(ACCENT));
    env.set(theme::PRIMARY_DARK, env.get(ACCENT_HOVER));
    env.set(theme::BUTTON_LIGHT, env.get(SURFACE_HOVER));
    env.set(theme::BUTTON_DARK, env.get(SURFACE));
    env.set(theme::DISABLED_TEXT_COLOR, env.get(TEXT_MUTED));
    env.set(theme::PLACEHOLDER_COLOR, env.get(TEXT_MUTED));
    env.set(theme::CURSOR_COLOR, env.get(ACCENT));
    env.set(theme::SELECTION_COLOR, Color::rgb8(0x7c, 0x3a, 0xed).with_alpha(0.3));
}

/// Helper to create a color with alpha
pub fn color_with_alpha(base: Color, alpha: f64) -> Color {
    let (r, g, b, _) = base.as_rgba();
    Color::rgba(r, g, b, alpha)
}

/// Gradient colors for blocks
pub fn block_gradient_start(env: &Env, block_type: &str) -> Color {
    match block_type {
        "io" => env.get(BLOCK_IO),
        "data" => env.get(BLOCK_DATA),
        "control" => env.get(BLOCK_CONTROL),
        "function" => env.get(BLOCK_FUNCTION),
        "math" => env.get(BLOCK_MATH),
        "string" => env.get(BLOCK_STRING),
        _ => env.get(ACCENT),
    }
}
