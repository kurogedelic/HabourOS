//! FileBrowser - 仮想ファイルシステムを閲覧する組込アプリ。
//!
//! 操作（キーボードのみ）:
//! - ↑/↓: カーソル移動
//! - Enter: ディレクトリなら进入、ファイルなら「開く」要求を発行
//! - Backspace または .. 選択: 親ディレクトリへ
//! - D: 選択中アイテムを削除（Enter で確定 / Esc でキャンセル）
//! - N: 新規ファイル作成（空ファイルを生成し、そのファイルを開く要求）
//! - Esc: 終了（デスクトップへ）
//!
//! 描画:
//! - 左にパス表示、中央に一覧（Amber outline）
//! - ディレクトリは `/` 付きで表示

use sdl2::keyboard::{Keycode, Mod};
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::app::{App, AppResult};
use crate::assets::AssetStore;
use crate::font::FontSystem;
use crate::input::{PointerAction, PointerButton, PointerInput};
use crate::theme::ThemeManager;
use crate::ui;
use crate::vfs::{Vfs, VfsError};

const CELL_W: i32 = 6;
const CELL_H: i32 = 13;
const PADDING: i32 = 6;
/// 「..」の表示用のダミーエントリインデックス。
const PARENT_MARKER: usize = usize::MAX;

/// ファイルブラウザの「外への要求」。
/// main.rs がこれを受け取って TextEditor へ遷移する等の協調を行う。
pub enum BrowserAction {
    /// 指定パスのファイルを TextEditor で開く。
    OpenFile(String),
    /// 指定パスをホスト側 export ディレクトリへ書き出す。
    Export(String),
    /// ホスト側 import ディレクトリを現在の VFS ディレクトリへ取り込む。
    ImportInto(String),
    /// 何もしない（継続）。
    None,
}

pub struct FileBrowser {
    /// 現在の絶対パス。
    cwd: String,
    /// カーソル位置（0 = 「..」、1.. = エントリ）。
    cursor: usize,
    /// 一覧キャッシュ。
    entries: Vec<(String, bool)>,
    /// 最後のアクション（draw 後に main が取り出す）。
    pending: BrowserAction,
    /// 最後のメッセージ（ステータス行表示用）。
    message: String,
    /// VFS を変更したかどうか。
    dirty: bool,
    /// 現在の表示列数。draw 時に画面幅から更新する。
    view_cols: usize,
    /// リネーム中の対象パス。
    rename_target: Option<String>,
    /// リネーム入力バッファ。
    rename_buffer: String,
    /// リネーム開始キー由来の TextInput を1回だけ抑制する。
    suppress_next_text: bool,
    /// 削除確認中の対象パス。
    delete_target: Option<String>,
    /// 削除確認中の表示名。
    delete_name: String,
}

impl FileBrowser {
    pub fn new(vfs: &Vfs) -> Self {
        let mut br = FileBrowser {
            cwd: "/".to_string(),
            cursor: 0,
            entries: Vec::new(),
            pending: BrowserAction::None,
            message: String::new(),
            dirty: false,
            view_cols: 4,
            rename_target: None,
            rename_buffer: String::new(),
            suppress_next_text: false,
            delete_target: None,
            delete_name: String::new(),
        };
        br.refresh(vfs);
        br
    }

    /// 一覧を再読込。
    fn refresh(&mut self, vfs: &Vfs) {
        self.entries = vfs.list_dir(&self.cwd).unwrap_or_default();
        // カーソル範囲内にクランプ（+1 は「..」分）
        let max = self.entries.len();
        if self.cursor > max {
            self.cursor = max;
        }
    }

    /// 保留中アクションを取り出す。
    pub fn take_action(&mut self) -> BrowserAction {
        std::mem::replace(&mut self.pending, BrowserAction::None)
    }

    /// VFS 変更フラグを取り出す。
    pub fn take_dirty(&mut self) -> bool {
        let dirty = self.dirty;
        self.dirty = false;
        dirty
    }

    /// 最後のメッセージ。
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn footer_text(&self) -> String {
        if self.rename_target.is_some() {
            format!("Rename: {}", self.rename_buffer)
        } else if self.delete_target.is_some() {
            format!("Delete {}? Enter confirm / Esc cancel", self.delete_name)
        } else {
            self.message.clone()
        }
    }

    pub fn has_modal_input(&self) -> bool {
        self.rename_target.is_some() || self.delete_target.is_some()
    }

    pub fn set_message(&mut self, message: impl Into<String>) {
        self.message = message.into();
    }

    pub fn refresh_public(&mut self, vfs: &Vfs) {
        self.refresh(vfs);
    }

    pub fn input_char(&mut self, ch: char) {
        if self.rename_target.is_some() && !ch.is_control() && ch != '/' && ch != '\\' {
            if self.suppress_next_text {
                self.suppress_next_text = false;
                return;
            }
            self.rename_buffer.push(ch);
        }
    }

    /// VFS を渡してキー処理。ブラウザ自体の終了時は Quit。
    pub fn handle_key_public(&mut self, key: Keycode, keymod: Mod, vfs: &mut Vfs) -> AppResult {
        self.handle_key(key, keymod, vfs)
    }

    pub fn handle_pointer_public(
        &mut self,
        input: PointerInput,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        vfs: &mut Vfs,
    ) -> AppResult {
        self.handle_pointer(input, x, y, w, h, vfs)
    }

    /// VFS を渡してキー処理。ブラウザ自体の終了時は Quit。
    fn handle_key(&mut self, key: Keycode, keymod: Mod, vfs: &mut Vfs) -> AppResult {
        if self.delete_target.is_some() {
            return self.handle_delete_key(key, vfs);
        }
        if self.rename_target.is_some() {
            return self.handle_rename_key(key, vfs);
        }
        match key {
            Keycode::Escape => return AppResult::Quit,

            Keycode::Up => {
                let cols = self.view_cols.max(1);
                if self.cursor >= cols {
                    self.cursor -= cols;
                }
            }
            Keycode::Down => {
                let cols = self.view_cols.max(1);
                let max = self.entries.len();
                if self.cursor + cols <= max {
                    self.cursor += cols;
                } else {
                    self.cursor = max;
                }
            }
            Keycode::Left => {
                if self.cursor > 0 {
                    self.cursor -= 1;
                }
            }
            Keycode::Right => {
                let max = self.entries.len();
                if self.cursor < max {
                    self.cursor += 1;
                }
            }
            Keycode::Home => self.cursor = 0,
            Keycode::End => self.cursor = self.entries.len(),

            Keycode::Return | Keycode::KpEnter => {
                if self.cursor == PARENT_MARKER || self.cursor == 0 {
                    // 「..」で親へ
                    self.go_parent();
                    self.refresh(vfs);
                } else {
                    let idx = self.cursor - 1;
                    if let Some((name, is_dir)) = self.entries.get(idx).cloned() {
                        let full = join_path(&self.cwd, &name);
                        if is_dir {
                            self.cwd = full;
                            self.cursor = 0;
                            self.refresh(vfs);
                        } else {
                            // ファイルを開く要求
                            self.pending = BrowserAction::OpenFile(full);
                            return AppResult::Quit;
                        }
                    }
                }
            }

            Keycode::Backspace => {
                self.go_parent();
                self.refresh(vfs);
            }

            Keycode::D => {
                self.start_delete();
            }

            Keycode::N => {
                if is_shift(keymod) {
                    self.create_directory(vfs);
                } else {
                    return self.create_file(vfs);
                }
            }

            Keycode::E => {
                if let Some(path) = self.selected_path() {
                    self.pending = BrowserAction::Export(path);
                } else {
                    self.message = "select an item to export".to_string();
                }
            }

            Keycode::I => {
                self.pending = BrowserAction::ImportInto(self.cwd.clone());
            }

            Keycode::R => {
                self.start_rename();
            }

            _ => {}
        }
        AppResult::Continue
    }

    fn handle_pointer(
        &mut self,
        input: PointerInput,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        vfs: &mut Vfs,
    ) -> AppResult {
        if self.has_modal_input() {
            return AppResult::Continue;
        }

        if input.action == PointerAction::Wheel {
            let cols = self.view_cols.max(1);
            if input.wheel_y > 0 {
                self.cursor = self.cursor.saturating_sub(cols);
            } else if input.wheel_y < 0 {
                self.cursor = (self.cursor + cols).min(self.entries.len());
            }
            return AppResult::Continue;
        }

        let layout = BrowserLayout::new(x, y, w, h, self.cursor);
        if let Some(idx) = layout.hit_entry(input.x, input.y, self.entries.len() + 1) {
            self.cursor = idx;
            if input.action == PointerAction::ButtonUp
                && input.button == Some(PointerButton::Left)
                && input.clicks >= 2
            {
                return self.handle_key(Keycode::Return, Mod::NOMOD, vfs);
            }
        }
        AppResult::Continue
    }

    fn handle_delete_key(&mut self, key: Keycode, vfs: &mut Vfs) -> AppResult {
        match key {
            Keycode::Escape => {
                self.delete_target = None;
                self.delete_name.clear();
                self.message = "delete cancelled".to_string();
            }
            Keycode::Return | Keycode::KpEnter => {
                let Some(path) = self.delete_target.take() else {
                    return AppResult::Continue;
                };
                let name = std::mem::take(&mut self.delete_name);
                match vfs.remove(&path) {
                    Ok(()) => {
                        self.message = format!("deleted {}", name);
                        self.dirty = true;
                        self.refresh(vfs);
                    }
                    Err(e) => self.message = format!("delete error: {}", e),
                }
            }
            _ => {}
        }
        AppResult::Continue
    }

    fn handle_rename_key(&mut self, key: Keycode, vfs: &mut Vfs) -> AppResult {
        match key {
            Keycode::Escape => {
                self.rename_target = None;
                self.rename_buffer.clear();
                self.suppress_next_text = false;
                self.message = "rename cancelled".to_string();
            }
            Keycode::Backspace => {
                self.rename_buffer.pop();
            }
            Keycode::Return | Keycode::KpEnter => {
                let Some(path) = self.rename_target.take() else {
                    return AppResult::Continue;
                };
                let new_name = self.rename_buffer.trim().to_string();
                self.rename_buffer.clear();
                self.suppress_next_text = false;
                if new_name.is_empty() {
                    self.message = "rename needs a name".to_string();
                    return AppResult::Continue;
                }
                match vfs.rename(&path, &new_name) {
                    Ok(new_path) => {
                        self.message = format!("renamed to {}", new_name);
                        self.dirty = true;
                        self.refresh(vfs);
                        self.select_path(&new_path);
                    }
                    Err(e) => self.message = format!("rename error: {}", e),
                }
            }
            _ => {}
        }
        AppResult::Continue
    }

    fn start_rename(&mut self) {
        if self.cursor == 0 {
            self.message = "cannot rename parent".to_string();
            return;
        }
        let idx = self.cursor - 1;
        let Some((name, _)) = self.entries.get(idx) else {
            return;
        };
        self.rename_target = Some(join_path(&self.cwd, name));
        self.rename_buffer = name.clone();
        self.suppress_next_text = true;
        self.message = "rename: type name, Enter to apply".to_string();
    }

    fn start_delete(&mut self) {
        if self.cursor == 0 {
            self.message = "cannot delete parent".to_string();
            return;
        }
        let idx = self.cursor - 1;
        let Some((name, _)) = self.entries.get(idx) else {
            return;
        };
        self.delete_target = Some(join_path(&self.cwd, name));
        self.delete_name = name.clone();
        self.message = "delete: Enter confirm / Esc cancel".to_string();
    }

    fn create_file(&mut self, vfs: &mut Vfs) -> AppResult {
        let mut i = 0;
        loop {
            let name = if i == 0 {
                "Untitled.txt".to_string()
            } else {
                format!("Untitled{}.txt", i)
            };
            let full = join_path(&self.cwd, &name);
            match vfs.create_file(&full, "") {
                Ok(()) => {
                    self.message = format!("created {}", name);
                    self.dirty = true;
                    self.pending = BrowserAction::OpenFile(full);
                    return AppResult::Quit;
                }
                Err(VfsError::AlreadyExists) => {
                    i += 1;
                    continue;
                }
                Err(e) => {
                    self.message = format!("error: {}", e);
                    break;
                }
            }
        }
        AppResult::Continue
    }

    fn create_directory(&mut self, vfs: &mut Vfs) {
        let mut i = 0;
        loop {
            let name = if i == 0 {
                "NewFolder".to_string()
            } else {
                format!("NewFolder{}", i)
            };
            let full = join_path(&self.cwd, &name);
            match vfs.create_dir(&full) {
                Ok(()) => {
                    self.message = format!("created {}/", name);
                    self.dirty = true;
                    self.refresh(vfs);
                    break;
                }
                Err(VfsError::AlreadyExists) => {
                    i += 1;
                    continue;
                }
                Err(e) => {
                    self.message = format!("error: {}", e);
                    break;
                }
            }
        }
    }

    fn selected_path(&self) -> Option<String> {
        if self.cursor == 0 {
            return None;
        }
        let idx = self.cursor - 1;
        let (name, _) = self.entries.get(idx)?;
        Some(join_path(&self.cwd, name))
    }

    fn select_path(&mut self, path: &str) {
        let Some(name) = path.rsplit('/').next() else {
            return;
        };
        if let Some(idx) = self.entries.iter().position(|(entry, _)| entry == name) {
            self.cursor = idx + 1;
        }
    }

    /// 親ディレクトリへ移動。ルートなら何もしない。
    fn go_parent(&mut self) {
        if self.cwd == "/" {
            return;
        }
        // 最後の `/xxx` を除去
        let trimmed = self.cwd.trim_end_matches('/');
        if let Some(pos) = trimmed.rfind('/') {
            self.cwd = if pos == 0 {
                "/".to_string()
            } else {
                trimmed[..pos].to_string()
            };
        } else {
            self.cwd = "/".to_string();
        }
        self.cursor = 0;
        // 注意: refresh は呼び出し側で VFS を渡して行う必要があるが、
        // go_parent は handle_key 内から呼ばれ、そこで後続の refresh がない。
        // そのため entries は古いままになる問題がある → handle_key 側で対応。
    }

    /// VFS と AssetStore を渡してアイコングリッド描画。
    pub fn draw_with_vfs(
        &mut self,
        c: &mut Canvas<Window>,
        assets: &mut AssetStore,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
        vfs: &Vfs,
    ) {
        let amber = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let background = theme.get_color("background");

        // 1) ヘッダ: 現在パス
        let path_label = format!("Path: {}", self.cwd);
        let path_label = fit_label(font, &path_label, w as i32 - PADDING * 2);
        font.draw_text(c, x + PADDING, y + 2, &path_label, amber);
        // ヘッダ下の区切り線
        ui::line(c, x + 1, y + 14, x + w as i32 - 2, y + 14, amber);

        // 2) アイコングリッド
        let icon_size = 32u32;
        let cell_w = 72i32;
        // ヘッダ下 + 余白
        let cols = ((w as i32 - 16) / cell_w).max(1) as usize;
        self.view_cols = cols;

        // 表示対象: インデックス 0 = 「..」、1.. = entries
        let total = self.entries.len() + 1;

        // スクロール（簡易: 表示開始行を計算）
        let layout = BrowserLayout::new(x, y, w, h, self.cursor);
        let visible = layout.visible;
        let scroll = layout.scroll;

        for i in 0..visible {
            let idx = scroll + i;
            if idx >= total {
                break;
            }
            let (cx, cy) = layout.cell_origin(i);
            let is_cursor = idx == self.cursor;

            // エントリ情報
            let (name, is_dir): (String, bool) = if idx == 0 {
                ("..".to_string(), true)
            } else {
                let (n, d) = &self.entries[idx - 1];
                (n.clone(), *d)
            };

            // アイコン描画
            let icon_dst = Rect::new(
                cx + (cell_w - icon_size as i32) / 2,
                cy,
                icon_size,
                icon_size,
            );
            let icon_key = if idx == 0 {
                "icons/Folder"
            } else if is_dir {
                "icons/Folder"
            } else {
                "icons/File"
            };

            if is_cursor {
                // 選択中: Amber 背景で反転（黒で染色して見せる）
                c.set_draw_color(muted);
                let _ = c.fill_rect(Rect::new(
                    icon_dst.x() - 2,
                    icon_dst.y() - 2,
                    icon_dst.width() + 4,
                    icon_dst.height() + 4,
                ));
                let _ = assets.blit_tinted(c, icon_key, Some(icon_dst), background);
            } else {
                let _ = assets.blit_tinted(c, icon_key, Some(icon_dst), amber);
            }

            // ラベル
            let label = if is_dir && idx != 0 {
                format!("{}/", name)
            } else {
                name.clone()
            };
            let label = fit_label(font, &label, cell_w - 4);
            let label_w = font.measure_text(&label);
            let lx = cx + (cell_w - label_w) / 2;
            let ly = cy + icon_size as i32 + 2;
            font.draw_text(c, lx, ly, &label, amber);
            if is_cursor {
                ui::line(c, lx, ly + 13, lx + label_w, ly + 13, amber);
            }
        }

        // 未使用警告抑制
        let _ = vfs;
    }
}

struct BrowserLayout {
    grid_x: i32,
    grid_y: i32,
    cell_w: i32,
    cell_h: i32,
    cols: usize,
    visible: usize,
    scroll: usize,
}

impl BrowserLayout {
    fn new(x: i32, y: i32, w: u32, h: u32, cursor: usize) -> Self {
        let cell_w = 72i32;
        let cell_h = 52i32;
        let cols = ((w as i32 - 16) / cell_w).max(1) as usize;
        let visible_rows = ((h as i32 - 20 - 8) / cell_h).max(1) as usize;
        let visible = visible_rows * cols;
        let scroll = if cursor >= visible {
            (cursor - visible + cols) / cols * cols
        } else {
            0
        };

        Self {
            grid_x: x + 8,
            grid_y: y + 20,
            cell_w,
            cell_h,
            cols,
            visible,
            scroll,
        }
    }

    fn cell_origin(&self, visible_idx: usize) -> (i32, i32) {
        let col = visible_idx % self.cols;
        let row = visible_idx / self.cols;
        (
            self.grid_x + col as i32 * self.cell_w,
            self.grid_y + row as i32 * self.cell_h,
        )
    }

    fn hit_entry(&self, px: i32, py: i32, total: usize) -> Option<usize> {
        for visible_idx in 0..self.visible {
            let idx = self.scroll + visible_idx;
            if idx >= total {
                break;
            }
            let (cx, cy) = self.cell_origin(visible_idx);
            let hit = Rect::new(cx, cy, self.cell_w as u32, self.cell_h as u32);
            if hit.contains_point((px, py)) {
                return Some(idx);
            }
        }
        None
    }
}

/// パス結合。
fn join_path(cwd: &str, name: &str) -> String {
    if cwd == "/" {
        format!("/{}", name)
    } else {
        format!("{}/{}", cwd, name)
    }
}

fn fit_label(font: &FontSystem, label: &str, max_w: i32) -> String {
    if font.measure_text(label) <= max_w {
        return label.to_string();
    }
    let mut out = String::new();
    for ch in label.chars() {
        let candidate = format!("{}{}...", out, ch);
        if font.measure_text(&candidate) > max_w {
            break;
        }
        out.push(ch);
    }
    if out.is_empty() {
        "...".to_string()
    } else {
        format!("{}...", out)
    }
}

fn is_shift(keymod: Mod) -> bool {
    keymod & Mod::LSHIFTMOD != Mod::NOMOD || keymod & Mod::RSHIFTMOD != Mod::NOMOD
}

// App trait 実装は VFS アクセスを要するため、main.rs 側で直接呼び出す設計。
// （App trait は VFS を持たないため、ブラウザは trait を経由せず直接使う）
impl App for FileBrowser {
    fn name(&self) -> &str {
        "FileBrowser"
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
        // VFS 無し版: 一覧キャッシュだけで描画
        let amber = theme.get_color("foreground");
        c.set_draw_color(amber);
        let _ = c.draw_rect(Rect::new(x, y, w, h));
        let path_label = format!(" {} ", self.cwd);
        font.draw_text(c, x + PADDING, y + 4, &path_label, amber);
        ui::line(c, x, y + 18, x + w as i32 - 1, y + 18, amber);
        let _ = (CELL_W, CELL_H);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_on_file_requests_open_file() {
        let mut vfs = Vfs::new();
        vfs.create_file("/a.txt", "hello").unwrap();
        let mut browser = FileBrowser::new(&vfs);
        browser.cursor = 1;

        let result = browser.handle_key_public(Keycode::Return, Mod::NOMOD, &mut vfs);
        assert!(matches!(result, AppResult::Quit));
        match browser.take_action() {
            BrowserAction::OpenFile(path) => assert_eq!(path, "/a.txt"),
            _ => panic!("expected OpenFile action"),
        }
    }

    #[test]
    fn delete_requires_confirmation() {
        let mut vfs = Vfs::new();
        vfs.create_file("/a.txt", "hello").unwrap();
        let mut browser = FileBrowser::new(&vfs);
        browser.cursor = 1;

        let _ = browser.handle_key_public(Keycode::D, Mod::NOMOD, &mut vfs);
        assert!(browser.has_modal_input());
        assert!(vfs.read_file("/a.txt").is_ok());

        let _ = browser.handle_key_public(Keycode::Return, Mod::NOMOD, &mut vfs);
        assert!(vfs.read_file("/a.txt").is_err());
        assert!(browser.take_dirty());
    }

    #[test]
    fn delete_can_be_cancelled() {
        let mut vfs = Vfs::new();
        vfs.create_file("/a.txt", "hello").unwrap();
        let mut browser = FileBrowser::new(&vfs);
        browser.cursor = 1;

        let _ = browser.handle_key_public(Keycode::D, Mod::NOMOD, &mut vfs);
        let _ = browser.handle_key_public(Keycode::Escape, Mod::NOMOD, &mut vfs);
        assert!(vfs.read_file("/a.txt").is_ok());
        assert!(!browser.has_modal_input());
    }
}
