#!/usr/bin/env python3
"""
gen_widgets.py - HarbourShell GUI 部品 PNG 生成器
================================================================
MacOS Classic 風 × Amber 向けに、白×透明の単色 PNG を生成する。
実行時に SDL2 の Texture::set_color_mod でテーマ色を染色するため、
PNG 側は「描かれるピクセル=白(255,255,255) / 透過=0」の2値構成にする。

Classic ベベルは複数レイヤー(face/outline/highlight/shadow)で表現するため、
1 つの部品を複数 PNG に分割して出力する設計。

使い方:
    python3 tools/gen_widgets.py          # 全部品再生成
    python3 tools/gen_widgets.py --status # status グループのみ
    python3 tools/gen_widgets.py --list   # 出力予定ファイル一覧
"""
import argparse
import sys
from pathlib import Path
from PIL import Image, ImageDraw

# ---- 定数 --------------------------------------------------------------
# 出力ルート（プロジェクトルート/assets/images）
ROOT = Path(__file__).resolve().parent.parent
OUT_ROOT = ROOT / "assets" / "images"

# 白(不透明) と 完全透過。RGBA。
WHITE = (255, 255, 255, 255)
CLEAR = (0, 0, 0, 0)

# Classic 準拠の標準サイズ
SIZE_CLOSEBOX = 11      # Mac OS の closebox は 11x11 相当
SIZE_TITLEBAR = 20      # タイトルバー高
SIZE_PUSH_W, SIZE_PUSH_H = 40, 20   # プッシュボタン
SIZE_CHECK = 11
SIZE_RADIO = 11
SIZE_STATUS = 16        # 既存 status icon 互換


# ---- ユーティリティ ----------------------------------------------------
def new_canvas(w: int, h: int) -> Image.Image:
    """白×透明のキャンバスを作る。"""
    return Image.new("RGBA", (w, h), CLEAR)


def save(img: Image.Image, rel_path: str) -> Path:
    """PNG として保存。親ディレクトリも作成。"""
    dst = OUT_ROOT / rel_path
    dst.parent.mkdir(parents=True, exist_ok=True)
    img.save(dst, "PNG")
    return dst


# ---- ジオメトリ描画プリミティブ（全て白で描く） -------------------------
def draw_rect_outline(draw: ImageDraw.ImageDraw, box, thick: int = 1):
    """矩形の枠を白で描く。box=(x0,y0,x1,y1) は PIL 流（右下含まない）。"""
    x0, y0, x1, y1 = box
    # 上辺
    draw.rectangle([x0, y0, x1 - 1, y0 + thick - 1], fill=WHITE)
    # 下辺
    draw.rectangle([x0, y1 - thick, x1 - 1, y1 - 1], fill=WHITE)
    # 左辺
    draw.rectangle([x0, y0, x0 + thick - 1, y1 - 1], fill=WHITE)
    # 右辺
    draw.rectangle([x1 - thick, y0, x1 - 1, y1 - 1], fill=WHITE)


def draw_rect_fill(draw: ImageDraw.ImageDraw, box):
    """矩形を白でべた塗り。"""
    x0, y0, x1, y1 = box
    draw.rectangle([x0, y0, x1 - 1, y1 - 1], fill=WHITE)


# ---- ウィンドウ枠 ------------------------------------------------------
def gen_window():
    """ウィンドウ周りの部品。"""
    out = []
    # titlebar_stripe: 2x20 の縞。実行時に stripe_a/b 2 層で貼る。
    # ここでは「1px 毎の縞パターン」を表す白ピクセルマップを出す。
    # 横方向に 1px 白 / 1px 透過 の縞。実行時にずらして2層重ねれば市松に。
    img = new_canvas(2, SIZE_TITLEBAR)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 0, SIZE_TITLEBAR - 1], fill=WHITE)  # 左1列だけ白
    out.append(save(img, "widgets/window/titlebar_stripe.png"))

    # closebox: 11x11。枠のみ。中心は透過。
    img = new_canvas(SIZE_CLOSEBOX, SIZE_CLOSEBOX)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, SIZE_CLOSEBOX, SIZE_CLOSEBOX), thick=1)
    out.append(save(img, "widgets/window/closebox_outline.png"))

    # zoombox: 11x11。枠 + 中央に小さい四角。
    img = new_canvas(SIZE_CLOSEBOX, SIZE_CLOSEBOX)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, SIZE_CLOSEBOX, SIZE_CLOSEBOX), thick=1)
    draw_rect_outline(d, (3, 3, SIZE_CLOSEBOX - 3, SIZE_CLOSEBOX - 3), thick=1)
    out.append(save(img, "widgets/window/zoombox_outline.png"))

    # border_outline: 8x8 の角パーツ（四隅に貼る用）。L字の白ピクセル。
    img = new_canvas(8, 8)
    d = ImageDraw.Draw(img)
    # 上辺と左辺
    d.rectangle([0, 0, 7, 0], fill=WHITE)
    d.rectangle([0, 0, 0, 7], fill=WHITE)
    out.append(save(img, "widgets/window/border_corner.png"))

    # ---- 追加: ウィンドウ枠テンプレ ----
    # titlebar_left / titlebar_right: タイトルバー両端の丸み（4x20）
    for side in ("left", "right"):
        img = new_canvas(4, SIZE_TITLEBAR)
        d = ImageDraw.Draw(img)
        if side == "left":
            # 左上の角丸: 右2列を白、左は上から2px目から
            d.rectangle([2, 0, 3, SIZE_TITLEBAR - 1], fill=WHITE)
            d.rectangle([1, 1, 1, SIZE_TITLEBAR - 1], fill=WHITE)
            d.point([0, 2], fill=WHITE)
        else:
            d.rectangle([0, 0, 1, SIZE_TITLEBAR - 1], fill=WHITE)
            d.rectangle([2, 1, 2, SIZE_TITLEBAR - 1], fill=WHITE)
            d.point([3, 2], fill=WHITE)
        out.append(save(img, f"widgets/window/titlebar_{side}.png"))

    # resize_handle: 右下のリサイズグリップ（8x8、斜めライン）
    img = new_canvas(8, 8)
    d = ImageDraw.Draw(img)
    for i in range(3):
        y = 7 - i
        for dx in range(i + 1):
            d.point([7 - dx, y - dx], fill=WHITE)
    out.append(save(img, "widgets/window/resize_handle.png"))

    # scrollbar_v_thumb: 縦スクロールバーのつまみ（10x24、両端にライン）
    img = new_canvas(10, 24)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, 10, 24), thick=1)
    # 中央のグリップ線
    d.rectangle([3, 10, 6, 10], fill=WHITE)
    d.rectangle([3, 12, 6, 12], fill=WHITE)
    d.rectangle([3, 14, 6, 14], fill=WHITE)
    out.append(save(img, "widgets/window/scrollbar_v_thumb.png"))

    # scrollbar_h_thumb: 横スクロールバーのつまみ（24x10）
    img = new_canvas(24, 10)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, 24, 10), thick=1)
    d.rectangle([10, 3, 10, 6], fill=WHITE)
    d.rectangle([12, 3, 12, 6], fill=WHITE)
    d.rectangle([14, 3, 14, 6], fill=WHITE)
    out.append(save(img, "widgets/window/scrollbar_h_thumb.png"))

    # arrow_up/down/left/right: スクロールバー端の矢印（10x10）
    for name, pts in [
        ("up", [(5, 1), (2, 7), (8, 7)]),
        ("down", [(5, 8), (2, 2), (8, 2)]),
        ("left", [(1, 5), (7, 2), (7, 8)]),
        ("right", [(8, 5), (2, 2), (2, 8)]),
    ]:
        img = new_canvas(10, 10)
        d = ImageDraw.Draw(img)
        # 三角を線で描く
        d.line([pts[0], pts[1]], fill=WHITE)
        d.line([pts[1], pts[2]], fill=WHITE)
        d.line([pts[2], pts[0]], fill=WHITE)
        # 中抜きを埋める（簡易）
        for fill_pt in _triangle_fill(pts):
            d.point(fill_pt, fill=WHITE)
        out.append(save(img, f"widgets/window/arrow_{name}.png"))

    return out


def _triangle_fill(pts):
    """3点からなる三角形内部のピクセル座標リストを返す（簡易スキャン）。"""
    xs = [p[0] for p in pts]
    ys = [p[1] for p in pts]
    minx, maxx = min(xs), max(xs)
    miny, maxy = min(ys), max(ys)
    out = []
    for y in range(miny, maxy + 1):
        for x in range(minx, maxx + 1):
            if _point_in_triangle((x, y), pts):
                out.append([x, y])
    return out


def _point_in_triangle(p, tri):
    def sign(a, b, c):
        return (a[0] - c[0]) * (b[1] - c[1]) - (b[0] - c[0]) * (a[1] - c[1])
    d1 = sign(p, tri[0], tri[1])
    d2 = sign(p, tri[1], tri[2])
    d3 = sign(p, tri[2], tri[0])
    has_neg = (d1 < 0) or (d2 < 0) or (d3 < 0)
    has_pos = (d1 > 0) or (d2 > 0) or (d3 > 0)
    return not (has_neg and has_pos)


# ---- ボタン類 ----------------------------------------------------------
def gen_push_button(down: bool = False):
    """プッシュボタン。4 層（face/outline/highlight/shadow）で出力。
    down=True はハイライト/シャドウを反転させる用法のため、
    ここでは face/outline は共通、hi/sh を反転位置で出力する。"""
    w, h = SIZE_PUSH_W, SIZE_PUSH_H
    suffix = "push_down" if down else "push"
    out = []

    # face: 全面べた塗り（内側 1px 縮める＝outline の内側）
    img = new_canvas(w, h)
    d = ImageDraw.Draw(img)
    draw_rect_fill(d, (1, 1, w - 1, h - 1))
    out.append(save(img, f"widgets/buttons/{suffix}_face.png"))

    # outline: 外周 1px
    img = new_canvas(w, h)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, w, h), thick=1)
    out.append(save(img, f"widgets/buttons/{suffix}_outline.png"))

    # highlight: 上辺+左辺の内側1px（左上ベベル）
    # down の場合は下辺+右辺に反転
    img = new_canvas(w, h)
    d = ImageDraw.Draw(img)
    if down:
        # 下辺・右辺
        d.rectangle([1, h - 2, w - 2, h - 2], fill=WHITE)
        d.rectangle([w - 2, 1, w - 2, h - 2], fill=WHITE)
    else:
        # 上辺・左辺
        d.rectangle([1, 1, w - 2, 1], fill=WHITE)
        d.rectangle([1, 1, 1, h - 2], fill=WHITE)
    out.append(save(img, f"widgets/buttons/{suffix}_highlight.png"))

    # shadow: 反対側の 1px
    img = new_canvas(w, h)
    d = ImageDraw.Draw(img)
    if down:
        d.rectangle([1, 1, w - 2, 1], fill=WHITE)
        d.rectangle([1, 1, 1, h - 2], fill=WHITE)
    else:
        d.rectangle([1, h - 2, w - 2, h - 2], fill=WHITE)
        d.rectangle([w - 2, 1, w - 2, h - 2], fill=WHITE)
    out.append(save(img, f"widgets/buttons/{suffix}_shadow.png"))
    return out


def gen_checkbox():
    """チェックボックス。on/off の outline のみ。"""
    out = []
    # off: 枠のみ
    img = new_canvas(SIZE_CHECK, SIZE_CHECK)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, SIZE_CHECK, SIZE_CHECK), thick=1)
    out.append(save(img, "widgets/buttons/checkbox_off_outline.png"))

    # on: 枠 + チェック印（バツではなく勾配チェック）
    img = new_canvas(SIZE_CHECK, SIZE_CHECK)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, SIZE_CHECK, SIZE_CHECK), thick=1)
    # 勾配チェックを白ピクセルで（荒い手書き）
    check_pts = [(2, 5), (3, 6), (4, 7), (5, 6), (8, 2), (8, 3), (4, 8)]
    for (cx, cy) in check_pts:
        d.point([cx, cy], fill=WHITE)
    d.line([(2, 5), (4, 7)], fill=WHITE)
    d.line([(4, 7), (8, 2)], fill=WHITE)
    out.append(save(img, "widgets/buttons/checkbox_on_outline.png"))
    return out


def gen_radio():
    """ラジオボタン。円枠。"""
    out = []
    # off: 円枠
    img = new_canvas(SIZE_RADIO, SIZE_RADIO)
    d = ImageDraw.Draw(img)
    d.ellipse([0, 0, SIZE_RADIO - 1, SIZE_RADIO - 1], outline=WHITE)
    out.append(save(img, "widgets/buttons/radio_off_outline.png"))

    # on: 円枠 + 内側に小さい黒（=実行時で染色された）円 → 白で塗る
    img = new_canvas(SIZE_RADIO, SIZE_RADIO)
    d = ImageDraw.Draw(img)
    d.ellipse([0, 0, SIZE_RADIO - 1, SIZE_RADIO - 1], outline=WHITE)
    d.ellipse([3, 3, SIZE_RADIO - 4, SIZE_RADIO - 4], fill=WHITE)
    out.append(save(img, "widgets/buttons/radio_on_outline.png"))
    return out


def gen_buttons():
    out = []
    out += gen_push_button(down=False)
    out += gen_push_button(down=True)
    out += gen_checkbox()
    out += gen_radio()
    return out


# ---- 追加ボタン類（タブ・メニュー等、後で使う） ----------------------
def gen_extras():
    """後で使うボタン類のテンプレ。"""
    out = []

    # tab_active / tab_inactive: タブ（48x18）。上辺+両側+底なし。
    for state in ("active", "inactive"):
        w, h = 48, 18
        img = new_canvas(w, h)
        d = ImageDraw.Draw(img)
        # 上辺・左辺・右辺（底は開く）
        d.rectangle([0, 0, w - 1, 0], fill=WHITE)
        d.rectangle([0, 0, 0, h - 1], fill=WHITE)
        d.rectangle([w - 1, 0, w - 1, h - 1], fill=WHITE)
        if state == "active":
            # active は下辺なしで「開いてる」表現。上辺を2pxにして強調。
            d.rectangle([0, 1, w - 1, 1], fill=WHITE)
        out.append(save(img, f"widgets/buttons/tab_{state}.png"))

    # menu_caret: メニューの右向き矢印（8x8）
    img = new_canvas(8, 8)
    d = ImageDraw.Draw(img)
    pts = [(6, 4), (2, 1), (2, 7)]
    d.line([pts[0], pts[1]], fill=WHITE)
    d.line([pts[1], pts[2]], fill=WHITE)
    d.line([pts[2], pts[0]], fill=WHITE)
    for fill_pt in _triangle_fill([(p[0], p[1]) for p in pts]):
        d.point(fill_pt, fill=WHITE)
    out.append(save(img, "widgets/buttons/menu_caret.png"))

    # separator_h: 水平区切り線（32x1）
    img = new_canvas(32, 1)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 31, 0], fill=WHITE)
    out.append(save(img, "widgets/buttons/separator_h.png"))

    # separator_v: 垂直区切り線（1x32）
    img = new_canvas(1, 32)
    d = ImageDraw.Draw(img)
    d.rectangle([0, 0, 0, 31], fill=WHITE)
    out.append(save(img, "widgets/buttons/separator_v.png"))

    return out


# ---- IME アイコン ----------------------------------------------------
def gen_ime():
    """topbar 右上に表示する IME 状態アイコン。
    直接入力 / ローマ字かな / ひらがな / カタカナ / 中文 を想定。
    16x12 の白×透明で、テキスト風の印を描く。"""
    out = []
    W, H = 20, 12

    def box_with_glyph(draw_fn, name):
        img = new_canvas(W, H)
        d = ImageDraw.Draw(img)
        # 枠
        draw_rect_outline(d, (0, 0, W, H), thick=1)
        draw_fn(d)
        out.append(save(img, f"widgets/ime/{name}.png"))

    # direct: 「A」風（直接入力）。中央に A 字
    def draw_a(d):
        # 簡易 A: 5x7
        d.line([(7, 8), (9, 2)], fill=WHITE)
        d.line([(9, 2), (11, 8)], fill=WHITE)
        d.rectangle([8, 6, 10, 6], fill=WHITE)
    box_with_glyph(draw_a, "direct")

    # romaji: 「R」風
    def draw_r(d):
        d.line([(7, 8), (7, 2)], fill=WHITE)
        d.rectangle([7, 2, 10, 2], fill=WHITE)
        d.rectangle([7, 4, 10, 4], fill=WHITE)
        d.line([(8, 5), (11, 8)], fill=WHITE)
    box_with_glyph(draw_r, "romaji")

    # hiragana: 「あ」風（小さい波+輪）
    def draw_hira(d):
        # 簡易ひらがな風ドット
        for (cx, cy) in [(7, 3), (9, 3), (11, 3), (8, 6), (10, 6), (9, 9)]:
            d.point([cx, cy], fill=WHITE)
        d.rectangle([7, 5, 11, 5], fill=WHITE)
    box_with_glyph(draw_hira, "hiragana")

    # katakana: 「ア」風（カタカナ）
    def draw_kata(d):
        d.rectangle([6, 2, 10, 2], fill=WHITE)
        d.line([(8, 2), (6, 9)], fill=WHITE)
        d.line([(8, 2), (11, 9)], fill=WHITE)
    box_with_glyph(draw_kata, "katakana")

    # zenkaku: 全角（「全」の代わりに二重枠）
    def draw_zen(d):
        draw_rect_outline(d, (4, 3, 14, 9), thick=1)
    box_with_glyph(draw_zen, "zenkaku")

    # chinese: 中文（「中」風：縦線+矩形）
    def draw_zh(d):
        d.rectangle([7, 2, 11, 9], fill=None, outline=WHITE)
        d.rectangle([9, 2, 9, 9], fill=WHITE)
    box_with_glyph(draw_zh, "chinese")

    return out


# ---- ステータスアイコン（既存ファイル名を維持して白×透明へ） ----------
def gen_status():
    """wifi_*/speaker_*/battery_* を 16x16 白×透明で再生成。
    ファイル名は既存 seashore.rs 互換を維持。"""
    out = []

    def wifi(bars: int, name: str):
        # 下から3段の弧バー。bars=0..3
        img = new_canvas(SIZE_STATUS, SIZE_STATUS)
        d = ImageDraw.Draw(img)
        # 3 段。上が広い。
        rows = [
            (2, 9, 5),   # y, x_off, width
            (5, 7, 3),
            (8, 5, 1),
        ]
        for i, (yo, xo, _) in enumerate(reversed(rows)):
            visible = i < bars
            if not visible:
                continue
            # 弧風に段々に広げる
            yy = 12 - yo
            d.rectangle([5 - (i * 2), yy, 10 + (i * 2), yy], fill=WHITE)
        out.append(save(img, f"status/{name}.png"))

    wifi(0, "wifi_none")
    wifi(1, "wifi_one")
    wifi(2, "wifi_two")
    wifi(3, "wifi_three")

    def speaker(level: str, name: str):
        # スピーカーボックス + 音波の段階
        img = new_canvas(SIZE_STATUS, SIZE_STATUS)
        d = ImageDraw.Draw(img)
        # スピーカー本体 8x8
        draw_rect_outline(d, (2, 4, 10, 12), thick=1)
        waves = {"mute": 0, "low": 1, "medium": 2, "high": 3}
        n = waves[level]
        for i in range(n):
            x = 11 + i * 2
            y0 = 4 + i      # 外側の波ほど短く（中央寄せ）
            y1 = 11 - i
            if y1 < y0:
                y0, y1 = y1, y0
            d.rectangle([x, y0, x, y1], fill=WHITE)
        out.append(save(img, f"status/{name}.png"))

    speaker("mute", "speaker_mute")
    speaker("low", "speaker_low")
    speaker("medium", "speaker_medium")
    speaker("high", "speaker_high")

    def battery(state: str, name: str):
        img = new_canvas(SIZE_STATUS, SIZE_STATUS)
        d = ImageDraw.Draw(img)
        # 本体 10x8
        draw_rect_outline(d, (2, 4, 12, 12), thick=1)
        # 端子
        d.rectangle([12, 6, 13, 9], fill=WHITE)
        fill_w = {"empty": 0, "low": 2, "medium": 5, "full": 8, "charging": 8}
        fw = fill_w[state]
        if fw > 0:
            d.rectangle([3, 5, 2 + fw, 11], fill=WHITE)
        if state == "charging":
            # 稲妻を白ピクセルで上書き
            for (cx, cy) in [(5, 5), (6, 7), (5, 8), (7, 8), (6, 10)]:
                d.point([cx, cy], fill=WHITE)
        out.append(save(img, f"status/{name}.png"))

    battery("empty", "battery_empty")
    battery("low", "battery_low")
    battery("medium", "battery_medium")
    battery("full", "battery_full")
    battery("charging", "battery_charging")

    # placeholder も白×透明の四角で更新
    img = new_canvas(SIZE_STATUS, SIZE_STATUS)
    d = ImageDraw.Draw(img)
    draw_rect_outline(d, (0, 0, SIZE_STATUS, SIZE_STATUS), thick=1)
    out.append(save(img, "status/placeholder.png"))

    return out


# ---- splash は触らない（128x32、既存維持）


# ---- メイン ------------------------------------------------------------
GROUPS = {
    "window": gen_window,
    "buttons": gen_buttons,
    "extras": gen_extras,
    "ime": gen_ime,
    "status": gen_status,
}


def main():
    ap = argparse.ArgumentParser(description="Generate Harbour OS widget PNGs")
    ap.add_argument("--window", action="store_true", help="window グループのみ")
    ap.add_argument("--buttons", action="store_true", help="buttons グループのみ")
    ap.add_argument("--extras", action="store_true", help="extras（タブ等）グループのみ")
    ap.add_argument("--ime", action="store_true", help="ime グループのみ")
    ap.add_argument("--status", action="store_true", help="status グループのみ")
    ap.add_argument("--list", action="store_true", help="出力せずファイル一覧のみ")
    args = ap.parse_args()

    if args.list:
        print("=== 出力予定ファイル ===")
        for name, fn in GROUPS.items():
            print(f"[{name}]")
            for p in fn():
                print(" ", p.relative_to(ROOT))
        return

    selected = [g for g, flag in
                [("window", args.window), ("buttons", args.buttons),
                 ("extras", args.extras), ("ime", args.ime),
                 ("status", args.status)] if flag]
    if not selected:
        selected = list(GROUPS.keys())

    total = 0
    for g in selected:
        paths = GROUPS[g]()
        for p in paths:
            print(f"  wrote {p.relative_to(ROOT)}")
        total += len(paths)
    print(f"done: {total} files")


if __name__ == "__main__":
    sys.exit(main() or 0)
