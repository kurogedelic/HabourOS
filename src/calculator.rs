//! Calculator - キーボード入力だけで使う最小計算機。

use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;

pub struct CalculatorState {
    input: String,
    result: String,
}

impl CalculatorState {
    pub fn new() -> Self {
        CalculatorState {
            input: String::new(),
            result: String::new(),
        }
    }

    pub fn input_key(&mut self, key: Keycode) {
        match key {
            Keycode::Backspace => {
                self.input.pop();
            }
            Keycode::C => {
                self.input.clear();
                self.result.clear();
            }
            Keycode::Return | Keycode::KpEnter => {
                self.result = evaluate(&self.input).unwrap_or_else(|e| format!("Error: {}", e));
            }
            _ => {}
        }
    }

    pub fn input_char(&mut self, ch: char) {
        if ch.is_ascii_digit() || matches!(ch, '+' | '-' | '*' | '/' | '.') {
            self.input.push(ch);
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

        font.draw_text(c, x + 6, y + 8, "Expression", muted);
        let input = if self.input.is_empty() {
            ">".to_string()
        } else {
            format!("> {}", self.input)
        };
        font.draw_text(c, x + 6, y + 26, &input, amber);
        crate::ui::line(c, x + 6, y + 43, x + w as i32 - 7, y + 43, amber);
        font.draw_text(c, x + 6, y + 62, "Result", muted);
        if !self.result.is_empty() {
            font.draw_text(c, x + 6, y + 80, &self.result, amber);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Token {
    Num(f64),
    Op(char),
}

pub fn evaluate(input: &str) -> Result<String, String> {
    let tokens = tokenize(input)?;
    if tokens.is_empty() {
        return Err("empty".to_string());
    }
    let value = eval_tokens(tokens)?;
    if !value.is_finite() {
        return Err("invalid number".to_string());
    }
    Ok(format_number(value))
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    let mut expect_number = true;

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }

        if ch.is_ascii_digit() || ch == '.' || (ch == '-' && expect_number) {
            let mut s = String::new();
            if ch == '-' {
                s.push(ch);
                chars.next();
            }
            while let Some(&c) = chars.peek() {
                if c.is_ascii_digit() || c == '.' {
                    s.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            let n = s.parse::<f64>().map_err(|_| "bad number".to_string())?;
            tokens.push(Token::Num(n));
            expect_number = false;
            continue;
        }

        if matches!(ch, '+' | '-' | '*' | '/') {
            tokens.push(Token::Op(ch));
            chars.next();
            expect_number = true;
            continue;
        }

        return Err("bad character".to_string());
    }

    Ok(tokens)
}

fn eval_tokens(mut tokens: Vec<Token>) -> Result<f64, String> {
    reduce_ops(&mut tokens, &['*', '/'])?;
    reduce_ops(&mut tokens, &['+', '-'])?;
    match tokens.as_slice() {
        [Token::Num(n)] => Ok(*n),
        _ => Err("incomplete".to_string()),
    }
}

fn reduce_ops(tokens: &mut Vec<Token>, ops: &[char]) -> Result<(), String> {
    let mut i = 0;
    while i < tokens.len() {
        let is_target = matches!(tokens.get(i), Some(Token::Op(op)) if ops.contains(op));
        if !is_target {
            i += 1;
            continue;
        }

        if i == 0 || i + 1 >= tokens.len() {
            return Err("incomplete".to_string());
        }
        let Token::Num(left) = tokens[i - 1] else {
            return Err("missing left".to_string());
        };
        let Token::Op(op) = tokens[i] else {
            return Err("missing op".to_string());
        };
        let Token::Num(right) = tokens[i + 1] else {
            return Err("missing right".to_string());
        };

        let value = match op {
            '+' => left + right,
            '-' => left - right,
            '*' => left * right,
            '/' => {
                if right == 0.0 {
                    return Err("divide by zero".to_string());
                }
                left / right
            }
            _ => return Err("bad op".to_string()),
        };

        tokens.splice(i - 1..=i + 1, [Token::Num(value)]);
        i = i.saturating_sub(1);
    }
    Ok(())
}

fn format_number(n: f64) -> String {
    if (n.fract()).abs() < 0.0000000001 {
        format!("{}", n as i64)
    } else {
        let s = format!("{:.8}", n);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_addition() {
        assert_eq!(evaluate("1+2").unwrap(), "3");
    }

    #[test]
    fn respects_operator_precedence() {
        assert_eq!(evaluate("1+2*3").unwrap(), "7");
    }

    #[test]
    fn handles_divide_by_zero() {
        assert!(evaluate("1/0").is_err());
    }

    #[test]
    fn rejects_single_dot() {
        assert!(evaluate(".").is_err());
    }

    #[test]
    fn rejects_consecutive_operators() {
        assert!(evaluate("1++2").is_err());
    }

    #[test]
    fn rejects_empty_input() {
        assert!(evaluate("").is_err());
    }
}
