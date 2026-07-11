//! Desktop - アイコングリッド型デスクトップ（Classic Mac 風）。
//!
//! 操作:
//! - 矢印キー: アイコン選択移動
//! - Enter: アプリ起動（起動要求を発行）
//! - Esc: 何もしない（デスクトップが最背面）
//!
//! 描画:
//! - アイコン（32x32、Amber 染色）+ ラベル（アイコン名）
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
        let usable_w = w as i32 - 48;
        self.cols = (usable_w / 72).max(1) as usize;
    }

    /// キー処理。起動要求を返す。
    pub fn handle_key(&mut self, key: sdl2::keyboard::Keycode) -> DesktopAction {
        use sdl2::keyboard::Keycode;
        let n = self.apps.len();
        match key {
            Keycode::Left => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
            }
            Keycode::Right => {
                if self.selected + 1 < n {
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
        _h: u32,
    ) -> DesktopAction {
        if let Some(idx) = self.hit_icon(input.x, input.y, x, y, w) {
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

        let icon_size = 32u32;
        let layout = self.layout(x, y, w);

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
            // 選択中は背景を Amber で塗ってアイコンを反転（黒アイコン風）
            if is_selected {
                c.set_draw_color(muted);
                let _ = c.fill_rect(Rect::new(
                    icon_dst.x() - 2,
                    icon_dst.y() - 2,
                    icon_dst.width() + 4,
                    icon_dst.height() + 4,
                ));
                // 白×透明アイコンをそのまま描くと Amber 背景に溶けるので、
                // 背景色で染色して「反転」表現
                let _ = assets.blit_tinted(c, app.icon, Some(icon_dst), background);
            } else {
                let _ = assets.blit_tinted(c, app.icon, Some(icon_dst), amber);
            }

            // ラベル
            let label_w = font.measure_text(app.label);
            let lx = cx + (layout.cell_w - label_w) / 2;
            let ly = cy + icon_size as i32 + 2;
            let label_col = if is_selected { amber } else { amber };
            font.draw_text(c, lx, ly, app.label, label_col);

            // 選択中ラベルの下線（Classic 風）
            if is_selected {
                ui::line(c, lx, ly + 13, lx + label_w, ly + 13, amber);
            }
        }

        // 操作ヒント（下部）
        if h >= 56 {
            let hint = "Arrows: select  Enter: launch";
            let hw = font.measure_text(hint);
            let hx = x + ((w as i32 - hw) / 2).max(4);
            font.draw_text(c, hx, y + h as i32 - 16, hint, muted);
        }
    }

    fn hit_icon(&self, px: i32, py: i32, x: i32, y: i32, w: u32) -> Option<usize> {
        let layout = self.layout(x, y, w);
        self.apps.iter().enumerate().find_map(|(idx, _)| {
            let (cx, cy) = layout.cell_origin(idx);
            let hit = Rect::new(cx, cy, layout.cell_w as u32, layout.cell_h as u32);
            if hit.contains_point((px, py)) {
                Some(idx)
            } else {
                None
            }
        })
    }

    fn layout(&self, x: i32, y: i32, w: u32) -> DesktopLayout {
        let cols = self.cols.max(1);
        let cell_w = 72i32;
        let cell_h = 56i32;
        let grid_w = (cols as i32 * cell_w).min(w as i32);
        let grid_x = x + ((w as i32 - grid_w) / 2).max(8);
        let grid_y = y + 24;
        DesktopLayout {
            cols,
            cell_w,
            cell_h,
            grid_x,
            grid_y,
        }
    }
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
