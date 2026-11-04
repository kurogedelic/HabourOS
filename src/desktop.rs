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
        if w <= 360 {
            // Cardputer Zero uses an index list, not desktop icon columns.
            self.cols = 1;
        } else {
            let usable_w = w as i32 - 48;
            self.cols = (usable_w / 72).max(1) as usize;
        }
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
            self.draw_compact(c, font, theme, x, y, w, h);
            return;
        }

        // Classic desktop for the PC/Web target.
        // Use small, unscaled icons; selection is indicated by a thin border
        // and an inverted label, not a large filled tile.
        let icon_size = if compact { 24u32 } else { 32u32 };
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

            let label_w = font.measure_text(app.label);
            let lx = cx + (layout.cell_w - label_w) / 2;
            let ly = cy + icon_size as i32 + 3;
            if compact && is_selected {
                c.set_draw_color(amber);
                let _ = c.fill_rect(Rect::new(lx - 3, ly - 1, (label_w + 6).max(1) as u32, 15));
            }
            font.draw_text(
                c,
                lx,
                ly,
                app.label,
                if compact && is_selected {
                    background
                } else {
                    amber
                },
            );

            if is_selected && !compact {
                ui::line(c, lx, ly + 13, lx + label_w, ly + 13, amber);
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

    /// Small-screen INDEX page: a functional text-based launch list and a
    /// context preview, taking cues from Fairlight CMI page layouts rather
    /// than scaling desktop icons into a handheld display.
    ///
    /// Uses the same four palette colors as the rest of HarbourOS; graphics
    /// are drawn directly from geometry and ASCII font, no extra bitmaps.
    fn draw_compact(
        &self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) {
        let ink = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let shadow = theme.get_color("shadow");
        let background = theme.get_color("background");

        let left = x + 6;
        let right = x + w as i32 - 8;
        let split = x + 145;
        let top = y + 2;
        // The caller passes the full output height, not the available
        // content height below the menu bar.
        let bottom = h as i32 - 1;

        font.draw_text(c, left, top, "INDEX / APPLICATIONS", ink);
        ui::line(c, left, top + 15, right, top + 15, muted);
        ui::line(c, split, top + 17, split, bottom - 20, shadow);

        for (i, app) in self.apps.iter().enumerate() {
            let row_y = y + 22 + i as i32 * 18;
            let label = match app.kind {
                AppKind::TextEditor => "TEXT EDITOR",
                AppKind::FileBrowser => "FILES",
                AppKind::Clock => "CLOCK",
                AppKind::Dictionary => "DICTIONARY",
                AppKind::Calculator => "CALCULATOR",
                AppKind::Snake => "SNAKE",
            };
            let text = format!("{:02}  {}", i + 1, label);
            if i == self.selected {
                c.set_draw_color(muted);
                let _ = c.fill_rect(Rect::new(left - 2, row_y - 2, 133, 17));
                font.draw_text(c, left, row_y, &text, background);
                font.draw_text(c, split - 16, row_y, ">", ink);
            } else {
                font.draw_text(c, left, row_y, &text, ink);
            }
        }

        // Selected application's display is an actual contextual preview,
        // not a synthetic system status or an unrelated decorative waveform.
        if let Some(app) = self.apps.get(self.selected) {
            let px = split + 10;
            let pw = (right - px).max(0);
            let kind = match app.kind {
                AppKind::TextEditor => "EDIT / BUFFER",
                AppKind::FileBrowser => "FILES / TREE",
                AppKind::Clock => "CLOCK / DIAL",
                AppKind::Dictionary => "DICT / SEARCH",
                AppKind::Calculator => "CALC / STACK",
                AppKind::Snake => "SNAKE / FIELD",
            };
            font.draw_text(c, px, y + 24, kind, ink);
            font.draw_text(c, px, y + 40, &format!("MODULE {:02} / 06", self.selected + 1), muted);
            ui::line(c, px, y + 58, px + pw, y + 58, shadow);
            self.draw_compact_preview(c, font, theme, app.kind, px, y + 63);
        }

        ui::line(c, left, bottom - 17, right, bottom - 17, muted);
        font.draw_text(c, left, bottom - 13, "FN+F/Z/X/C MOVE    ENTER OPEN", muted);
    }

    fn draw_compact_preview(
        &self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        kind: AppKind,
        x: i32,
        y: i32,
    ) {
        let ink = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let shadow = theme.get_color("shadow");
        match kind {
            AppKind::TextEditor => {
                font.draw_text(c, x, y, "UNTITLED.TXT", muted);
                font.draw_text(c, x, y + 17, "01  HELLO, WORLD", ink);
                font.draw_text(c, x, y + 31, "02  _", ink);
            }
            AppKind::FileBrowser => {
                font.draw_text(c, x, y, "/ROOT", ink);
                font.draw_text(c, x + 6, y + 15, "|-- README.TXT", muted);
                font.draw_text(c, x + 6, y + 30, "|-- NOTES/", muted);
            }
            AppKind::Clock => {
                // Small oscilloscope-like square wave whose grid encodes
                // the twelve clock divisions without pretending live data.
                ui::line(c, x, y + 24, x + 136, y + 24, shadow);
                for i in 0..13 {
                    let tx = x + i * 11;
                    ui::line(c, tx, y + 20, tx, y + 28, shadow);
                }
                font.draw_text(c, x, y, "00  03  06  09  12", muted);
                font.draw_text(c, x, y + 33, "HOURS / MINUTES", ink);
            }
            AppKind::Dictionary => {
                font.draw_text(c, x, y, "A B C D E F G H I", ink);
                font.draw_text(c, x, y + 15, "J K L M N O P Q R", muted);
                font.draw_text(c, x, y + 30, "S T U V W X Y Z", muted);
            }
            AppKind::Calculator => {
                font.draw_text(c, x, y, "   0.0000", ink);
                ui::line(c, x, y + 15, x + 128, y + 15, muted);
                font.draw_text(c, x, y + 20, "7  8  9  /", muted);
                font.draw_text(c, x, y + 34, "4  5  6  *", muted);
            }
            AppKind::Snake => {
                let sx = x + 7;
                for i in 0..9 {
                    ui::line(c, sx + i * 12, y + 2, sx + i * 12, y + 38, shadow);
                }
                for i in 0..4 {
                    ui::line(c, sx, y + 2 + i * 12, sx + 96, y + 2 + i * 12, shadow);
                }
                ui::line(c, sx + 12, y + 26, sx + 48, y + 26, ink);
                ui::line(c, sx + 48, y + 26, sx + 48, y + 14, ink);
                ui::line(c, sx + 48, y + 14, sx + 72, y + 14, ink);
                c.set_draw_color(ink);
                let _ = c.fill_rect(Rect::new(sx + 84, y + 26, 4, 4));
            }
        }
    }

    fn hit_icon(&self, px: i32, py: i32, x: i32, y: i32, w: u32, h: u32) -> Option<usize> {
        if compact_display(w, h) {
            let first_y = y + 22;
            if px >= x + 4 && px <= x + 139 && py >= first_y {
                let index = ((py - first_y) / 18) as usize;
                if index < self.apps.len() {
                    return Some(index);
                }
            }
            return None;
        }
        let layout = self.layout(x, y, w, h);
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

    fn layout(&self, x: i32, y: i32, w: u32, h: u32) -> DesktopLayout {
        let cols = self.cols.max(1);
        let compact = compact_display(w, h);
        let cell_w = if compact { 92i32 } else { 72i32 };
        let cell_h = if compact { 54i32 } else { 56i32 };
        let grid_w = (cols as i32 * cell_w).min(w as i32);
        let grid_x = x + ((w as i32 - grid_w) / 2).max(if compact { 4 } else { 8 });
        let grid_y = y + if compact { 18 } else { 24 };
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
    fn cardputer_zero_index_fits_native_resolution() {
        let mut desktop = Desktop::new();
        desktop.set_view(320, 150);
        assert_eq!(desktop.cols, 1);
        assert_eq!(desktop.apps.len(), 6);
        let last_row = 20 + 22 + 5 * 18;
        assert!(last_row + 15 < 170);
        let row_click = desktop.hit_icon(18, last_row + 4, 2, 20, 320, 170);
        assert_eq!(row_click, Some(5));
        assert!(desktop.hit_icon(210, last_row, 2, 20, 320, 170).is_none());
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
