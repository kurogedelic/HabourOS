use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

// UIコンポーネント描画関数。
//
// 設計方針:
// - 色はハードコード定数を持たず、すべて呼び出し側（ThemeManager 経由）から
//   引数で受け取る。これによりテーマ変更が UI 全体へ伝播する。
// - ここではジオメトリ描画（矩形・線・円）のみを提供する。
//   GUI 部品そのものは PNG アセット（assets.rs）で表現する。

pub fn draw_rect(c: &mut Canvas<Window>, r: Rect, col: Color) {
    c.set_draw_color(col);
    let _ = c.draw_rect(r);
}

// 2pxの太い枠を描画
pub fn draw_rect_thick(c: &mut Canvas<Window>, r: Rect, col: Color) {
    c.set_draw_color(col);

    // 外側の枠（1px）
    let _ = c.draw_rect(r);

    // 内側の枠（1px内側）
    let inner_rect = Rect::new(
        r.x() + 1,
        r.y() + 1,
        r.width().saturating_sub(2),
        r.height().saturating_sub(2),
    );
    let _ = c.draw_rect(inner_rect);
}

pub fn fill_rect(c: &mut Canvas<Window>, r: Rect, col: Color) {
    c.set_draw_color(col);
    let _ = c.fill_rect(r);
}

pub fn line(c: &mut Canvas<Window>, x1: i32, y1: i32, x2: i32, y2: i32, col: Color) {
    c.set_draw_color(col);
    let _ = c.draw_line((x1, y1), (x2, y2));
}

/// 1-bit 風のポインタカーソルを描画する。
/// Small Classic Mac-style pointer for 320×170 displays.
/// Hotspot is the upper-left pixel, matching SDL pointer coordinates.
pub fn draw_compact_pointer_cursor(
    c: &mut Canvas<Window>,
    x: i32,
    y: i32,
    outline: Color,
    paper: Color,
    shadow: Color,
) {
    const ARROW: [&str; 16] = [
        "X           ",
        "XX          ",
        "XFX         ",
        "XFFX        ",
        "XFFFX       ",
        "XFFFFX      ",
        "XFFFFFX     ",
        "XFFFFFFX    ",
        "XFFFFFFFX   ",
        "XFFFXXXXX   ",
        "XFFX XFFX   ",
        "XFX  XFFX   ",
        "XX    XFFX  ",
        "X     XFFX  ",
        "       XX   ",
        "            ",
    ];
    draw_pointer_pixels(c, x + 1, y + 1, &ARROW, 'X', shadow);
    draw_pointer_pixels(c, x, y, &ARROW, 'F', paper);
    draw_pointer_pixels(c, x, y, &ARROW, 'X', outline);
}

pub fn draw_pointer_cursor(
    c: &mut Canvas<Window>,
    x: i32,
    y: i32,
    outline: Color,
    fill: Color,
    shadow: Color,
) {
    const POINTER: [&str; 22] = [
        "X               ",
        "XX              ",
        "XFX             ",
        "XFFX            ",
        "XFFFX           ",
        "XFFFFX          ",
        "XFFFFFX         ",
        "XFFFFFFX        ",
        "XFFFFFFFX       ",
        "XFFFFFFFFX      ",
        "XFFFFFFXXXXX    ",
        "XFFFXXFFX       ",
        "XFFX XFFX       ",
        "XFX  XFFX       ",
        "XX    XFFX      ",
        "X     XFFX      ",
        "      XFFX      ",
        "       XFFX     ",
        "       XFFX     ",
        "        XX      ",
        "                ",
        "                ",
    ];

    draw_pointer_pixels(c, x + 1, y + 1, &POINTER, 'X', shadow);
    draw_pointer_pixels(c, x, y, &POINTER, 'F', fill);
    draw_pointer_pixels(c, x, y, &POINTER, 'X', outline);
}

fn draw_pointer_pixels(
    c: &mut Canvas<Window>,
    x: i32,
    y: i32,
    rows: &[&str],
    needle: char,
    color: Color,
) {
    c.set_draw_color(color);
    for (row, line) in rows.iter().enumerate() {
        for (col, pixel) in line.chars().enumerate() {
            if pixel == needle {
                let _ = c.draw_point((x + col as i32, y + row as i32));
            }
        }
    }
}

// 円描画（ミッドポイント円アルゴリズム）
pub fn draw_circle(c: &mut Canvas<Window>, center: (i32, i32), radius: i32, col: Color) {
    let (cx, cy) = center;
    let mut x = 0i32;
    let mut y = radius;
    let mut d = 1 - radius;

    c.set_draw_color(col);
    while x <= y {
        let _ = c.draw_point((cx + x, cy + y));
        let _ = c.draw_point((cx + x, cy - y));
        let _ = c.draw_point((cx - x, cy + y));
        let _ = c.draw_point((cx - x, cy - y));
        let _ = c.draw_point((cx + y, cy + x));
        let _ = c.draw_point((cx - y, cy + x));
        let _ = c.draw_point((cx + y, cy - x));
        let _ = c.draw_point((cx - y, cy - x));

        x += 1;
        if d < 0 {
            d += 2 * x + 1;
        } else {
            y -= 1;
            d += 2 * (x - y) + 1;
        }
    }
}

// パネル/ウィンドウ描画
pub fn draw_panel(
    c: &mut Canvas<Window>,
    x: i32,
    y: i32,
    w: u32,
    h: u32,
    _title: &str,
    active: bool,
    outline_col: Color,
    outline_dim_col: Color,
) {
    let border_color = if active { outline_col } else { outline_dim_col };
    c.set_draw_color(border_color);
    let _ = c.draw_rect(Rect::new(x, y, w, h));

    let title_h = 24u32;
    c.set_draw_color(border_color);
    let _ = c.draw_line(
        (x, y + title_h as i32),
        (x + w as i32 - 1, y + title_h as i32),
    );

    // 左側のコントロールボタン
    let btn_w = 24i32;
    for i in 0..3 {
        let bx = x + i * btn_w;
        c.set_draw_color(border_color);
        let _ = c.draw_rect(Rect::new(bx, y, btn_w as u32, title_h));
    }

    // 右側の最大化ボタン
    let rx = x + w as i32 - btn_w;
    c.set_draw_color(border_color);
    let _ = c.draw_rect(Rect::new(rx, y, btn_w as u32, title_h));
}

/// Render icon labels on up to two lines without splitting ordinary words.
/// Prefer spaces, CamelCase, or filename separators; ellipsize a long
/// unbreakable word rather than producing unreadable fragments.
pub fn wrap_icon_label<F>(label: &str, max_width: i32, measure: F) -> Vec<String>
where
    F: Fn(&str) -> i32,
{
    let label = label.trim();
    if label.is_empty() || max_width <= 0 {
        return Vec::new();
    }
    if measure(label) <= max_width {
        return vec![label.to_string()];
    }

    let chars: Vec<char> = label.chars().collect();
    let mut best: Option<(i32, String, String)> = None;
    for i in 1..chars.len() {
        let previous = chars[i - 1];
        let current = chars[i];
        let boundary = previous.is_whitespace()
            || matches!(previous, '_' | '-' | '/')
            || (previous.is_lowercase() && current.is_uppercase());
        if !boundary {
            continue;
        }
        let left: String = chars[..i]
            .iter()
            .collect::<String>()
            .trim_end_matches(|c: char| c.is_whitespace() || c == '_' || c == '-')
            .to_string();
        let right: String = chars[i..]
            .iter()
            .collect::<String>()
            .trim_start_matches(|c: char| c.is_whitespace() || c == '_' || c == '-')
            .to_string();
        if left.is_empty() || right.is_empty() || measure(&left) > max_width {
            continue;
        }
        let shown_right = clip_icon_label(&right, max_width, &measure);
        if shown_right.is_empty() {
            continue;
        }
        // Full words are always preferred to clipped ones. For ties, balance.
        let hidden = right
            .chars()
            .count()
            .saturating_sub(shown_right.trim_end_matches("...").chars().count());
        let score = (if measure(&right) > max_width {
            1000 + hidden as i32 * 40
        } else {
            0
        }) + (measure(&left) - measure(&shown_right)).abs();
        if best.as_ref().map_or(true, |(old, _, _)| score < *old) {
            best = Some((score, left, shown_right));
        }
    }
    match best {
        Some((_, left, right)) => vec![left, right],
        None => vec![clip_icon_label(label, max_width, &measure)],
    }
}

fn clip_icon_label<F>(label: &str, max_width: i32, measure: &F) -> String
where
    F: Fn(&str) -> i32,
{
    if measure(label) <= max_width {
        return label.to_string();
    }
    if measure("...") > max_width {
        return String::new();
    }
    let mut result = String::new();
    for ch in label.chars() {
        if measure(&format!("{}{}...", result, ch)) > max_width {
            break;
        }
        result.push(ch);
    }
    result.push_str("...");
    result
}

#[cfg(test)]
mod icon_label_tests {
    use super::wrap_icon_label;

    fn width(text: &str) -> i32 {
        text.chars().count() as i32 * 6
    }

    #[test]
    fn wraps_text_editor_on_word_boundary() {
        assert_eq!(
            wrap_icon_label("Text Editor", 64, width),
            vec!["Text", "Editor"]
        );
    }

    #[test]
    fn does_not_break_short_unspaced_words() {
        assert_eq!(wrap_icon_label("Dictionary", 64, width), vec!["Dictionary"]);
        assert_eq!(wrap_icon_label("Calculator", 64, width), vec!["Calculator"]);
    }

    #[test]
    fn ellipsizes_unbreakable_words_instead_of_splitting() {
        assert_eq!(wrap_icon_label("Dictionary", 52, width), vec!["Dicti..."]);
    }

    #[test]
    fn long_file_names_fit_in_at_most_two_lines() {
        let lines = wrap_icon_label("very_long_filename_for_notes.txt", 64, width);
        assert!(lines.len() <= 2);
        assert!(lines.iter().all(|line| width(line) <= 64));
    }

    #[test]
    fn short_names_stay_on_one_line() {
        assert_eq!(wrap_icon_label("Files", 64, width), vec!["Files"]);
    }
}
