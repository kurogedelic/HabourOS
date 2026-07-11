use sdl2::pixels::Color;
use serde::Deserialize;
use std::fs;

const DEFAULT_PALETTE: [&str; 4] = ["#000000", "#3a2108", "#9a5a12", "#ffb040"];

#[derive(Debug, Deserialize)]
pub struct ThemeColors {
    #[serde(default = "default_palette")]
    pub palette: Vec<String>,

    #[serde(default)]
    pub background: Option<String>,
    #[serde(default)]
    pub foreground: Option<String>,
    #[serde(default)]
    pub primary: Option<String>,
    #[serde(default)]
    pub secondary: Option<String>,
    #[serde(default)]
    pub accent: Option<String>,
    #[serde(default)]
    pub muted: Option<String>,
    #[serde(default)]
    pub highlight: Option<String>,
    #[serde(default)]
    pub shadow: Option<String>,
    #[serde(default)]
    pub topbar_bg: Option<String>,

    // --- widget tone（Classic 風ベベル用） ---
    #[serde(default)]
    pub widget_face: Option<String>,
    #[serde(default)]
    pub widget_face_active: Option<String>,
    #[serde(default)]
    pub widget_outline: Option<String>,
    #[serde(default)]
    pub widget_outline_dim: Option<String>,
    #[serde(default)]
    pub widget_highlight: Option<String>,
    #[serde(default)]
    pub widget_shadow: Option<String>,
    #[serde(default)]
    pub titlebar_stripe_a: Option<String>,
    #[serde(default)]
    pub titlebar_stripe_b: Option<String>,
}

fn default_palette() -> Vec<String> {
    DEFAULT_PALETTE
        .iter()
        .map(|color| color.to_string())
        .collect()
}

#[derive(Debug, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
}

impl Theme {
    pub fn load_from_yaml(path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let theme: Theme = serde_yaml::from_str(&content).map_err(|e| e.to_string())?;
        Ok(theme)
    }

    pub fn parse_hex_color(hex: &str) -> Result<Color, String> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return Err("Invalid hex color format".to_string());
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| "Invalid hex color")?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| "Invalid hex color")?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| "Invalid hex color")?;

        Ok(Color::RGB(r, g, b))
    }
}

pub struct ThemeManager {
    pub theme: Theme,
}

impl ThemeManager {
    pub fn new(theme: Theme) -> Self {
        ThemeManager { theme }
    }

    /// 色キー名から Color を取得する。未知のキーは白を返す。
    pub fn get_color(&self, color_key: &str) -> Color {
        let colors = &self.theme.colors;
        let fallback_step = match color_key {
            "background" | "widget_face" => Some(0),
            "shadow" | "widget_shadow" | "titlebar_stripe_b" => Some(1),
            "secondary" | "muted" | "topbar_bg" | "widget_face_active" | "widget_outline_dim"
            | "titlebar_stripe_a" => Some(2),
            "foreground" | "primary" | "accent" | "highlight" | "widget_outline"
            | "widget_highlight" => Some(3),
            "palette0" | "tone0" => Some(0),
            "palette1" | "tone1" => Some(1),
            "palette2" | "tone2" => Some(2),
            "palette3" | "tone3" => Some(3),
            _ => None,
        };

        let hex = match color_key {
            "background" => colors.background.as_deref(),
            "foreground" => colors.foreground.as_deref(),
            "primary" => colors.primary.as_deref(),
            "secondary" => colors.secondary.as_deref(),
            "accent" => colors.accent.as_deref(),
            "muted" => colors.muted.as_deref(),
            "highlight" => colors.highlight.as_deref(),
            "shadow" => colors.shadow.as_deref(),
            "topbar_bg" => colors.topbar_bg.as_deref(),
            "widget_face" => colors.widget_face.as_deref(),
            "widget_face_active" => colors.widget_face_active.as_deref(),
            "widget_outline" => colors.widget_outline.as_deref(),
            "widget_outline_dim" => colors.widget_outline_dim.as_deref(),
            "widget_highlight" => colors.widget_highlight.as_deref(),
            "widget_shadow" => colors.widget_shadow.as_deref(),
            "titlebar_stripe_a" => colors.titlebar_stripe_a.as_deref(),
            "titlebar_stripe_b" => colors.titlebar_stripe_b.as_deref(),
            _ => None,
        }
        .or_else(|| fallback_step.and_then(|step| self.palette_hex(step)));

        let Some(hex) = hex else {
            return Color::RGB(255, 255, 255);
        };
        Theme::parse_hex_color(hex).unwrap_or(Color::RGB(255, 255, 255))
    }

    fn palette_hex(&self, step: usize) -> Option<&str> {
        self.theme
            .colors
            .palette
            .get(step)
            .map(String::as_str)
            .or_else(|| DEFAULT_PALETTE.get(step).copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn color_tuple(color: Color) -> (u8, u8, u8) {
        (color.r, color.g, color.b)
    }

    #[test]
    fn semantic_colors_fall_back_to_palette_steps() {
        let theme: Theme = serde_yaml::from_str(
            r##"
name: test
colors:
  palette:
    - "#000000"
    - "#111111"
    - "#222222"
    - "#333333"
"##,
        )
        .unwrap();
        let tm = ThemeManager::new(theme);

        assert_eq!(color_tuple(tm.get_color("background")), (0, 0, 0));
        assert_eq!(color_tuple(tm.get_color("shadow")), (17, 17, 17));
        assert_eq!(color_tuple(tm.get_color("muted")), (34, 34, 34));
        assert_eq!(color_tuple(tm.get_color("foreground")), (51, 51, 51));
        assert_eq!(color_tuple(tm.get_color("palette2")), (34, 34, 34));
    }

    #[test]
    fn explicit_semantic_color_overrides_palette_step() {
        let theme: Theme = serde_yaml::from_str(
            r##"
name: test
colors:
  palette:
    - "#000000"
    - "#111111"
    - "#222222"
    - "#333333"
  foreground: "#abcdef"
"##,
        )
        .unwrap();
        let tm = ThemeManager::new(theme);

        assert_eq!(color_tuple(tm.get_color("foreground")), (171, 205, 239));
        assert_eq!(color_tuple(tm.get_color("highlight")), (51, 51, 51));
    }
}
