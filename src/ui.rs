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
