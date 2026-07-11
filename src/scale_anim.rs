//! ScaleAnim - アプリ起動/終了時のスケールウィンドウエフェクト。
//!
//! 挙動:
//! - Opening: アイコン位置（小さい矩形）からアプリ領域（大きい矩形）へ拡大
//! - Closing: アプリ領域からアイコン位置へ縮小
//! - 終了時にコールバック（main 側で Mode 切替）で状態を確定
//!
//! 描画:
//! - 現在のスケール矩形を Amber の枠で描く（アプリの境界を視覚化）
//! - ease-out で自然な減速

use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::theme::ThemeManager;

/// アニメーションの方向。
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AnimDir {
    Opening,
    Closing,
}

/// アニメーション状態。
pub struct ScaleAnim {
    /// 開始矩形（アイコン位置）
    pub from: Rect,
    /// 終了矩形（アプリ領域）
    pub to: Rect,
    /// 進行度 0.0..1.0
    pub t: f32,
    /// 方向
    pub dir: AnimDir,
    /// 1フレームあたりの進行速度（0.0..1.0）。約0.12 で ~8フレーム。
    pub speed: f32,
}

impl ScaleAnim {
    pub fn new_opening(from: Rect, to: Rect) -> Self {
        ScaleAnim {
            from,
            to,
            t: 0.0,
            dir: AnimDir::Opening,
            speed: 0.14,
        }
    }

    pub fn new_closing(from: Rect, to: Rect) -> Self {
        ScaleAnim {
            from,
            to,
            t: 0.0,
            dir: AnimDir::Closing,
            speed: 0.18,
        }
    }

    /// 1フレーム進める。完了したら true。
    pub fn step(&mut self) -> bool {
        self.t += self.speed;
        if self.t >= 1.0 {
            self.t = 1.0;
            true
        } else {
            false
        }
    }

    /// 現在の矩形を計算（ease-out）。
    pub fn current_rect(&self) -> Rect {
        let e = ease_out(self.t);
        lerp_rect(&self.from, &self.to, e)
    }

    /// 現在の不透明度（0.0..1.0）。Closing では 1→0 へ。
    pub fn alpha(&self) -> f32 {
        match self.dir {
            AnimDir::Opening => ease_out(self.t),
            AnimDir::Closing => 1.0 - ease_out(self.t),
        }
    }

    /// 描画（Amber 枠のみ）。apps の実描画は main 側で行う想定だが、
    /// アニメ中は枠だけ描いて中身は隠す。
    pub fn draw(&self, c: &mut Canvas<Window>, theme: &ThemeManager) {
        let amber = theme.get_color("foreground");
        let r = self.current_rect();
        c.set_draw_color(amber);
        // 2px 枠
        let _ = c.draw_rect(r);
        let inner = Rect::new(
            r.x() + 1,
            r.y() + 1,
            r.width().saturating_sub(2),
            r.height().saturating_sub(2),
        );
        let _ = c.draw_rect(inner);
    }
}

/// ease-out: 1 - (1-t)^2
fn ease_out(t: f32) -> f32 {
    let u = 1.0 - t;
    1.0 - u * u
}

/// 矩形の線形補間。
fn lerp_rect(from: &Rect, to: &Rect, t: f32) -> Rect {
    let x = lerp_i32(from.x(), to.x(), t);
    let y = lerp_i32(from.y(), to.y(), t);
    let w = lerp_u32(from.width(), to.width(), t);
    let h = lerp_u32(from.height(), to.height(), t);
    Rect::new(x, y, w, h)
}

fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    let a = a as f32;
    let b = b as f32;
    (a + (b - a) * t).round() as i32
}

fn lerp_u32(a: u32, b: u32, t: f32) -> u32 {
    let a = a as f32;
    let b = b as f32;
    (a + (b - a) * t).max(0.0).round() as u32
}
