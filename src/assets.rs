//! AssetStore - PNG アセットの一括ロードと、テーマ色による染色描画。
//!
//! 設計:
//! - 全 PNG は「白×透明」で描かれている（tools/gen_widgets.py が出力）。
//! - `Texture::set_color_mod(tone)` で白ピクセルを tone 色に置換して描画する。
//!   ※ set_color_mod は乗算なので、白(255)以外は暗くなる。黒基準 PNG は
//!      色が乗らないため、アセットは必ず白基準でなければならない。
//! - Classic ベベルは「複数 PNG を重ね描き」で表現する（blit_layers）。
//!
//! ライフタイム:
//! - `Texture<'r>` は `TextureCreator` の借用に紐付くため、`AssetStore<'r>` は
//!   `&'r TextureCreator` を借用して保持する。呼び出し元（main.rs）が
//!   TextureCreator の所有権を持つ。

use std::collections::HashMap;

use sdl2::image::LoadSurface;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, ScaleMode, Texture, TextureCreator, TextureQuery};
use sdl2::surface::Surface;
use sdl2::video::{Window, WindowContext};

/// アセットルート（実行時のカレントディレクトリ基準の相対パス）。
const ASSET_ROOT: &str = "assets/images";

/// PNG テクスチャを名前（相対パスから拡張子を除いたもの）で引けるストア。
pub struct AssetStore<'r> {
    textures: HashMap<String, Texture<'r>>,
}

impl<'r> AssetStore<'r> {
    /// 指定ディレクトリ以下の PNG を再帰的にロードする。
    /// 失敗したファイルは警告を出してスキップ（続行）。
    pub fn load_dir(creator: &'r TextureCreator<WindowContext>, dir: &str) -> Result<Self, String> {
        let mut textures: HashMap<String, Texture<'r>> = HashMap::new();

        let root = std::path::Path::new(dir);
        if !root.exists() {
            eprintln!("[assets] directory not found: {}", dir);
            return Ok(AssetStore { textures });
        }

        let entries = collect_pngs(root);
        for path in entries {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/");
            match load_texture(creator, &path) {
                Ok(tex) => {
                    textures.insert(rel, tex);
                }
                Err(e) => {
                    eprintln!("[assets] failed to load {}: {}", path.display(), e);
                }
            }
        }

        println!("[assets] loaded {} textures from {}", textures.len(), dir);
        Ok(AssetStore { textures })
    }

    /// `assets/images` を既定のルートとしてロードする convenience。
    pub fn new(creator: &'r TextureCreator<WindowContext>) -> Result<Self, String> {
        Self::load_dir(creator, ASSET_ROOT)
    }

    /// キーからテクスチャを取得。存在しなければ None。
    pub fn get(&self, key: &str) -> Option<&Texture<'r>> {
        self.textures.get(key)
    }

    /// ロード済みテクスチャ数。
    pub fn len(&self) -> usize {
        self.textures.len()
    }

    /// テクスチャのサイズを取得。
    pub fn size(&self, key: &str) -> Option<(u32, u32)> {
        self.textures.get(key).map(|t| {
            let q = t.query();
            (q.width, q.height)
        })
    }

    /// 単色染色して描画。
    /// - `key`: アセットキー（例: "status/wifi_three"）
    /// - `dst`: 描画先矩形（None ならテクスチャ本来のサイズで左上原点）
    /// - `tone`: 染色する色
    pub fn blit_tinted(
        &mut self,
        canvas: &mut Canvas<Window>,
        key: &str,
        dst: Option<Rect>,
        tone: Color,
    ) -> Result<(), String> {
        let tex = self
            .textures
            .get_mut(key)
            .ok_or_else(|| format!("asset not found: {}", key))?;
        tex.set_color_mod(tone.r, tone.g, tone.b);
        let dst_rect = match dst {
            Some(r) => Some(r),
            None => {
                let TextureQuery { width, height, .. } = tex.query();
                Some(Rect::new(0, 0, width, height))
            }
        };
        canvas.copy(tex, None, dst_rect)
    }

    /// 複数のレイヤー PNG を同位置に重ね描き（Classic ベベル用）。
    /// - `keys`: 下から順に描画するアセットキー
    /// - `tones`: 各レイヤーの染色色（keys と同長）
    /// - `dst`: 描画先矩形（全レイヤー共通）
    pub fn blit_layers(
        &mut self,
        canvas: &mut Canvas<Window>,
        keys: &[&str],
        tones: &[Color],
        dst: Option<Rect>,
    ) -> Result<(), String> {
        if keys.len() != tones.len() {
            return Err("keys and tones length mismatch".to_string());
        }
        if keys.is_empty() {
            return Ok(());
        }
        // 最初のレイヤーで dst の実サイズを確定
        let dst_rect = match dst {
            Some(r) => r,
            None => {
                let first = self
                    .textures
                    .get(keys[0])
                    .ok_or_else(|| format!("asset not found: {}", keys[0]))?;
                let q = first.query();
                Rect::new(0, 0, q.width, q.height)
            }
        };
        for (k, tone) in keys.iter().zip(tones.iter()) {
            let tex = self
                .textures
                .get_mut(*k)
                .ok_or_else(|| format!("asset not found: {}", k))?;
            tex.set_color_mod(tone.r, tone.g, tone.b);
            canvas.copy(tex, None, Some(dst_rect))?;
        }
        Ok(())
    }
}

/// PNG をロードして Texture を作る。
fn load_texture<'r>(
    creator: &'r TextureCreator<WindowContext>,
    path: &std::path::Path,
) -> Result<Texture<'r>, String> {
    let surface = Surface::from_file(path).map_err(|e| e.to_string())?;
    let mut texture = creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    texture.set_scale_mode(ScaleMode::Nearest);
    Ok(texture)
}

/// 指定ディレクトリ以下の .png を再帰的に集める（ソート済み）。
fn collect_pngs(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    collect_pngs_inner(dir, &mut out);
    out.sort();
    out
}

fn collect_pngs_inner(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // 隠しディレクトリ（.DS_Store 等）は中身を見ない
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            collect_pngs_inner(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("png") {
            out.push(path);
        }
    }
}
