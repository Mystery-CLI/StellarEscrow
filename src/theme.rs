/// Theme preference keys — use with `set_user_preference` / `get_user_preference`.
///
/// All values are stored as UTF-8 strings via the existing `UserPreference` system.
/// Clients SHOULD validate values against the constants below before storing.

// ── Keys ─────────────────────────────────────────────────────────────────────

/// Color mode: `"light"` | `"dark"` | `"system"`
pub const PREF_THEME_MODE: &str = "theme.mode";

/// Accent color as a hex string, e.g. `"#6366f1"`
pub const PREF_THEME_COLOR: &str = "theme.color";

/// Font size: `"sm"` | `"md"` | `"lg"`
pub const PREF_FONT_SIZE: &str = "theme.font_size";

// ── Valid values ──────────────────────────────────────────────────────────────

pub const THEME_LIGHT: &str = "light";
pub const THEME_DARK: &str = "dark";
pub const THEME_SYSTEM: &str = "system";

pub const FONT_SM: &str = "sm";
pub const FONT_MD: &str = "md";
pub const FONT_LG: &str = "lg";
