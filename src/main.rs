use sdl2::event::{Event, WindowEvent};
use sdl2::keyboard::{Keycode, Mod};
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::video::FullscreenType;

use harbour_shell::calculator::CalculatorState;
use harbour_shell::clock::ClockState;
use harbour_shell::desktop::{AppKind, Desktop, DesktopAction};
use harbour_shell::dialog::{ConfirmDialog, DialogResult};
use harbour_shell::dictionary::DictionaryState;
use harbour_shell::file_browser::{BrowserAction, FileBrowser};
use harbour_shell::font::{load_mplus_f12r, FontSystem};
use harbour_shell::harbour::HarbourOS;
use harbour_shell::input::{KeyInput, PointerAction, PointerButton, PointerInput};
use harbour_shell::scale_anim::ScaleAnim;
use harbour_shell::seashore::Seashore;
use harbour_shell::snake::SnakeState;
use harbour_shell::text_editor::TextEditor;
use harbour_shell::theme::{Theme, ThemeManager};
use harbour_shell::vfs::Vfs;

const WINDOW_WIDTH: u32 = 480;
const WINDOW_HEIGHT: u32 = 320;
const FRAME_MARGIN: i32 = 2;
const TOPBAR_H: i32 = 20;
/// アプリウィンドウのタイトルバー高
const TITLEBAR_H: i32 = 14;
const COMMANDBAR_H: i32 = 17;
const POINTER_W: i32 = 16;
const POINTER_H: i32 = 22;

/// トップレベル状態。
enum Mode {
    Desktop,
    /// 開くアニメーション中（完了後 target へ）
    Opening {
        anim: ScaleAnim,
        target: AppKind,
    },
    /// 閉じるアニメーション中（完了後 Desktop へ）
    Closing {
        anim: ScaleAnim,
    },
    /// 閉じるアニメ→完了後 Editor の開くアニメへ（ファイルを開く時）
    ClosingToEditor {
        anim: ScaleAnim,
    },
    Editor,
    Browser,
    Clock,
    Dictionary,
    Calculator,
    Snake,
    /// エディタの終了確認ダイアログ
    ConfirmQuit,
}

fn main() -> Result<(), String> {
    // HarbourOS is pixel art only. Never allow texture interpolation.
    let _ = sdl2::hint::set("SDL_RENDER_SCALE_QUALITY", "0");
    sdl2::image::init(sdl2::image::InitFlag::PNG)?;
    let sdl = sdl2::init()?;
    sdl.mouse().show_cursor(false);
    let video = sdl.video()?;
    let boot_w = env_u32("HARBOUR_WIDTH").unwrap_or(WINDOW_WIDTH);
    let boot_h = env_u32("HARBOUR_HEIGHT").unwrap_or(WINDOW_HEIGHT);
    let boot_fullscreen = env_bool("HARBOUR_FULLSCREEN");

    let window = video
        .window("Harbour OS", boot_w, boot_h)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .present_vsync()
        .build()
        .map_err(|e| e.to_string())?;
    if boot_fullscreen {
        let _ = canvas.window_mut().set_fullscreen(FullscreenType::Desktop);
    }

    let texture_creator = canvas.texture_creator();

    // フォント
    let mut fs = FontSystem::new();
    fs.load_font(load_mplus_f12r());
    if let Err(e) = fs.load_jis_font("fonts/mplus_j12r.bdf") {
        eprintln!("[font] JIS BDF load failed: {}", e);
    }

    // テーマ
    let theme = Theme::load_from_yaml("assets/themes/default.yaml")?;
    let tm = ThemeManager::new(theme);

    // Seashore（topbar + アセット）
    let mut os = HarbourOS::new();
    let mut seashore = Seashore::new(&texture_creator)?;

    // VFS
    let vfs_path =
        std::env::var("HARBOUR_VFS_PATH").unwrap_or_else(|_| "harbour_vfs.yaml".to_string());
    let export_dir =
        std::env::var("HARBOUR_EXPORT_DIR").unwrap_or_else(|_| "harbour_export".to_string());
    let import_dir =
        std::env::var("HARBOUR_IMPORT_DIR").unwrap_or_else(|_| "harbour_import".to_string());
    let mut vfs = Vfs::load_or_samples(&vfs_path);
    if !std::path::Path::new(&vfs_path).exists() {
        save_vfs_snapshot(&vfs, &vfs_path);
    }

    // デスクトップ
    let mut desktop = Desktop::new();

    // TextEditor / FileBrowser（遅延・都度生成）
    let mut editor = TextEditor::with_welcome();
    let mut editor_path: Option<String> = None;
    let mut browser: Option<FileBrowser> = None;
    let clock = ClockState::new();
    let mut dictionary = DictionaryState::new();
    let mut calculator = CalculatorState::new();
    let mut snake = SnakeState::new();

    let mut dialog = ConfirmDialog::new();

    let mut mode = Mode::Desktop;

    let mut events = sdl.event_pump()?;
    let mouse = events.mouse_state();
    os.input_pointer(PointerInput::move_to(mouse.x(), mouse.y(), 0, 0));

    'main: loop {
        for e in events.poll_iter() {
            match e {
                Event::Quit { .. } => break 'main,

                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Enter | WindowEvent::FocusGained => {
                        sdl.mouse().show_cursor(false);
                        let pointer = *os.pointer();
                        os.input_pointer(PointerInput::boundary(
                            PointerAction::Enter,
                            pointer.x,
                            pointer.y,
                        ));
                    }
                    WindowEvent::Leave => {
                        let pointer = *os.pointer();
                        os.input_pointer(PointerInput::boundary(
                            PointerAction::Leave,
                            pointer.x,
                            pointer.y,
                        ));
                    }
                    _ => {}
                },

                Event::MouseMotion {
                    x, y, xrel, yrel, ..
                } => {
                    sdl.mouse().show_cursor(false);
                    let input = PointerInput::move_to(x, y, xrel, yrel);
                    os.input_pointer(input);
                    handle_pointer_input(
                        input,
                        &mut mode,
                        &mut desktop,
                        &mut browser,
                        &mut editor_path,
                        &mut editor,
                        &mut vfs,
                        &vfs_path,
                        &canvas,
                    );
                }

                Event::MouseButtonDown {
                    x,
                    y,
                    mouse_btn,
                    clicks,
                    ..
                } => {
                    sdl.mouse().show_cursor(false);
                    if let Some(button) = PointerButton::from_sdl(mouse_btn) {
                        let input =
                            PointerInput::button(PointerAction::ButtonDown, x, y, button, clicks);
                        os.input_pointer(input);
                        handle_pointer_input(
                            input,
                            &mut mode,
                            &mut desktop,
                            &mut browser,
                            &mut editor_path,
                            &mut editor,
                            &mut vfs,
                            &vfs_path,
                            &canvas,
                        );
                    }
                }

                Event::MouseButtonUp {
                    x,
                    y,
                    mouse_btn,
                    clicks,
                    ..
                } => {
                    sdl.mouse().show_cursor(false);
                    if let Some(button) = PointerButton::from_sdl(mouse_btn) {
                        let input =
                            PointerInput::button(PointerAction::ButtonUp, x, y, button, clicks);
                        os.input_pointer(input);
                        handle_pointer_input(
                            input,
                            &mut mode,
                            &mut desktop,
                            &mut browser,
                            &mut editor_path,
                            &mut editor,
                            &mut vfs,
                            &vfs_path,
                            &canvas,
                        );
                    }
                }

                Event::MouseWheel { x, y, .. } => {
                    let pointer = *os.pointer();
                    let input = PointerInput::wheel(pointer.x, pointer.y, x, y);
                    os.input_pointer(input);
                    handle_pointer_input(
                        input,
                        &mut mode,
                        &mut desktop,
                        &mut browser,
                        &mut editor_path,
                        &mut editor,
                        &mut vfs,
                        &vfs_path,
                        &canvas,
                    );
                }

                Event::KeyDown {
                    keycode,
                    keymod,
                    repeat,
                    ..
                } => {
                    if repeat {
                        continue;
                    }
                    let Some(key) = keycode else { continue };
                    os.input_key(KeyInput::new(key, keymod));

                    // 共通: Ctrl+Q 終了、F11 フルスクリーン
                    if is_ctrl_q(key, keymod) {
                        break 'main;
                    }
                    if key == Keycode::F11 {
                        toggle_fullscreen(&mut canvas);
                        continue;
                    }

                    match mode {
                        Mode::Desktop => match desktop.handle_key(key) {
                            DesktopAction::Launch(kind) => {
                                // アイコン矩形を起点に Opening アニメ
                                let (out_w, out_h) = canvas
                                    .output_size()
                                    .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                                desktop.set_view(out_w, out_h.saturating_sub(TOPBAR_H as u32));
                                let from = icon_rect(&desktop, desktop.selected, out_w, TOPBAR_H);
                                let to = app_window_rect(&canvas);
                                mode = Mode::Opening {
                                    anim: ScaleAnim::new_opening(from, to),
                                    target: kind,
                                };
                            }
                            DesktopAction::None => {}
                        },

                        Mode::Opening { .. }
                        | Mode::Closing { .. }
                        | Mode::ClosingToEditor { .. } => {
                            // アニメ中はキー無視
                        }

                        Mode::Editor => {
                            if key == Keycode::Escape {
                                mode = Mode::ConfirmQuit;
                                continue;
                            }
                            if key == Keycode::B
                                && (keymod & Mod::LCTRLMOD != Mod::NOMOD
                                    || keymod & Mod::RCTRLMOD != Mod::NOMOD)
                            {
                                // エディタからブラウザへ直接（アニメ省略）
                                browser = Some(FileBrowser::new(&vfs));
                                mode = Mode::Browser;
                                continue;
                            }
                            if key == Keycode::S
                                && (keymod & Mod::LCTRLMOD != Mod::NOMOD
                                    || keymod & Mod::RCTRLMOD != Mod::NOMOD)
                            {
                                let saved_path =
                                    save_editor_to_vfs(&mut editor, &editor_path, &mut vfs);
                                editor_path = Some(saved_path);
                                save_vfs_snapshot(&vfs, &vfs_path);
                                continue;
                            }
                            editor.input_key(key, keymod);
                        }

                        Mode::Browser => {
                            if key == Keycode::Escape {
                                if browser
                                    .as_ref()
                                    .map(|br| br.has_modal_input())
                                    .unwrap_or(false)
                                {
                                    if let Some(ref mut br) = browser {
                                        let _ = br.handle_key_public(key, keymod, &mut vfs);
                                    }
                                    continue;
                                }
                                // デスクトップへ戻る（閉じるアニメ）
                                let (out_w, _) = canvas
                                    .output_size()
                                    .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                                let to = icon_rect(&desktop, desktop.selected, out_w, TOPBAR_H);
                                let from = app_window_rect(&canvas);
                                mode = Mode::Closing {
                                    anim: ScaleAnim::new_closing(from, to),
                                };
                                browser = None;
                                continue;
                            }
                            let mut open_path: Option<String> = None;
                            if let Some(ref mut br) = browser {
                                let _ = br.handle_key_public(key, keymod, &mut vfs);
                                if br.take_dirty() {
                                    save_vfs_snapshot(&vfs, &vfs_path);
                                }
                                match br.take_action() {
                                    BrowserAction::Export(path) => {
                                        match vfs.export_to_host(&path, &export_dir) {
                                            Ok(()) => br.set_message(format!(
                                                "exported {} -> {}",
                                                path, export_dir
                                            )),
                                            Err(e) => {
                                                br.set_message(format!("export error: {}", e))
                                            }
                                        }
                                    }
                                    BrowserAction::ImportInto(dest) => {
                                        match vfs.import_host_dir(&import_dir, &dest) {
                                            Ok(count) => {
                                                br.set_message(format!(
                                                    "imported {} file(s) from {}",
                                                    count, import_dir
                                                ));
                                                br.refresh_public(&vfs);
                                                save_vfs_snapshot(&vfs, &vfs_path);
                                            }
                                            Err(e) => {
                                                br.set_message(format!("import error: {}", e))
                                            }
                                        }
                                    }
                                    BrowserAction::OpenFile(path) => {
                                        open_path = Some(path);
                                    }
                                    BrowserAction::None => {}
                                }
                            }
                            if let Some(path) = open_path {
                                if let Ok(content) = vfs.read_file(&path) {
                                    editor = TextEditor::from_text(content);
                                    editor_path = Some(path);
                                    // ブラウザ→エディタへ：
                                    // 閉じるアニメ → 開くアニメ → Editor
                                    let (out_w, _) = canvas
                                        .output_size()
                                        .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                                    let to = icon_rect(&desktop, desktop.selected, out_w, TOPBAR_H);
                                    let from = app_window_rect(&canvas);
                                    browser = None;
                                    mode = Mode::ClosingToEditor {
                                        anim: ScaleAnim::new_closing(from, to),
                                    };
                                }
                            }
                        }

                        Mode::Clock => {
                            if key == Keycode::Escape {
                                mode = closing_to_desktop(&canvas, &desktop);
                            }
                        }

                        Mode::Dictionary => {
                            if key == Keycode::Escape {
                                mode = closing_to_desktop(&canvas, &desktop);
                                continue;
                            }
                            dictionary.input_key(key);
                        }

                        Mode::Calculator => {
                            if key == Keycode::Escape {
                                mode = closing_to_desktop(&canvas, &desktop);
                                continue;
                            }
                            calculator.input_key(key);
                        }

                        Mode::Snake => {
                            if key == Keycode::Escape {
                                mode = closing_to_desktop(&canvas, &desktop);
                                continue;
                            }
                            snake.input_key(key);
                        }

                        Mode::ConfirmQuit => {
                            if let Some(result) = dialog.input_key(key, keymod) {
                                match result {
                                    DialogResult::Cancel => mode = Mode::Editor,
                                    DialogResult::Chosen(choice) => {
                                        use harbour_shell::dialog::DialogChoice;
                                        if matches!(choice, DialogChoice::Save) {
                                            let saved_path = save_editor_to_vfs(
                                                &mut editor,
                                                &editor_path,
                                                &mut vfs,
                                            );
                                            editor_path = Some(saved_path);
                                            save_vfs_snapshot(&vfs, &vfs_path);
                                        }
                                        // デスクトップへ（閉じるアニメ）
                                        let (out_w, _) = canvas
                                            .output_size()
                                            .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                                        let to =
                                            icon_rect(&desktop, desktop.selected, out_w, TOPBAR_H);
                                        let from = app_window_rect(&canvas);
                                        mode = Mode::Closing {
                                            anim: ScaleAnim::new_closing(from, to),
                                        };
                                    }
                                }
                            }
                        }
                    }
                }

                Event::TextInput { text, .. } => match mode {
                    Mode::Editor => {
                        for ch in text.chars() {
                            editor.input_char(ch);
                        }
                    }
                    Mode::Browser => {
                        if let Some(ref mut br) = browser {
                            for ch in text.chars() {
                                br.input_char(ch);
                            }
                        }
                    }
                    Mode::Dictionary => {
                        for ch in text.chars() {
                            dictionary.input_char(ch);
                        }
                    }
                    Mode::Calculator => {
                        for ch in text.chars() {
                            calculator.input_char(ch);
                        }
                    }
                    _ => {}
                },

                _ => {}
            }
        }

        // ---- アニメ進行 ----
        let mut next_mode: Option<Mode> = None;
        if let Mode::Opening { anim, target } = &mut mode {
            let done = anim.step();
            if done {
                next_mode = Some(match target {
                    AppKind::TextEditor => Mode::Editor,
                    AppKind::FileBrowser => {
                        browser = Some(FileBrowser::new(&vfs));
                        Mode::Browser
                    }
                    AppKind::Clock => Mode::Clock,
                    AppKind::Dictionary => Mode::Dictionary,
                    AppKind::Calculator => Mode::Calculator,
                    AppKind::Snake => Mode::Snake,
                });
            }
        } else if let Mode::Closing { anim } = &mut mode {
            let done = anim.step();
            if done {
                next_mode = Some(Mode::Desktop);
            }
        } else if let Mode::ClosingToEditor { anim } = &mut mode {
            let done = anim.step();
            if done {
                // 閉じる完了 → Editor の開くアニメへ
                let (out_w, _) = canvas
                    .output_size()
                    .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                let from = icon_rect(&desktop, desktop.selected, out_w, TOPBAR_H);
                let to = app_window_rect(&canvas);
                next_mode = Some(Mode::Opening {
                    anim: ScaleAnim::new_opening(from, to),
                    target: AppKind::TextEditor,
                });
            }
        }
        if let Some(m) = next_mode {
            mode = m;
        }

        // ---- 更新 ----
        seashore.update(&mut os);
        if let Mode::Snake = mode {
            snake.update();
        }

        // ---- 描画 ----
        let (out_w, out_h) = canvas
            .output_size()
            .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
        os.clamp_pointer(out_w, out_h);
        desktop.set_view(out_w, out_h.saturating_sub(TOPBAR_H as u32));
        let bg = tm.get_color("background");
        canvas.set_draw_color(Color::RGB(bg.r, bg.g, bg.b));
        canvas.clear();

        let bar_h = seashore.draw_top_bar(&mut canvas, &os);

        // アプリ領域矩形（topbar の下）
        let app_rect = Rect::new(
            FRAME_MARGIN,
            bar_h + FRAME_MARGIN,
            out_w.saturating_sub(FRAME_MARGIN as u32 * 2),
            out_h
                .saturating_sub(bar_h as u32)
                .saturating_sub(FRAME_MARGIN as u32 * 2),
        );

        match &mode {
            Mode::Desktop => {
                desktop.draw(
                    &mut canvas,
                    &mut seashore.assets,
                    &fs,
                    &tm,
                    FRAME_MARGIN,
                    bar_h,
                    out_w,
                    out_h,
                );
            }
            Mode::Opening { anim, .. } => {
                // デスクトップを薄く残し + アニメ枠
                desktop.draw(
                    &mut canvas,
                    &mut seashore.assets,
                    &fs,
                    &tm,
                    FRAME_MARGIN,
                    bar_h,
                    out_w,
                    out_h,
                );
                anim.draw(&mut canvas, &tm);
            }
            Mode::Closing { anim } | Mode::ClosingToEditor { anim } => {
                anim.draw(&mut canvas, &tm);
            }
            Mode::Editor => {
                let title = editor_title(&editor_path, editor.is_dirty());
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, &title);
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                editor.set_view(content.width(), content.height());
                editor.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                draw_command_bar(
                    &mut canvas,
                    &fs,
                    &tm,
                    &inner,
                    "",
                    "Ctrl+S Save   Ctrl+Z Undo   Ctrl+B Files   Esc Close",
                );
            }
            Mode::Browser => {
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, "Files");
                if let Some(ref mut br) = browser {
                    let inner = inner_rect(&app_rect);
                    let content = content_rect(&inner);
                    br.draw_with_vfs(
                        &mut canvas,
                        &mut seashore.assets,
                        &fs,
                        &tm,
                        content.x(),
                        content.y(),
                        content.width(),
                        content.height(),
                        &vfs,
                    );
                    let footer = br.footer_text();
                    draw_command_bar(
                        &mut canvas,
                        &fs,
                        &tm,
                        &inner,
                        &footer,
                        "Enter Open   N New   S+N Folder   R Rename   D Del   E Export   I Import   Esc Back",
                    );
                }
            }
            Mode::Clock => {
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, "Clock");
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                clock.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                draw_command_bar(&mut canvas, &fs, &tm, &inner, "", "Esc Desktop");
            }
            Mode::Dictionary => {
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, "Dictionary");
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                dictionary.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                draw_command_bar(
                    &mut canvas,
                    &fs,
                    &tm,
                    &inner,
                    "",
                    "Enter Lookup   Backspace Delete   Esc Desktop",
                );
            }
            Mode::Calculator => {
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, "Calculator");
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                calculator.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                draw_command_bar(
                    &mut canvas,
                    &fs,
                    &tm,
                    &inner,
                    "",
                    "Enter Calculate   Backspace Delete   C Clear   Esc Desktop",
                );
            }
            Mode::Snake => {
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, "Snake");
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                snake.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                let footer = snake.footer_text();
                draw_command_bar(
                    &mut canvas,
                    &fs,
                    &tm,
                    &inner,
                    &footer,
                    "Arrows Turn   R/Space Restart   Esc Desktop",
                );
            }
            Mode::ConfirmQuit => {
                let title = editor_title(&editor_path, editor.is_dirty());
                draw_app_window(&mut canvas, &fs, &tm, &app_rect, &title);
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                editor.set_view(content.width(), content.height());
                editor.draw(
                    &mut canvas,
                    &fs,
                    &tm,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                );
                draw_command_bar(
                    &mut canvas,
                    &fs,
                    &tm,
                    &inner,
                    "Quit TextEditor?",
                    "Tab Select   Enter Confirm   Esc Cancel",
                );
                dialog.draw(&mut canvas, &fs, &tm, out_w, out_h);
            }
        }

        let pointer = os.pointer();
        draw_pointer(&mut canvas, &tm, pointer.x, pointer.y);

        canvas.present();
        sdl.mouse().show_cursor(false);
        std::thread::sleep(std::time::Duration::from_millis(16));
    }

    Ok(())
}

fn handle_pointer_input(
    input: PointerInput,
    mode: &mut Mode,
    desktop: &mut Desktop,
    browser: &mut Option<FileBrowser>,
    editor_path: &mut Option<String>,
    editor: &mut TextEditor,
    vfs: &mut Vfs,
    vfs_path: &str,
    canvas: &sdl2::render::Canvas<sdl2::video::Window>,
) {
    if matches!(
        mode,
        Mode::Opening { .. } | Mode::Closing { .. } | Mode::ClosingToEditor { .. }
    ) {
        return;
    }

    if input.action == PointerAction::ButtonUp
        && input.button == Some(PointerButton::Left)
        && app_close_rect(canvas).contains_point((input.x, input.y))
    {
        match mode {
            Mode::Editor => {
                *mode = Mode::ConfirmQuit;
                return;
            }
            Mode::Browser => {
                *mode = closing_to_desktop(canvas, desktop);
                *browser = None;
                return;
            }
            Mode::Clock | Mode::Dictionary | Mode::Calculator | Mode::Snake => {
                *mode = closing_to_desktop(canvas, desktop);
                return;
            }
            Mode::Desktop | Mode::ConfirmQuit => {}
            Mode::Opening { .. } | Mode::Closing { .. } | Mode::ClosingToEditor { .. } => {}
        }
    }

    match mode {
        Mode::Desktop => {
            let (out_w, out_h) = canvas
                .output_size()
                .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
            desktop.set_view(out_w, out_h.saturating_sub(TOPBAR_H as u32));
            if let DesktopAction::Launch(kind) =
                desktop.handle_pointer(input, FRAME_MARGIN, TOPBAR_H, out_w, out_h)
            {
                *mode = opening_from_desktop(canvas, desktop, kind);
            }
        }
        Mode::Browser => {
            if let Some(br) = browser {
                let app_rect = app_window_rect(canvas);
                let inner = inner_rect(&app_rect);
                let content = content_rect(&inner);
                let _ = br.handle_pointer_public(
                    input,
                    content.x(),
                    content.y(),
                    content.width(),
                    content.height(),
                    vfs,
                );
                if br.take_dirty() {
                    save_vfs_snapshot(vfs, vfs_path);
                }

                let mut open_path = None;
                match br.take_action() {
                    BrowserAction::OpenFile(path) => open_path = Some(path),
                    BrowserAction::None => {}
                    BrowserAction::Export(_) | BrowserAction::ImportInto(_) => {}
                }
                if let Some(path) = open_path {
                    if let Ok(content) = vfs.read_file(&path) {
                        *editor = TextEditor::from_text(content);
                        *editor_path = Some(path);
                        let (out_w, _) = canvas
                            .output_size()
                            .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
                        let to = icon_rect(desktop, desktop.selected, out_w, TOPBAR_H);
                        let from = app_window_rect(canvas);
                        *browser = None;
                        *mode = Mode::ClosingToEditor {
                            anim: ScaleAnim::new_closing(from, to),
                        };
                    }
                }
            }
        }
        Mode::Editor
        | Mode::Clock
        | Mode::Dictionary
        | Mode::Calculator
        | Mode::Snake
        | Mode::ConfirmQuit => {}
        Mode::Opening { .. } | Mode::Closing { .. } | Mode::ClosingToEditor { .. } => {}
    }
}

fn opening_from_desktop(
    canvas: &sdl2::render::Canvas<sdl2::video::Window>,
    desktop: &Desktop,
    target: AppKind,
) -> Mode {
    let (out_w, _) = canvas
        .output_size()
        .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
    let from = icon_rect(desktop, desktop.selected, out_w, TOPBAR_H);
    let to = app_window_rect(canvas);
    Mode::Opening {
        anim: ScaleAnim::new_opening(from, to),
        target,
    }
}

fn app_close_rect(canvas: &sdl2::render::Canvas<sdl2::video::Window>) -> Rect {
    let rect = app_window_rect(canvas);
    Rect::new(rect.x() + 4, rect.y() + 3, 10, 10)
}

/// アイコンの画面上の矩形（アニメ起点）。簡易: グリッド位置から計算。
fn icon_rect(desktop: &Desktop, idx: usize, screen_w: u32, topbar_h: i32) -> Rect {
    let cell_w = 72i32;
    let cell_h = 56i32;
    let cols = desktop.cols.max(1);
    let grid_w = (cols as i32 * cell_w).min(screen_w as i32);
    let grid_x = FRAME_MARGIN + ((screen_w as i32 - grid_w) / 2).max(8);
    let grid_y = topbar_h + 24;
    let col = (idx % cols) as i32;
    let row = (idx / cols) as i32;
    let cx = grid_x + col * cell_w;
    let cy = grid_y + row * cell_h;
    Rect::new(cx + (cell_w - 32) / 2, cy, 32, 32)
}

fn closing_to_desktop(
    canvas: &sdl2::render::Canvas<sdl2::video::Window>,
    desktop: &Desktop,
) -> Mode {
    let (out_w, _) = canvas
        .output_size()
        .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
    let to = icon_rect(desktop, desktop.selected, out_w, TOPBAR_H);
    let from = app_window_rect(canvas);
    Mode::Closing {
        anim: ScaleAnim::new_closing(from, to),
    }
}

/// アプリウィンドウ領域。
fn app_window_rect(canvas: &sdl2::render::Canvas<sdl2::video::Window>) -> Rect {
    let (w, h) = canvas
        .output_size()
        .unwrap_or((WINDOW_WIDTH, WINDOW_HEIGHT));
    Rect::new(
        FRAME_MARGIN,
        TOPBAR_H + FRAME_MARGIN,
        w.saturating_sub(FRAME_MARGIN as u32 * 2),
        h.saturating_sub(TOPBAR_H as u32)
            .saturating_sub(FRAME_MARGIN as u32 * 2),
    )
}

/// タイトルバー付きアプリ枠を描画。
fn draw_app_window(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    fs: &FontSystem,
    tm: &ThemeManager,
    rect: &Rect,
    title: &str,
) {
    let amber = tm.get_color("foreground");
    let muted = tm.get_color("muted");

    // 外枠は全アプリ共通の2px罫線。
    harbour_shell::ui::draw_rect_thick(canvas, *rect, amber);

    // タイトルバー下線
    harbour_shell::ui::line(
        canvas,
        rect.x() + 1,
        rect.y() + TITLEBAR_H,
        rect.x() + rect.width() as i32 - 2,
        rect.y() + TITLEBAR_H,
        amber,
    );

    // タイトル
    let title = fit_text(fs, title, rect.width() as i32 - 36);
    fs.draw_text(canvas, rect.x() + 18, rect.y() + 1, &title, amber);

    // 左端の小さなシステムマーク。ボタンではなく装飾として統一する。
    harbour_shell::ui::draw_rect(canvas, Rect::new(rect.x() + 5, rect.y() + 4, 7, 7), muted);
    harbour_shell::ui::line(
        canvas,
        rect.x() + 7,
        rect.y() + 6,
        rect.x() + 10,
        rect.y() + 9,
        amber,
    );
}

fn draw_command_bar(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    fs: &FontSystem,
    tm: &ThemeManager,
    inner: &Rect,
    status: &str,
    commands: &str,
) {
    if inner.height() < COMMANDBAR_H as u32 {
        return;
    }
    let amber = tm.get_color("foreground");
    let muted = tm.get_color("muted");
    let y = inner.y() + inner.height() as i32 - COMMANDBAR_H;
    harbour_shell::ui::line(
        canvas,
        inner.x() + 1,
        y,
        inner.x() + inner.width() as i32 - 2,
        y,
        amber,
    );

    let max_w = inner.width() as i32 - 10;
    if status.trim().is_empty() {
        let text = fit_text(fs, commands, max_w);
        fs.draw_text(canvas, inner.x() + 5, y + 2, &text, muted);
    } else {
        let status = fit_text(fs, status, max_w / 2);
        fs.draw_text(canvas, inner.x() + 5, y + 2, &status, amber);
        let sx = inner.x() + 5 + fs.measure_text(&status) + 6;
        fs.draw_text(canvas, sx, y + 2, "|", amber);
        let commands_w = max_w - fs.measure_text(&status) - 18;
        if commands_w > 12 {
            let commands = fit_text(fs, commands, commands_w);
            fs.draw_text(canvas, sx + 12, y + 2, &commands, muted);
        }
    }
}

/// タイトルバーを除いた内側領域。
fn inner_rect(rect: &Rect) -> Rect {
    Rect::new(
        rect.x() + 2,
        rect.y() + TITLEBAR_H + 1,
        rect.width().saturating_sub(4),
        rect.height().saturating_sub(TITLEBAR_H as u32 + 3),
    )
}

fn content_rect(inner: &Rect) -> Rect {
    Rect::new(
        inner.x(),
        inner.y(),
        inner.width(),
        inner.height().saturating_sub(COMMANDBAR_H as u32),
    )
}

fn draw_pointer(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    tm: &ThemeManager,
    x: i32,
    y: i32,
) {
    let max_x = canvas
        .output_size()
        .map(|(w, _)| w as i32 - POINTER_W)
        .unwrap_or(WINDOW_WIDTH as i32 - POINTER_W)
        .max(0);
    let max_y = canvas
        .output_size()
        .map(|(_, h)| h as i32 - POINTER_H)
        .unwrap_or(WINDOW_HEIGHT as i32 - POINTER_H)
        .max(0);
    let px = x.clamp(0, max_x);
    let py = y.clamp(0, max_y);
    harbour_shell::ui::draw_pointer_cursor(
        canvas,
        px,
        py,
        tm.get_color("foreground"),
        tm.get_color("highlight"),
        tm.get_color("shadow"),
    );
}

fn is_ctrl_q(key: Keycode, keymod: Mod) -> bool {
    key == Keycode::Q
        && (keymod & Mod::LCTRLMOD != Mod::NOMOD || keymod & Mod::RCTRLMOD != Mod::NOMOD)
}

fn env_u32(name: &str) -> Option<u32> {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse::<u32>().ok())
        .filter(|v| *v >= 160)
}

fn env_bool(name: &str) -> bool {
    std::env::var(name)
        .map(|v| {
            matches!(
                v.as_str(),
                "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
            )
        })
        .unwrap_or(false)
}

fn save_editor_to_vfs(editor: &mut TextEditor, path: &Option<String>, vfs: &mut Vfs) -> String {
    let content = editor.text_for_save();
    let p = path.clone().unwrap_or_else(|| "/Untitled.txt".to_string());
    let _ = vfs.write_file(&p, &content);
    editor.mark_clean();
    p
}

fn editor_title(path: &Option<String>, dirty: bool) -> String {
    let marker = if dirty { "*" } else { "" };
    match path {
        Some(path) => format!("TextEditor{} - {}", marker, path),
        None => format!("TextEditor{} - /Untitled.txt", marker),
    }
}

fn fit_text(fs: &FontSystem, text: &str, max_w: i32) -> String {
    if fs.measure_text(text) <= max_w {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars() {
        let candidate = format!("{}{}...", out, ch);
        if fs.measure_text(&candidate) > max_w {
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

fn save_vfs_snapshot(vfs: &Vfs, path: &str) {
    if let Err(e) = vfs.save_to_file(path) {
        eprintln!("[vfs] save failed: {}", e);
    }
}

fn toggle_fullscreen(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    let cur = canvas.window().fullscreen_state();
    if cur == FullscreenType::Off {
        let _ = canvas.window_mut().set_fullscreen(FullscreenType::Desktop);
    } else {
        let _ = canvas.window_mut().set_fullscreen(FullscreenType::Off);
    }
}
