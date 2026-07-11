use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

/// A Unicode bitmap font with sparse character support
pub struct UnicodeFont {
    pub width: u8,
    pub height: u8,
    // First character code in the range
    first_char: u32,
    // Number of character codes in the range
    char_count: u32,
    // Index array: maps character code to bitmap data offset (or 0xFFFFFFFF if not found)
    char_index: &'static [u32],
    // Bitmap data: all character bitmaps concatenated
    bitmap_data: &'static [u16],
}

impl UnicodeFont {
    /// Create a new Unicode font from indexed data.
    pub const fn new(
        width: u8,
        height: u8,
        first_char: u32,
        char_count: u32,
        char_index: &'static [u32],
        bitmap_data: &'static [u16],
    ) -> Self {
        UnicodeFont {
            width,
            height,
            first_char,
            char_count,
            char_index,
            bitmap_data,
        }
    }

    /// Get the bitmap data for a character, or None if the character is not in the font.
    pub fn glyph_data(&self, ch: char) -> Option<&'static [u16]> {
        let code = ch as u32;
        if code < self.first_char || code >= self.first_char + self.char_count {
            return None;
        }

        let index = (code - self.first_char) as usize;
        if index >= self.char_index.len() {
            return None;
        }

        let data_offset = self.char_index[index] as usize;
        if data_offset == 0xFFFFFFFF {
            return None; // Character not in font
        }

        // Calculate the end offset (next character or end of data)
        let mut next_offset = data_offset;
        for i in (index + 1)..self.char_index.len() {
            let next_idx = self.char_index[i] as usize;
            if next_idx != 0xFFFFFFFF {
                next_offset = next_idx;
                break;
            }
        }
        if next_offset == data_offset {
            next_offset = self.bitmap_data.len();
        }

        Some(&self.bitmap_data[data_offset..next_offset])
    }

    /// Measure the width of a string in pixels.
    pub fn measure_text(&self, text: &str) -> i32 {
        let mut width = 0;
        for ch in text.chars() {
            if self.glyph_data(ch).is_some() {
                width += self.width as i32;
            } else {
                // For missing characters, use the font's width
                width += self.width as i32;
            }
        }
        width
    }

    /// Draw text at the given position using the canvas.
    pub fn draw_text(&self, c: &mut Canvas<Window>, x: i32, y: i32, text: &str, color: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            if let Some(data) = self.glyph_data(ch) {
                self.draw_glyph(c, cursor_x, y, data, color);
                cursor_x += self.width as i32;
            } else {
                // For missing characters, draw a placeholder box
                self.draw_missing(c, cursor_x, y, color);
                cursor_x += self.width as i32;
            }
        }
    }

    fn draw_glyph(&self, c: &mut Canvas<Window>, x: i32, y: i32, data: &[u16], color: Color) {
        // M+ font data is stored as u16 but contains 8-bit bitmap data
        // Each row is one u16 value (but only lower 8 bits are used for 6px wide fonts)
        for (row, word) in data.iter().enumerate() {
            let byte = (*word & 0xFF) as u8; // Extract lower 8 bits

            // For 6px wide fonts, bits 7-2 represent the 6 pixels (MSB first)
            for col in 0..self.width {
                let bit = (byte >> (7 - col)) & 1;
                if bit == 1 {
                    c.set_draw_color(color);
                    let _ = c.fill_rect(Rect::new(x + col as i32, y + row as i32, 1, 1));
                }
            }
        }
    }

    fn draw_missing(&self, c: &mut Canvas<Window>, x: i32, y: i32, color: Color) {
        // Draw a small question mark or box for missing characters
        let w = self.width as i32;
        let h = self.height as i32;

        // Draw a box outline
        c.set_draw_color(color);
        let _ = c.fill_rect(Rect::new(x, y, w as u32, 1));
        let _ = c.fill_rect(Rect::new(x, y + h - 1, w as u32, 1));
        let _ = c.fill_rect(Rect::new(x, y, 1, h as u32));
        let _ = c.fill_rect(Rect::new(x + w - 1, y, 1, h as u32));
    }
}

pub struct FontSystem {
    pub font: Option<UnicodeFont>,
    /// 日本語 BDF（JIS X 0208）。Latin-1 にない文字のフォールバック。
    pub jis_font: Option<crate::bdf::BdfFont>,
    /// 日本語フォントのピクセル幅（M+ J12R = 12）。
    pub jis_width: u32,
    /// 日本語フォントのピクセル高（M+ J12R = 13）。
    pub jis_height: u32,
}

impl FontSystem {
    pub fn new() -> Self {
        FontSystem {
            font: None,
            jis_font: None,
            jis_width: 12,
            jis_height: 13,
        }
    }

    pub fn load_font(&mut self, font: UnicodeFont) {
        self.font = Some(font);
    }

    /// 日本語 BDF をロード。パスは fonts/ 以下の BDF。
    pub fn load_jis_font(&mut self, path: &str) -> Result<(), String> {
        let bdf = crate::bdf::BdfFont::load(path)?;
        println!("[font] loaded JIS BDF: {} ({} glyphs)", path, bdf.len());
        self.jis_font = Some(bdf);
        Ok(())
    }

    /// 1文字の表示幅を返す。日本語（JIS X 0208 対象）は jis_width、
    /// それ以外は Latin-1 フォント幅。
    pub fn char_width(&self, ch: char) -> i32 {
        if crate::jis::unicode_to_jis(ch).is_some() && self.jis_font.is_some() {
            self.jis_width as i32
        } else {
            self.font.as_ref().map(|f| f.width as i32).unwrap_or(6)
        }
    }

    pub fn draw_text(&self, c: &mut Canvas<Window>, x: i32, y: i32, text: &str, color: Color) {
        let mut cursor_x = x;
        for ch in text.chars() {
            // まず日本語フォントで描けるか試す
            if let Some(jis_code) = crate::jis::unicode_to_jis(ch) {
                if let Some(ref jf) = self.jis_font {
                    if let Some(glyph) = jf.glyph_jis(jis_code) {
                        self.draw_jis_glyph(c, cursor_x, y, glyph, color);
                        cursor_x += glyph.width as i32;
                        continue;
                    }
                }
            }
            // フォールバック: Latin-1 フォント
            if let Some(ref font) = self.font {
                font.draw_text(c, cursor_x, y, &ch.to_string(), color);
                cursor_x += font.width as i32;
            } else {
                cursor_x += 6;
            }
        }
    }

    /// JIS グリフを描画。各行のビットマップ（MSB first）を Ambe ドットで。
    fn draw_jis_glyph(
        &self,
        c: &mut Canvas<Window>,
        x: i32,
        y: i32,
        glyph: &crate::bdf::BdfGlyph,
        color: Color,
    ) {
        c.set_draw_color(color);
        let w = glyph.width;
        for (row, word) in glyph.rows.iter().enumerate() {
            for col in 0..w {
                // 各行は u16。MSB(15) から描画。幅12なら bit15..bit4 を使う。
                let bit = (word >> (15 - col)) & 1;
                if bit == 1 {
                    let _ = c.fill_rect(Rect::new(x + col as i32, y + row as i32, 1, 1));
                }
            }
        }
    }

    pub fn draw_text_bold(&self, c: &mut Canvas<Window>, x: i32, y: i32, text: &str, color: Color) {
        // 太字効果: 1pxずらして2回描画
        self.draw_text(c, x, y, text, color);
        self.draw_text(c, x + 1, y, text, color);
    }

    pub fn measure_text(&self, text: &str) -> i32 {
        let mut width = 0;
        for ch in text.chars() {
            width += self.char_width(ch);
        }
        width
    }
}

// Include the M+ font
mod mplus_f12r {
    include!("mplus_f12r.rs");
}

pub fn load_mplus_f12r() -> UnicodeFont {
    mplus_f12r::MPLUS_12X13_FONT
}
