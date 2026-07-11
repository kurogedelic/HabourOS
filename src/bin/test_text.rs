//! test_text - フォントレンダリングの検証用スタンドアロン。
//! 黒背景に各テーマ色でサンプル文字列を描画し、BDF フォントが
//! 正しく表示されるかを目視確認する。ESC/Q で終了。

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use harbour_shell::font::{load_mplus_f12r, FontSystem};
use harbour_shell::theme::{Theme, ThemeManager};

fn main() -> Result<(), String> {
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("Harbour OS - Text Test", 480, 320)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    // フォントロード
    let mut fs = FontSystem::new();
    fs.load_font(load_mplus_f12r());

    // テーマ
    let theme = Theme::load_from_yaml("assets/themes/default.yaml")?;
    let tm = ThemeManager::new(theme);

    let samples: &[(&str, Color)] = &[
        ("0123456789", tm.get_color("highlight")),
        ("Harbour OS", tm.get_color("foreground")),
        ("Harbour OS v0.1", tm.get_color("accent")),
        ("Clock Diary Files", tm.get_color("foreground")),
        ("the quick brown fox", tm.get_color("highlight")),
        ("JUMPY FOX", tm.get_color("accent")),
    ];

    let mut events = sdl.event_pump()?;
    'main: loop {
        for e in events.poll_iter() {
            match e {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::Q),
                    ..
                } => break 'main,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        // 各行を描画（y を 20px ごと）
        for (i, (text, col)) in samples.iter().enumerate() {
            let y = 20 + i as i32 * 20;
            fs.draw_text(&mut canvas, 20, y, text, *col);
        }

        // 太字テスト
        fs.draw_text_bold(
            &mut canvas,
            20,
            160,
            "BOLD: Harbour",
            tm.get_color("highlight"),
        );

        // 見つからない文字のプレースホルダテスト
        fs.draw_text(
            &mut canvas,
            20,
            190,
            "missing: <>[]{}",
            tm.get_color("accent"),
        );

        // 測定結果の表示用マーカー
        let w = fs.measure_text("measure test");
        canvas.set_draw_color(tm.get_color("accent"));
        let _ = canvas.draw_rect(Rect::new(20, 220, w as u32, 14));

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
