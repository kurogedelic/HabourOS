use sdl2::rect::Rect;
use sdl2::render::{Canvas, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::assets::AssetStore;
use crate::harbour::HarbourOS;
use crate::theme::{Theme, ThemeManager};

const WINDOW_WIDTH: u32 = 480;

/// Harbour OS デスクトップ（旧 Seashore）。
/// `'r` は TextureCreator / Texture の借用ライフタイムに紐付く。
pub struct Seashore<'r> {
    pub font_system: crate::font::FontSystem,
    pub theme: ThemeManager,
    pub assets: AssetStore<'r>,
}

impl<'r> Seashore<'r> {
    pub fn new(creator: &'r TextureCreator<WindowContext>) -> Result<Self, String> {
        let mut font_system = crate::font::FontSystem::new();
        // M+フォントをロード
        let font = crate::font::load_mplus_f12r();
        font_system.load_font(font);
        println!("Loaded M+ F12R font");

        // テーマをロード
        let theme =
            Theme::load_from_yaml("assets/themes/default.yaml").expect("Failed to load theme");
        println!("Loaded theme: {}", theme.name);

        let theme_manager = ThemeManager::new(theme);

        // アセットをロード（assets/images 以下の全 PNG）
        let assets = AssetStore::new(creator)?;
        println!("Loaded {} assets", assets.len());

        Ok(Self {
            font_system,
            theme: theme_manager,
            assets,
        })
    }

    pub fn update(&mut self, os: &mut HarbourOS) {
        os.update_time();
    }

    pub fn draw(&mut self, c: &mut Canvas<Window>, os: &HarbourOS) {
        // 背景
        let bg = self.theme.get_color("background");
        c.set_draw_color(bg);
        let _ = c.clear();

        // トップバー（最前面）
        self.draw_top_bar(c, os);

        // ウィンドウ（上から順に描画）
        self.draw_windows(c, os);
    }

    /// トップメニューバーを描画。戻り値はバーの高さ（下端 y 座標）。
    /// 方針: Black 背景 + Amber outline/文字/アイコン。fill は使わない。
    pub fn draw_top_bar(&mut self, c: &mut Canvas<Window>, os: &HarbourOS) -> i32 {
        let bar_height = 20i32;
        let amber = self.theme.get_color("foreground");
        let black = self.theme.get_color("background");

        // バー幅は実際の出力サイズに追従
        let (w, _) = c.output_size().unwrap_or((WINDOW_WIDTH, 0));

        // 1) バー領域の背景（fill はバー地のみ例外的に黒で塗る＝背景と同色なので実質不可視）
        let _ = c.set_draw_color(black);

        // 2) バー下端の区切り線（Amber outline 1px）
        crate::ui::line(c, 0, bar_height, w as i32 - 1, bar_height, amber);

        // 3) 時刻を左端に表示（Amber）
        os.draw_topbar_clock(c, &self.font_system, amber);

        // 4) ステータスアイコンを右詰めで表示（PNG アセット、Amber 染色）
        self.draw_status_icons(c, w as i32);

        bar_height
    }

    fn draw_status_icons(&mut self, c: &mut Canvas<Window>, bar_w: i32) {
        let bar_height = 20i32;
        let icon_tone = self.theme.get_color("foreground");
        let icon_y = (bar_height - 16) / 2;

        // 右詰め: WiFi, Speaker, Battery の順（20px 間隔）
        let spacing = 20i32;
        let battery_x = bar_w - 16 - 6;
        let speaker_x = battery_x - spacing;
        let wifi_x = speaker_x - spacing;

        let _ = self.assets.blit_tinted(
            c,
            "status/wifi_three",
            Some(Rect::new(wifi_x, icon_y, 16, 16)),
            icon_tone,
        );
        let _ = self.assets.blit_tinted(
            c,
            "status/speaker_high",
            Some(Rect::new(speaker_x, icon_y, 16, 16)),
            icon_tone,
        );
        let _ = self.assets.blit_tinted(
            c,
            "status/battery_full",
            Some(Rect::new(battery_x, icon_y, 16, 16)),
            icon_tone,
        );
    }

    /// 登録されたウィンドウを全て描画（リスト順＝下から上）。
    fn draw_windows(&mut self, c: &mut Canvas<Window>, os: &HarbourOS) {
        // 描画に必要な色を先に取り出し（borrow checker 対策）
        let outline = self.theme.get_color("widget_outline");
        let outline_dim = self.theme.get_color("widget_outline_dim");
        let face = self.theme.get_color("widget_face");
        let face_active = self.theme.get_color("widget_face_active");
        let stripe_a = self.theme.get_color("titlebar_stripe_a");
        let stripe_b = self.theme.get_color("titlebar_stripe_b");
        let title_active = self.theme.get_color("highlight");
        let title_inactive = self.theme.get_color("foreground");

        for win in os.windows.iter() {
            let border_col = if win.active { outline } else { outline_dim };
            let face_col = if win.active { face_active } else { face };
            let title_col = if win.active {
                title_active
            } else {
                title_inactive
            };

            // 1) クライアント領域（本体の地色）
            let client = win.client_rect();
            crate::ui::fill_rect(c, client, face_col);

            // 2) タイトルバー地（縞模様：stripe_a/b を 1px ずつ交互）
            let tb = win.titlebar_rect();
            for ry in 0..tb.height() as i32 {
                let tone = if ry % 2 == 0 { stripe_a } else { stripe_b };
                crate::ui::fill_rect(c, Rect::new(tb.x(), tb.y() + ry, tb.width(), 1), tone);
            }

            // 3) 外枠（ベベル：全体を outline、内側ハイライトは省略してシンプルに）
            crate::ui::draw_rect_thick(c, Rect::new(win.x, win.y, win.w, win.h), border_col);

            // 4) タイトルバーとクライアントの境界線
            let sep_y = win.y + win.titlebar_h as i32;
            crate::ui::line(c, win.x, sep_y, win.x + win.w as i32 - 1, sep_y, border_col);

            // 5) クローズボックス（PNG、border 染色）
            let cb = win.closebox_rect();
            let _ =
                self.assets
                    .blit_tinted(c, "widgets/window/closebox_outline", Some(cb), border_col);

            // 6) タイトルテキスト（closebox の右）
            self.font_system.draw_text(
                c,
                win.title_x(),
                win.y + (win.titlebar_h as i32 - 13) / 2,
                &win.title,
                title_col,
            );
        }
    }
}
