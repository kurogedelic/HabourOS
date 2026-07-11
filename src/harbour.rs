use chrono::{Local, Timelike};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::input::{InputState, KeyInput, PointerInput, PointerState};

/// ウィンドウの配置・サイズ・内容。Harbour OS デスクトップがリストで管理する。
#[derive(Clone)]
pub struct AppWindow {
    pub title: String,
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    /// タイトルバー高（Classic 準拠）。既定 18。
    pub titlebar_h: u32,
    /// アクティブ（最前面）かどうか。ベベル色に影響する。
    pub active: bool,
}

impl AppWindow {
    pub fn new(title: &str, x: i32, y: i32, w: u32, h: u32) -> Self {
        AppWindow {
            title: title.to_string(),
            x,
            y,
            w,
            h,
            titlebar_h: 18,
            active: false,
        }
    }

    /// クライアント領域（タイトルバー直下）の矩形。
    pub fn client_rect(&self) -> Rect {
        Rect::new(
            self.x + 1,
            self.y + self.titlebar_h as i32,
            self.w.saturating_sub(2),
            self.h.saturating_sub(self.titlebar_h + 1),
        )
    }

    /// タイトルバーの矩形。
    pub fn titlebar_rect(&self) -> Rect {
        Rect::new(self.x, self.y, self.w, self.titlebar_h)
    }

    /// クローズボックスの矩形（タイトルバー左端）。
    pub fn closebox_rect(&self) -> Rect {
        // タイトルバー垂直中央、左から3px
        let cb = 11i32;
        let cy = self.y + (self.titlebar_h as i32 - cb) / 2;
        Rect::new(self.x + 3, cy, cb as u32, cb as u32)
    }

    /// タイトルテキストの描画開始 X（closebox の右）。
    pub fn title_x(&self) -> i32 {
        self.x + 3 + 11 + 4
    }
}

/// HarbourOS - シンプルな仮想OSカーネル
pub struct HarbourOS {
    pub system_time: String,
    pub windows: Vec<AppWindow>,
    input: InputState,
}

impl HarbourOS {
    pub fn new() -> Self {
        Self::with_pointer(0, 0)
    }

    pub fn with_pointer(pointer_x: i32, pointer_y: i32) -> Self {
        let mut os = Self {
            system_time: String::new(),
            windows: Vec::new(),
            input: InputState::new(pointer_x, pointer_y),
        };
        os.update_time();
        os
    }

    pub fn update_time(&mut self) {
        let now = Local::now();
        self.system_time = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
    }

    pub fn input_key(&mut self, input: KeyInput) {
        self.input.record_key(input);
    }

    pub fn input_pointer(&mut self, input: PointerInput) {
        self.input.record_pointer(input);
    }

    pub fn clamp_pointer(&mut self, screen_w: u32, screen_h: u32) {
        self.input.pointer.clamp_to(screen_w, screen_h);
    }

    pub fn pointer(&self) -> &PointerState {
        &self.input.pointer
    }

    pub fn last_key(&self) -> Option<KeyInput> {
        self.input.last_key
    }

    pub fn last_pointer(&self) -> Option<PointerInput> {
        self.input.last_pointer
    }

    /// ウィンドウを追加し、最前面（アクティブ）にする。
    pub fn add_window(&mut self, mut win: AppWindow) {
        for w in self.windows.iter_mut() {
            w.active = false;
        }
        win.active = true;
        self.windows.push(win);
    }

    /// 指定インデックスのウィンドウを最前面（アクティブ）にする。
    pub fn focus_window(&mut self, idx: usize) {
        for (i, w) in self.windows.iter_mut().enumerate() {
            w.active = i == idx;
        }
    }

    /// トップバーの時計を描画。色は呼び出し側（ThemeManager 経由）で指定する。
    pub fn draw_topbar_clock(
        &self,
        c: &mut Canvas<Window>,
        font_system: &crate::font::FontSystem,
        clock_color: Color,
    ) {
        let bar_height = 20i32;
        let offset = 3i32; // 左端からのオフセット（1pxオフセット + 2px角丸）

        // 時計を左端に配置
        let clock_text = &self.system_time;
        let clock_x = offset;
        let clock_y = (bar_height - 13) / 2; // 13はフォント高さ

        // ボールドで描画
        font_system.draw_text_bold(c, clock_x, clock_y, clock_text, clock_color);

        // 時計の横に区切り線（時計と同色）
        let clock_width = font_system.measure_text(clock_text);
        let divider_x = clock_x + clock_width + 4; // 4px間隔
        c.set_draw_color(clock_color);
        let _ = c.fill_rect(Rect::new(divider_x, 3, 1, 14));
    }
}
