//! Clock - 現在時刻を表示する最小アプリ。

use chrono::{Datelike, Local, Timelike};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;

pub struct ClockState;

impl ClockState {
    pub fn new() -> Self {
        ClockState
    }

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
        let muted = theme.get_color("muted");
        let now = Local::now();

        let time = format!("{:02}:{:02}:{:02}", now.hour(), now.minute(), now.second());
        let date = format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day());
        let weekday = now.format("%A").to_string();

        let center_x = x + w as i32 / 2;
        let center_y = y + h as i32 / 2;

        draw_large_digits(c, font, &time, center_x, center_y - 36, amber);

        let date_w = font.measure_text(&date);
        font.draw_text(c, center_x - date_w / 2, center_y + 18, &date, amber);

        let weekday_w = font.measure_text(&weekday);
        font.draw_text(c, center_x - weekday_w / 2, center_y + 34, &weekday, muted);
    }
}

fn draw_large_digits(
    c: &mut Canvas<Window>,
    font: &FontSystem,
    text: &str,
    center_x: i32,
    y: i32,
    color: sdl2::pixels::Color,
) {
    let scale = 2;
    let char_w = 6 * scale;
    let total_w = text.chars().count() as i32 * char_w;
    let mut x = center_x - total_w / 2;
    for ch in text.chars() {
        let s = ch.to_string();
        for oy in 0..scale {
            for ox in 0..scale {
                font.draw_text(c, x + ox, y + oy, &s, color);
            }
        }
        x += char_w;
    }
}
