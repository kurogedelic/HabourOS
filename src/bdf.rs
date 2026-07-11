//! BDF フォントの実行時ローダ。
//!
//! `include!` で Rust ソースに埋め込む方式は、1MB 超の日本語フォントでは
//! コンパイル時間が爆発するため、実行時に BDF を直接パースして使う。
//!
//! 対応: BDF 2.1、JIS X 0208 / Unicode 両方の ENCODING 値を格納。
//! M+ F12R (Latin-1, 6x13) と M+ J12R (JIS X 0208, 12x13) の両方を想定。

use std::collections::HashMap;

/// 1グリフのビットマップ。各行1つの u16（上位ビットから描画）。
#[derive(Clone)]
pub struct BdfGlyph {
    /// ピクセル幅（DWIDTH 相当）。
    pub width: u32,
    /// ピクセル高（BBX height）。
    pub height: u32,
    /// ビットマップ行データ。MSB first。
    pub rows: Vec<u16>,
}

/// BDF フォント。
pub struct BdfFont {
    /// 全体のピクセル幅（固定幅フォントの場合）。
    pub width: u32,
    /// 全体のピクセル高。
    pub height: u32,
    /// 文字コード → グリフ。
    glyphs: HashMap<u32, BdfGlyph>,
}

impl BdfFont {
    /// BDF ファイルをパースしてロード。
    pub fn load(path: &str) -> Result<Self, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("failed to read {}: {}", path, e))?;
        Self::parse(&content)
    }

    fn parse(content: &str) -> Result<Self, String> {
        let mut font = BdfFont {
            width: 0,
            height: 0,
            glyphs: HashMap::new(),
        };

        // FONTBOUNDINGBOX w h ...
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("FONTBOUNDINGBOX ") {
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() >= 2 {
                    font.width = parts[0].parse().unwrap_or(0);
                    font.height = parts[1].parse().unwrap_or(0);
                }
                break;
            }
        }

        // グリフを順にパース
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim_end();
            if line == "STARTCHAR" || line.starts_with("STARTCHAR ") {
                // ENCODING を探す
                let mut encoding: Option<u32> = None;
                let mut bbx_w = font.width;
                let mut bbx_h = font.height;
                let mut bitmap_rows: Vec<u16> = Vec::new();
                let mut in_bitmap = false;

                i += 1;
                while i < lines.len() {
                    let l = lines[i].trim_end();
                    if let Some(rest) = l.strip_prefix("ENCODING ") {
                        encoding = rest.split_whitespace().next().and_then(|s| s.parse().ok());
                    } else if let Some(rest) = l.strip_prefix("BBX ") {
                        let parts: Vec<&str> = rest.split_whitespace().collect();
                        if parts.len() >= 2 {
                            bbx_w = parts[0].parse().unwrap_or(bbx_w);
                            bbx_h = parts[1].parse().unwrap_or(bbx_h);
                        }
                    } else if l == "BITMAP" {
                        in_bitmap = true;
                    } else if l == "ENDCHAR" {
                        break;
                    } else if in_bitmap {
                        // 16進数の行
                        if let Ok(v) = u16::from_str_radix(l.trim(), 16) {
                            bitmap_rows.push(v);
                        }
                    }
                    i += 1;
                }

                if let Some(code) = encoding {
                    // ENCODING が -1 (default char) は無視
                    if code != u32::MAX {
                        font.glyphs.insert(
                            code,
                            BdfGlyph {
                                width: bbx_w,
                                height: bbx_h,
                                rows: bitmap_rows,
                            },
                        );
                    }
                }
            }
            i += 1;
        }

        Ok(font)
    }

    /// グリフ数。
    pub fn len(&self) -> usize {
        self.glyphs.len()
    }

    /// JIS X 0208 コードでグリフ取得。
    pub fn glyph_jis(&self, jis_code: u32) -> Option<&BdfGlyph> {
        self.glyphs.get(&jis_code)
    }

    /// Unicode からグリフ取得。
    /// ※ M+ の JIS X 0208 BDF は ENCODING が JIS コードなので、
    ///   Unicode → JIS 変換テーブルが必要。これは別途用意する。
    pub fn glyph_unicode(&self, code: u32) -> Option<&BdfGlyph> {
        self.glyphs.get(&code)
    }
}
