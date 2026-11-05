//! Desktop - Classic Mac-style icon grid with Cardputer Zero layout.
//!
//! 操作:
//! - 矢印キー: アイコン選択移動
//! - Enter: アプリ起動（起動要求を発行）
//! - Esc: 何もしない（デスクトップが最背面）
//!
//! 描画:
//! - アイコン（24x24、Amber 染色）+ 最大2行ラベル
//! - 選択中アイコンは反転ハイライト

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::AssetStore;
use crate::font::FontSystem;
use crate::input::{PointerAction, PointerButton, PointerInput};
use crate::theme::ThemeManager;
use crate::ui;

/// デスクトップに並ぶアプリ定義。
#[derive(Clone)]
pub struct DesktopApp {
    /// アイコンのアセットキー（例: "icons/TextEditor"）
    pub icon: &'static str,
    /// 表示名
    pub label: &'static str,
    /// 起動時に発行するアプリ識別子
    pub kind: AppKind,
}

/// 起動できるアプリ種別。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AppKind {
    TextEditor,
    FileBrowser,
    Clock,
    Dictionary,
    Calculator,
    Snake,
}

/// デスクトップ状態。
pub struct Desktop {
    /// 並んだアプリ一覧
    pub apps: Vec<DesktopApp>,
    /// 選択中インデックス
    pub selected: usize,
    /// アイコングリッドのレイアウト（列数）
    pub cols: usize,
}

/// デスクトップからの要求。
pub enum DesktopAction {
    Launch(AppKind),
    None,
}

impl Desktop {
    pub fn new() -> Self {
        Desktop {
            apps: vec![
                DesktopApp {
                    icon: "icons/TextEditor",
                    label: "TextEditor",
                    kind: AppKind::TextEditor,
                },
                DesktopApp {
                    icon: "icons/Folder",
                    label: "Files",
                    kind: AppKind::FileBrowser,
                },
                DesktopApp {
                    icon: "icons/Clock",
                    label: "Clock",
                    kind: AppKind::Clock,
                },
                DesktopApp {
                    icon: "icons/Dictionary",
                    label: "Dictionary",
                    kind: AppKind::Dictionary,
                },
                DesktopApp {
                    icon: "icons/Calculator",
                    label: "Calculator",
                    kind: AppKind::Calculator,
                },
                DesktopApp {
                    icon: "icons/Snake",
                    label: "Snake",
                    kind: AppKind::Snake,
                },
            ],
            selected: 0,
            cols: 4,
        }
    }

    /// 画面幅に合わせてアイコングリッド列数を更新する。
    pub fn set_view(&mut self, w: u32, _h: u32) {
        if w <= 360 {
            // A right-aligned two-column desktop, not an INDEX page.
            self.cols = 2;
        } else {
            let usable_w = w as i32 - 48;
            self.cols = (usable_w / 72).max(1) as usize;
        }
    }

    /// キー処理。起動要求を返す。
    pub fn handle_key(&mut self, key: sdl2::keyboard::Keycode) -> DesktopAction {
        use sdl2::keyboard::Keycode;
        let n = self.apps.len();
        if self.cols == 2 {
            // Direct 1–6 app shortcuts remain available on the handheld.
            let direct = match key {
                Keycode::Num1 => Some(0),
                Keycode::Num2 => Some(1),
                Keycode::Num3 => Some(2),
                Keycode::Num4 => Some(3),
                Keycode::Num5 => Some(4),
                Keycode::Num6 => Some(5),
                _ => None,
            };
            if let Some(idx) = direct {
                self.selected = idx;
                if let Some(app) = self.apps.get(idx) {
                    return DesktopAction::Launch(app.kind);
                }
            }
        }
        match key {
            Keycode::Left => {
                if self.selected > 0 && (self.cols != 2 || self.selected % 2 == 1) {
                    self.selected -= 1;
                }
            }
            Keycode::Right => {
                if self.selected + 1 < n && (self.cols != 2 || self.selected % 2 == 0) {
                    self.selected += 1;
                }
            }
            Keycode::Up => {
                if self.selected >= self.cols {
                    self.selected -= self.cols;
                }
            }
            Keycode::Down => {
                if self.selected + self.cols < n {
                    self.selected += self.cols;
                }
            }
            Keycode::Home => self.selected = 0,
            Keycode::End => self.selected = n.saturating_sub(1),
            Keycode::Return | Keycode::KpEnter => {
                if let Some(app) = self.apps.get(self.selected) {
                    return DesktopAction::Launch(app.kind);
                }
            }
            _ => {}
        }
        DesktopAction::None
    }

    pub fn handle_pointer(
        &mut self,
        input: PointerInput,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) -> DesktopAction {
        if let Some(idx) = self.hit_icon(input.x, input.y, x, y, w, h) {
            self.selected = idx;
            if input.action == PointerAction::ButtonUp
                && input.button == Some(PointerButton::Left)
                && input.clicks >= 2
            {
                if let Some(app) = self.apps.get(idx) {
                    return DesktopAction::Launch(app.kind);
                }
            }
        }
        DesktopAction::None
    }

    /// 描画。
    pub fn draw(
        &self,
        c: &mut Canvas<Window>,
        assets: &mut AssetStore,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) {
        let amber = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let background = theme.get_color("background");
        let compact = compact_display(w, h);
        if compact {
            self.draw_compact(c, assets, font, theme, x, y, w, h);
            return;
        }

        // Classic desktop for the PC/Web target.
        // Use small, unscaled icons; selection is indicated by a thin border
        // and an inverted label, not a large filled tile.
        let icon_size = 24u32;
        let layout = self.layout(x, y, w, h);

        for (i, app) in self.apps.iter().enumerate() {
            let (cx, cy) = layout.cell_origin(i);

            let is_selected = i == self.selected;

            // アイコン描画（Amber 染色）
            let icon_dst = Rect::new(
                cx + (layout.cell_w - icon_size as i32) / 2,
                cy,
                icon_size,
                icon_size,
            );
            if compact {
                // One-pixel outline replaces the heavy inverse icon block.
                if is_selected {
                    ui::draw_rect(
                        c,
                        Rect::new(
                            icon_dst.x() - 2,
                            icon_dst.y() - 2,
                            icon_dst.width() + 4,
                            icon_dst.height() + 4,
                        ),
                        muted,
                    );
                }
                let _ = assets.blit_tinted(c, app.icon, Some(icon_dst), amber);
            } else if is_selected {
                c.set_draw_color(muted);
                let _ = c.fill_rect(Rect::new(
                    icon_dst.x() - 2,
                    icon_dst.y() - 2,
                    icon_dst.width() + 4,
                    icon_dst.height() + 4,
                ));
                let _ = assets.blit_tinted(c, app.icon, Some(icon_dst), background);
            } else {
                let _ = assets.blit_tinted(c, app.icon, Some(icon_dst), amber);
            }

            // Center each line independently; use a readable native bitmap font.
            for (line, label) in ui::wrap_icon_label(app.label, 52, |s| font.measure_text(s)).iter().enumerate() {
                let lw = font.measure_text(label);
                let lx = cx + (layout.cell_w - lw) / 2;
                let ly = cy + icon_size as i32 + 1 + line as i32 * 13;
                font.draw_text(c, lx, ly, label, amber);
                if is_selected { ui::line(c, lx, ly + 12, lx + lw, ly + 12, amber); }
            }
        }

        // 操作ヒント（下部）
        if !compact && h >= 56 {
            let hint = "Arrows: select  Enter: launch";
            let hw = font.measure_text(hint);
            let hx = x + ((w as i32 - hw) / 2).max(4);
            font.draw_text(c, hx, y + h as i32 - 16, hint, muted);
        }
    }

    /// Cardputer Zero: a real icon desktop, with System 6/7-inspired
    /// layout adapted to a 320x170 screen. The left side remains a workspace
    /// and all six apps sit in two right-aligned columns.
    fn draw_compact(
        &self,
        c: &mut Canvas<Window>,
        assets: &mut AssetStore,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) {
        let ink = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let paper = theme.get_color("background");
        let grid = self.layout(x, y, w, h);
        const ICON_SIZE: u32 = 24;

        for (i, app) in self.apps.iter().enumerate() {
            let (cx, cy) = grid.cell_origin(i);
            let selected = i == self.selected;
            let icon_x = cx + (grid.cell_w - ICON_SIZE as i32) / 2;
            let icon_rect = Rect::new(icon_x, cy, ICON_SIZE, ICON_SIZE);

            // Small focus border, never a large opaque icon tile.
            if selected {
                ui::draw_rect(
                    c,
                    Rect::new(icon_x - 2, cy - 2, ICON_SIZE + 4, ICON_SIZE + 4),
                    muted,
                );
            }
            let _ = assets.blit_tinted(c, app.icon, Some(icon_rect), ink);

            // Wrap long names to two centered lines without scaling the font.
            for (line, label) in ui::wrap_icon_label(app.label, 52, |s| font.measure_text(s)).iter().enumerate() {
                let lw = font.measure_text(label);
                let lx = cx + (grid.cell_w - lw) / 2;
                let ly = cy + ICON_SIZE as i32 + 1 + line as i32 * 13;
                if selected {
                    c.set_draw_color(ink);
                    let _ = c.fill_rect(Rect::new(lx - 3, ly - 1, (lw + 6) as u32, 14));
                }
                font.draw_text(c, lx, ly, label, if selected { paper } else { ink });
            }
        }
    }

    fn hit_icon(&self, px: i32, py: i32, x: i32, y: i32, w: u32, h: u32) -> Option<usize> {
        let layout = self.layout(x, y, w, h);
        self.apps.iter().enumerate().find_map(|(idx, _)| {
            let (cx, cy) = layout.cell_origin(idx);
            let hit = Rect::new(cx, cy - 3, layout.cell_w as u32, layout.cell_h as u32);
            if hit.contains_point((px, py)) {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn layout(&self, x: i32, y: i32, w: u32, h: u32) -> DesktopLayout {
        let cols = self.cols.max(1);
        let compact = compact_display(w, h);
        let cell_w = if compact { 68i32 } else { 72i32 };
        let cell_h = if compact { 50i32 } else { 56i32 };
        let grid_w = (cols as i32 * cell_w).min(w as i32);
        let grid_x = if compact {
            x + (w as i32 - grid_w - 10).max(0)
        } else {
            x + ((w as i32 - grid_w) / 2).max(8)
        };
        let grid_y = y + if compact { 2 } else { 24 };
        DesktopLayout {
            cols,
            cell_w,
            cell_h,
            grid_x,
            grid_y,
        }
    }
}

/// Physical screen coordinates are used directly; no interpolation or scaling.
fn compact_display(w: u32, h: u32) -> bool {
    w <= 360 && h <= 200
}

struct DesktopLayout {
    cols: usize,
    cell_w: i32,
    cell_h: i32,
    grid_x: i32,
    grid_y: i32,
}

impl DesktopLayout {
    fn cell_origin(&self, idx: usize) -> (i32, i32) {
        let col = idx % self.cols;
        let row = idx / self.cols;
        (
            self.grid_x + col as i32 * self.cell_w,
            self.grid_y + row as i32 * self.cell_h,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{PointerAction, PointerButton, PointerInput};

    #[test]
    fn cardputer_zero_desktop_uses_two_right_aligned_columns() {
        let mut desktop = Desktop::new();
        desktop.set_view(320, 154);
        assert_eq!(desktop.cols, 2);
        assert_eq!(desktop.apps.len(), 6);
        let layout = desktop.layout(2, 16, 320, 170);
        assert_eq!(layout.grid_x, 176);
        assert_eq!(layout.grid_y, 18);
        assert_eq!(layout.cell_origin(5), (244, 118));
        // Last label fits beneath the third icon row.
        assert!(layout.cell_origin(5).1 + 24 + 1 + 13 * 2 <= 170);
        // Left desktop workspace is not a clickable icon.
        assert_eq!(desktop.hit_icon(60, 50, 2, 16, 320, 170), None);
        assert_eq!(desktop.hit_icon(265, 123, 2, 16, 320, 170), Some(5));
    }

    #[test]
    fn arrows_follow_icon_grid_without_crossing_rows() {
        use sdl2::keyboard::Keycode;
        let mut desktop = Desktop::new();
        desktop.set_view(320, 154);
        desktop.handle_key(Keycode::Left);
        assert_eq!(desktop.selected, 0);
        desktop.handle_key(Keycode::Right);
        assert_eq!(desktop.selected, 1);
        desktop.handle_key(Keycode::Right);
        assert_eq!(desktop.selected, 1);
        desktop.handle_key(Keycode::Down);
        assert_eq!(desktop.selected, 3);
        desktop.handle_key(Keycode::Up);
        assert_eq!(desktop.selected, 1);
        desktop.handle_key(Keycode::Left);
        assert_eq!(desktop.selected, 0);
    }

    #[test]
    fn number_key_launches_handheld_icon() {
        let mut desktop = Desktop::new();
        desktop.set_view(320, 154);
        assert!(matches!(
            desktop.handle_key(sdl2::keyboard::Keycode::Num6),
            DesktopAction::Launch(AppKind::Snake)
        ));
        assert_eq!(desktop.selected, 5);
    }

    #[test]
    fn double_clicking_icon_requests_launch() {
        let mut desktop = Desktop::new();
        desktop.set_view(480, 300);
        let input = PointerInput::button(PointerAction::ButtonUp, 50, 50, PointerButton::Left, 2);

        assert!(matches!(
            desktop.handle_pointer(input, 2, 20, 480, 300),
            DesktopAction::Launch(AppKind::TextEditor)
        ));
    }
}
