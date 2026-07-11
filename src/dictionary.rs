//! Dictionary - 小さな内蔵辞書アプリ。

use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub word: &'static str,
    pub reading: &'static str,
    pub meaning: &'static str,
}

const ENTRIES: &[Entry] = &[
    Entry {
        word: "amber",
        reading: "amber",
        meaning: "琥珀色 / 黄褐色",
    },
    Entry {
        word: "harbour",
        reading: "harbour",
        meaning: "港 / 避難所 / 情報を係留する場所",
    },
    Entry {
        word: "clock",
        reading: "clock",
        meaning: "時計",
    },
    Entry {
        word: "journal",
        reading: "journal",
        meaning: "日誌",
    },
    Entry {
        word: "dictionary",
        reading: "dictionary",
        meaning: "辞書",
    },
];

pub fn lookup(query: &str) -> Option<Entry> {
    let query = query.trim().to_ascii_lowercase();
    ENTRIES.iter().copied().find(|entry| entry.word == query)
}

pub struct DictionaryState {
    query: String,
    result: Option<Entry>,
    message: String,
}

impl DictionaryState {
    pub fn new() -> Self {
        DictionaryState {
            query: String::new(),
            result: None,
            message: "Type a word".to_string(),
        }
    }

    pub fn input_key(&mut self, key: Keycode) {
        match key {
            Keycode::Backspace => {
                self.query.pop();
            }
            Keycode::Return | Keycode::KpEnter => {
                self.result = lookup(&self.query);
                self.message = if self.result.is_some() {
                    "Found".to_string()
                } else {
                    "No entry".to_string()
                };
            }
            _ => {}
        }
    }

    pub fn input_char(&mut self, ch: char) {
        if !ch.is_control() && !ch.is_whitespace() {
            self.query.push(ch);
        }
    }

    pub fn draw(
        &self,
        c: &mut Canvas<Window>,
        font: &FontSystem,
        theme: &ThemeManager,
        x: i32,
        y: i32,
        w: u32,
        _h: u32,
    ) {
        let amber = theme.get_color("foreground");
        let muted = theme.get_color("muted");

        font.draw_text(c, x + 6, y + 8, "Search", muted);
        let input = format!("> {}", self.query);
        font.draw_text(c, x + 6, y + 24, &input, amber);
        crate::ui::line(c, x + 6, y + 39, x + w as i32 - 7, y + 39, amber);

        match self.result {
            Some(entry) => {
                font.draw_text(c, x + 6, y + 56, "Headword", muted);
                font.draw_text(c, x + 70, y + 56, entry.word, amber);
                font.draw_text(c, x + 6, y + 74, "Reading", muted);
                font.draw_text(c, x + 70, y + 74, entry.reading, amber);
                crate::ui::line(c, x + 6, y + 92, x + w as i32 - 7, y + 92, amber);
                font.draw_text(c, x + 6, y + 108, "Meaning", muted);
                draw_wrapped(
                    c,
                    font,
                    amber,
                    x + 70,
                    y + 108,
                    w as i32 - 76,
                    entry.meaning,
                );
            }
            None => {
                font.draw_text(c, x + 6, y + 56, &self.message, muted);
            }
        }
    }
}

fn draw_wrapped(
    c: &mut Canvas<Window>,
    font: &FontSystem,
    color: sdl2::pixels::Color,
    x: i32,
    y: i32,
    max_w: i32,
    text: &str,
) {
    let mut line = String::new();
    let mut cy = y;
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_string()
        } else {
            format!("{} {}", line, word)
        };
        if font.measure_text(&candidate) > max_w && !line.is_empty() {
            font.draw_text(c, x, cy, &line, color);
            line = word.to_string();
            cy += 13;
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        font.draw_text(c, x, cy, &line, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_returns_amber_entry() {
        let entry = lookup("amber").unwrap();
        assert_eq!(entry.word, "amber");
        assert_eq!(entry.reading, "amber");
        assert!(entry.meaning.contains("琥珀色"));
    }
}
