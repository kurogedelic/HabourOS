//! TextEditor v0 - 最初の実用アプリ。
//!
//! 要件:
//! - テキスト入力 / カーソル移動 / 改行 / Backspace / スクロール
//! - しない: 保存・開く・検索・Undo・ファイルシステム連携
//!
//! 方針:
//! - 単色（Black 背景 + Amber 文字/ボーダー/カーソル）
//! - fill は使わず、outline とテキスト描画のみ
//! - キーボードだけで完結
//!
//! データモデル:
//! - 行の Vec（各行は String）。空ドキュメントは [String::new()] 1行。
//! - カーソルは (row, col)。常に 0..=len の範囲。

use sdl2::keyboard::{Keycode, Mod};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;

/// グリッドサイズ（M+ F12R: 6x13）
const CELL_W: i32 = 6;
const CELL_H: i32 = 13;
/// エディタ領域の内側余白
const PADDING: i32 = 6;
const MAX_UNDO: usize = 64;

#[derive(Clone)]
struct EditorSnapshot {
    lines: Vec<String>,
    row: usize,
    col: usize,
    scroll: usize,
    dirty: bool,
}

pub struct TextEditor {
    /// 行データ。必ず1行以上。
    lines: Vec<String>,
    /// カーソル位置 (row, col)
    row: usize,
    col: usize,
    /// 縦スクロール量（表示開始行）
    scroll: usize,
    /// 表示領域サイズ（draw 時に更新）
    view_w: u32,
    view_h: u32,
    /// 保存後に編集されたかどうか。
    dirty: bool,
    undo_stack: Vec<EditorSnapshot>,
}

impl TextEditor {
    pub fn new() -> Self {
        TextEditor {
            lines: vec![String::new()],
            row: 0,
            col: 0,
            scroll: 0,
            view_w: 0,
            view_h: 0,
            dirty: false,
            undo_stack: Vec::new(),
        }
    }

    /// ウェルカムメッセージで初期化。
    pub fn with_welcome() -> Self {
        let mut ed = Self::new();
        ed.lines = vec![
            "# Harbour TextEditor".to_string(),
            "".to_string(),
            "Type to begin. Enter = new line, Backspace = delete.".to_string(),
            "Ctrl+S saves. Arrows move. Esc quits.".to_string(),
            "".to_string(),
        ];
        // 最後の行（空行）の先頭にカーソル
        ed.row = ed.lines.len() - 1;
        ed.col = 0;
        ed
    }

    /// 既存テキストから生成（ファイルを開く時）。
    pub fn from_text(content: &str) -> Self {
        let lines: Vec<String> = if content.is_empty() {
            vec![String::new()]
        } else {
            content.split('\n').map(|s| s.to_string()).collect()
        };
        let row = lines.len() - 1;
        TextEditor {
            lines,
            row,
            col: 0,
            scroll: 0,
            view_w: 0,
            view_h: 0,
            dirty: false,
            undo_stack: Vec::new(),
        }
    }

    /// 全テキストを結合して返す（保存用）。
    pub fn text_for_save(&self) -> String {
        self.lines.join("\n")
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.lines = snapshot.lines;
            self.row = snapshot.row.min(self.lines.len().saturating_sub(1));
            self.col = snapshot.col.min(self.lines[self.row].len());
            self.scroll = snapshot.scroll;
            self.dirty = snapshot.dirty;
            self.clamp_scroll();
        }
    }

    /// 表示領域サイズを更新（描画前に呼ぶ）。
    pub fn set_view(&mut self, w: u32, h: u32) {
        self.view_w = w;
        self.view_h = h;
        self.clamp_scroll();
    }

    // ---- 入力 ----------------------------------------------------------

    /// SDL2 のテキスト入力イベント（Unicode 文字）を処理。
    pub fn input_char(&mut self, ch: char) {
        // 制御文字は無視（改行は別途 input_key で処理）
        if ch.is_control() {
            return;
        }
        self.push_undo();
        // そのまま挿入。描画は FontSystem が Latin-1 / JIS フォントで
        // フォールバック描画する（日本語対応）。
        let line = &mut self.lines[self.row];
        line.insert(self.col, ch);
        self.col += 1;
        self.dirty = true;
    }

    /// キー入力を処理。終了すべき場合は true を返す（Esc）。
    pub fn input_key(&mut self, key: Keycode, keymod: Mod) -> bool {
        match key {
            Keycode::Escape => return true,

            Keycode::Z if is_ctrl(keymod) => {
                self.undo();
            }

            Keycode::Return | Keycode::KpEnter => {
                self.push_undo();
                // 現在行をカーソル位置で分割
                let after: String = self.lines[self.row].split_off(self.col);
                self.lines.insert(self.row + 1, after);
                self.row += 1;
                self.col = 0;
                self.dirty = true;
            }

            Keycode::Backspace => {
                if self.col > 0 {
                    self.push_undo();
                    self.lines[self.row].remove(self.col - 1);
                    self.col -= 1;
                    self.dirty = true;
                } else if self.row > 0 {
                    self.push_undo();
                    // 前行の末尾に連結
                    let cur = self.lines.remove(self.row);
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                    self.lines[self.row].push_str(&cur);
                    self.dirty = true;
                }
            }

            Keycode::Delete => {
                if self.col < self.lines[self.row].len() {
                    self.push_undo();
                    self.lines[self.row].remove(self.col);
                    self.dirty = true;
                } else if self.row + 1 < self.lines.len() {
                    self.push_undo();
                    let next = self.lines.remove(self.row + 1);
                    self.lines[self.row].push_str(&next);
                    self.dirty = true;
                }
            }

            Keycode::Left => {
                if self.col > 0 {
                    self.col -= 1;
                } else if self.row > 0 {
                    self.row -= 1;
                    self.col = self.lines[self.row].len();
                }
            }
            Keycode::Right => {
                if self.col < self.lines[self.row].len() {
                    self.col += 1;
                } else if self.row + 1 < self.lines.len() {
                    self.row += 1;
                    self.col = 0;
                }
            }
            Keycode::Up => {
                if self.row > 0 {
                    self.row -= 1;
                    self.col = self.col.min(self.lines[self.row].len());
                }
            }
            Keycode::Down => {
                if self.row + 1 < self.lines.len() {
                    self.row += 1;
                    self.col = self.col.min(self.lines[self.row].len());
                }
            }

            Keycode::Home => self.col = 0,
            Keycode::End => self.col = self.lines[self.row].len(),

            Keycode::PageUp => {
                let visible = self.visible_rows();
                self.row = self.row.saturating_sub(visible);
                self.col = self.col.min(self.lines[self.row].len());
            }
            Keycode::PageDown => {
                let visible = self.visible_rows();
                self.row = (self.row + visible).min(self.lines.len() - 1);
                self.col = self.col.min(self.lines[self.row].len());
            }

            // タブはスペース2個として扱う（インデント）
            Keycode::Tab => {
                self.push_undo();
                let line = &mut self.lines[self.row];
                line.insert(self.col, ' ');
                line.insert(self.col + 1, ' ');
                self.col += 2;
                self.dirty = true;
            }

            _ => {}
        }

        self.clamp_scroll();
        false
    }

    fn push_undo(&mut self) {
        if self.undo_stack.len() >= MAX_UNDO {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(EditorSnapshot {
            lines: self.lines.clone(),
            row: self.row,
            col: self.col,
            scroll: self.scroll,
            dirty: self.dirty,
        });
    }

    /// 表示可能な行数を計算。
    fn visible_rows(&self) -> usize {
        let inner = self.view_h as i32 - PADDING * 2;
        if inner <= 0 {
            return 1;
        }
        (inner / CELL_H) as usize
    }

    /// カーソルが表示範囲に入るようスクロール量を調整。
    fn clamp_scroll(&mut self) {
        let visible = self.visible_rows();
        if visible == 0 {
            return;
        }
        if self.row < self.scroll {
            self.scroll = self.row;
        }
        if self.row >= self.scroll + visible {
            self.scroll = self.row + 1 - visible;
        }
    }

    // ---- 描画 ----------------------------------------------------------

    /// エディタを描画。枠 + テキスト + カーソル。
    pub fn draw(
        &self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) {
        let amber = theme.get_color("foreground");

        // 1) テキスト領域。外枠はアプリウィンドウ側が描画する。
        let tx0 = x + PADDING;
        let ty0 = y + PADDING;

        // 2) テキスト描画（スクロール考慮）
        let visible = ((h as i32 - PADDING * 2) / CELL_H).max(1) as usize;
        let max_col = ((w as i32 - PADDING * 2) / CELL_W).max(1) as usize;

        for (i, line) in self
            .lines
            .iter()
            .skip(self.scroll)
            .take(visible)
            .enumerate()
        {
            let ly = ty0 + i as i32 * CELL_H;
            // 表示幅に収まる分だけ描画
            let slice: String = line.chars().take(max_col).collect();
            font.draw_text(c, tx0, ly, &slice, amber);
        }

        // 3) カーソル（Amber のブロック、fill はカーソルのみ許可）
        let cur_row = self.row;
        let cur_col = self.col;
        if cur_row >= self.scroll && cur_row < self.scroll + visible {
            let cy = ty0 + (cur_row - self.scroll) as i32 * CELL_H;
            // 行頭より前なら x=tx0、超過なら直前までの文字幅
            let prefix: String = self.lines[cur_row].chars().take(cur_col).collect();
            let cx = tx0 + font.measure_text(&prefix);
            // 反転カーソル風: 黒背景に Amber 枠の空枠、または Amber 1px 縦線
            // → CRT 風に「縦線」を採用
            c.set_draw_color(amber);
            let _ = c.fill_rect(Rect::new(cx, cy, CELL_W as u32, CELL_H as u32 - 2));
        }
    }

    // ---- テスト用アクセサ ----------------------------------------------
    #[doc(hidden)]
    pub fn lines_for_test(&self) -> &[String] {
        &self.lines
    }
    #[doc(hidden)]
    pub fn row_for_test(&self) -> usize {
        self.row
    }
    #[doc(hidden)]
    pub fn col_for_test(&self) -> usize {
        self.col
    }
    #[doc(hidden)]
    pub fn scroll_for_test(&self) -> usize {
        self.scroll
    }
}

// ---- App トレイト実装 ------------------------------------------------
use crate::app::{App, AppResult};

impl App for TextEditor {
    fn name(&self) -> &str {
        "TextEditor"
    }

    fn draw(
        &mut self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    ) {
        self.set_view(w, h);
        TextEditor::draw(self, c, font, theme, x, y, w, h);
    }

    fn input_key(&mut self, key: Keycode, modifiers: Mod) -> AppResult {
        // 既存ロジック（bool を返す）を AppResult に変換
        let quit = TextEditor::input_key(self, key, modifiers);
        if quit {
            AppResult::Quit
        } else {
            AppResult::Continue
        }
    }

    fn input_char(&mut self, ch: char) {
        TextEditor::input_char(self, ch);
    }
}

fn is_ctrl(keymod: Mod) -> bool {
    keymod & Mod::LCTRLMOD != Mod::NOMOD || keymod & Mod::RCTRLMOD != Mod::NOMOD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_restores_previous_text() {
        let mut editor = TextEditor::new();
        editor.input_char('a');
        editor.input_char('b');
        assert_eq!(editor.text_for_save(), "ab");
        editor.undo();
        assert_eq!(editor.text_for_save(), "a");
    }
}
