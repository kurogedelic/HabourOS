//! App - 組込アプリ共通インターフェース。
//!
//! Harbour OS デスクトップはこのトレイトを通じて任意のアプリを起動・描画する。
//! すべてのアプリはキーボード操作を基本にしつつ、必要に応じてポインタ入力も受ける。

use sdl2::keyboard::{Keycode, Mod};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::input::PointerInput;
use crate::theme::ThemeManager;

/// アプリの実行結果。Quit で Harbour OS デスクトップへ制御を戻す。
pub enum AppResult {
    /// 継続実行
    Continue,
    /// アプリ終了（デスクトップ/ランチャーへ戻る）
    Quit,
}

/// すべての組込アプリが実装するトレイト。
pub trait App {
    /// アプリ名（ランチャー表示用）。
    fn name(&self) -> &str;

    /// フレーム毎の更新。描画前に呼ばれる。
    fn update(&mut self) {}

    /// 描画。与えられた矩形内に描くこと。
    fn draw(
        &mut self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        h: u32,
    );

    /// キー入力（機能キー・矢印など）。Quit で終了。
    fn input_key(&mut self, key: Keycode, modifiers: Mod) -> AppResult {
        let _ = (key, modifiers);
        AppResult::Continue
    }

    /// テキスト入力（印字可能文字）。
    fn input_char(&mut self, ch: char) {
        let _ = ch;
    }

    /// ポインタ入力（移動・クリック・ホイールなど）。
    fn input_pointer(&mut self, input: PointerInput) -> AppResult {
        let _ = input;
        AppResult::Continue
    }

    /// フォーカス取得時（起動時・復帰時）。
    fn on_focus(&mut self) {}

    /// フォーカス喪失時（終了時・バックグラウンドへ）。
    fn on_blur(&mut self) {}
}
