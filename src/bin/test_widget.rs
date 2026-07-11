//! test_widget - Classic ベベルの多層染色を検証するスタンドアロン実行ファイル。
//!
//! 生成したウィジェット PNG（face/outline/highlight/shadow）を theme 色で
//! 重ね描きし、正しくベベルが表現できるかを目視確認する。
//! ESC/Q で終了。

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;

use harbour_shell::assets::AssetStore;
use harbour_shell::theme::{Theme, ThemeManager};

fn main() -> Result<(), String> {
    sdl2::image::init(sdl2::image::InitFlag::PNG)?;
    let sdl = sdl2::init()?;
    let video = sdl.video()?;

    let window = video
        .window("Harbour OS - Widget Test", 320, 240)
        .position_centered()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let mut assets = AssetStore::new(&creator)?;

    let theme = Theme::load_from_yaml("assets/themes/default.yaml")?;
    let tm = ThemeManager::new(theme);

    // ベベル色セット
    let face = tm.get_color("widget_face");
    let outline = tm.get_color("widget_outline");
    let highlight = tm.get_color("widget_highlight");
    let shadow = tm.get_color("widget_shadow");
    let fg = tm.get_color("foreground");
    let stripe_a = tm.get_color("titlebar_stripe_a");
    let stripe_b = tm.get_color("titlebar_stripe_b");

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

        // 1) プッシュボタン（up）多層ベベル
        let _ = assets.blit_layers(
            &mut canvas,
            &[
                "widgets/buttons/push_face",
                "widgets/buttons/push_shadow",
                "widgets/buttons/push_highlight",
                "widgets/buttons/push_outline",
            ],
            &[face, shadow, highlight, outline],
            Some(Rect::new(20, 20, 40, 20)),
        );

        // 2) プッシュボタン（down）
        let _ = assets.blit_layers(
            &mut canvas,
            &[
                "widgets/buttons/push_down_face",
                "widgets/buttons/push_down_shadow",
                "widgets/buttons/push_down_highlight",
                "widgets/buttons/push_down_outline",
            ],
            &[face, shadow, highlight, outline],
            Some(Rect::new(70, 20, 40, 20)),
        );

        // 3) チェックボックス off / on（outline 染色）
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/buttons/checkbox_off_outline",
            Some(Rect::new(20, 60, 11, 11)),
            outline,
        );
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/buttons/checkbox_on_outline",
            Some(Rect::new(40, 60, 11, 11)),
            outline,
        );

        // 4) ラジオ off / on
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/buttons/radio_off_outline",
            Some(Rect::new(70, 60, 11, 11)),
            outline,
        );
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/buttons/radio_on_outline",
            Some(Rect::new(90, 60, 11, 11)),
            outline,
        );

        // 5) タイトルバー縞（a/b 2 層をずらして市松に）
        for x in (0..320).step_by(2) {
            let _ = assets.blit_tinted(
                &mut canvas,
                "widgets/window/titlebar_stripe",
                Some(Rect::new(x, 90, 2, 20)),
                stripe_a,
            );
            let _ = assets.blit_tinted(
                &mut canvas,
                "widgets/window/titlebar_stripe",
                Some(Rect::new(x + 1, 90, 2, 20)),
                stripe_b,
            );
        }

        // 6) closebox / zoombox
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/window/closebox_outline",
            Some(Rect::new(20, 120, 11, 11)),
            outline,
        );
        let _ = assets.blit_tinted(
            &mut canvas,
            "widgets/window/zoombox_outline",
            Some(Rect::new(40, 120, 11, 11)),
            outline,
        );

        // 7) ステータスアイコン（fg 染色）
        let _ = assets.blit_tinted(
            &mut canvas,
            "status/wifi_three",
            Some(Rect::new(20, 150, 16, 16)),
            fg,
        );
        let _ = assets.blit_tinted(
            &mut canvas,
            "status/speaker_high",
            Some(Rect::new(45, 150, 16, 16)),
            fg,
        );
        let _ = assets.blit_tinted(
            &mut canvas,
            "status/battery_full",
            Some(Rect::new(70, 150, 16, 16)),
            fg,
        );

        canvas.present();
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}
