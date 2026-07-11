//! ConfirmDialog - アプリ終了確認ダイアログ。
//!
//! 方針:
//! - outline-only（fill なし）。Black 背景 + Amber outline/文字
//! - キーボードだけで操作可能（Tab / 矢印 で選択切替、Enter で決定、Esc でキャンセル）
//! - v0 では SAVE は no-op（保存機能未実装）。将来の拡張ポイント。

use sdl2::keyboard::{Keycode, Mod};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;
use crate::ui;

/// ダイアログの選択肢。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DialogChoice {
    Discard,
    Save,
}

impl DialogChoice {
    fn label(self) -> &'static str {
        match self {
            DialogChoice::Discard => "[ DISCARD ]",
            DialogChoice::Save => "[ SAVE ]",
        }
    }
}

/// ダイアログの結果。
pub enum DialogResult {
    /// ユーザが選択したアクション
    Chosen(DialogChoice),
    /// Esc でキャンセル（編集に戻る）
    Cancel,
}

/// 終了確認ダイアログの状態。
pub struct ConfirmDialog {
    selected: DialogChoice,
    /// ダイアログ矩形（画面中央に配置）。draw 時に計算。
    rect: Rect,
}

impl ConfirmDialog {
    pub fn new() -> Self {
        ConfirmDialog {
            selected: DialogChoice::Discard,
            // 初期値はダミー。draw で画面サイズから計算する。
            rect: Rect::new(0, 0, 0, 0),
        }
    }

    /// キー入力。結果を返す。
    pub fn input_key(&mut self, key: Keycode, _mod: Mod) -> Option<DialogResult> {
        match key {
            Keycode::Escape => Some(DialogResult::Cancel),
            Keycode::Return | Keycode::KpEnter => Some(DialogResult::Chosen(self.selected)),
            // 選択切替: Tab / 左右
            Keycode::Tab | Keycode::Left | Keycode::Right => {
                self.selected = match self.selected {
                    DialogChoice::Discard => DialogChoice::Save,
                    DialogChoice::Save => DialogChoice::Discard,
                };
                None
            }
            _ => None,
        }
    }

    /// 描画。画面中央にダイアログを配置。
    /// 外側（暗転 etc）は呼び出し側で処理すること。
    pub fn draw(
        &mut self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        screen_w: u32,
        screen_h: u32,
    ) {
        let amber = theme.get_color("foreground");
        let black = theme.get_color("background");

        // ---- ダイアログサイズ計算 ----
        // タイトルバー + メッセージ1行 + 選択肢1行。
        let dlg_w = screen_w.saturating_sub(16).clamp(160, 220);
        let dlg_h = screen_h.saturating_sub(16).clamp(72, 80);
        let dlg_x = screen_w as i32 / 2 - dlg_w as i32 / 2;
        let dlg_y = screen_h as i32 / 2 - dlg_h as i32 / 2;
        self.rect = Rect::new(dlg_x, dlg_y, dlg_w, dlg_h);

        // 1) 外枠とタイトルバー
        ui::draw_rect_thick(c, self.rect, amber);
        ui::line(
            c,
            dlg_x + 1,
            dlg_y + 14,
            dlg_x + dlg_w as i32 - 2,
            dlg_y + 14,
            amber,
        );
        font.draw_text(c, dlg_x + 18, dlg_y + 1, "Confirm", amber);
        ui::draw_rect(c, Rect::new(dlg_x + 5, dlg_y + 4, 7, 7), amber);

        // 2) メッセージ（中央寄せ）
        let msg = "Quit TextEditor?";
        let msg_w = font.measure_text(msg);
        let msg_x = dlg_x + (dlg_w as i32 - msg_w) / 2;
        let msg_y = dlg_y + 24;
        font.draw_text(c, msg_x, msg_y, msg, amber);

        // 3) 選択肢ボタン（横並び、中央寄せ）
        let choices = [DialogChoice::Discard, DialogChoice::Save];
        // 各ラベルの幅を測って全体幅を計算
        let gap = if dlg_w < 190 { 10i32 } else { 24i32 };
        let widths: Vec<i32> = choices
            .iter()
            .map(|ch| font.measure_text(ch.label()))
            .collect();
        let total_w: i32 = widths.iter().sum::<i32>() + gap * (choices.len() as i32 - 1);
        let mut bx = dlg_x + (dlg_w as i32 - total_w) / 2;
        let by = dlg_y + (dlg_h as i32 - 22).max(42);

        for (i, ch) in choices.iter().enumerate() {
            let label = ch.label();
            let w = widths[i];
            // 選択中なら枠で囲む（outline）
            if *ch == self.selected {
                let box_rect = Rect::new(bx - 3, by - 2, w as u32 + 6, 15);
                // 選択枠は Amber。背景は黒（fill だが選択ハイライトは例外的に許可）
                let _ = c.set_draw_color(black);
                let _ = c.fill_rect(box_rect);
                ui::draw_rect(c, box_rect, amber);
            }
            font.draw_text(c, bx, by, label, amber);
            bx += w + gap;
        }
    }

    /// ダイアログ矩形（テスト/デバッグ用）。
    pub fn rect(&self) -> Rect {
        self.rect
    }
}
