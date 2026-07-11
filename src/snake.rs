//! Snake - Harbour OS 向けの最小ゲームアプリ。

use std::time::{Duration, Instant};

use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::font::FontSystem;
use crate::theme::ThemeManager;
use crate::ui;

const GRID_W: i32 = 24;
const GRID_H: i32 = 16;
const STEP_MS: u64 = 140;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub struct SnakeState {
    snake: Vec<Pos>,
    dir: Dir,
    next_dir: Dir,
    food: Pos,
    score: u32,
    game_over: bool,
    last_step: Instant,
}

impl SnakeState {
    pub fn new() -> Self {
        let mut state = SnakeState {
            snake: vec![Pos { x: 8, y: 8 }, Pos { x: 7, y: 8 }, Pos { x: 6, y: 8 }],
            dir: Dir::Right,
            next_dir: Dir::Right,
            food: Pos { x: 15, y: 8 },
            score: 0,
            game_over: false,
            last_step: Instant::now(),
        };
        state.place_food();
        state
    }

    pub fn input_key(&mut self, key: Keycode) {
        match key {
            Keycode::Up if self.dir != Dir::Down => self.next_dir = Dir::Up,
            Keycode::Down if self.dir != Dir::Up => self.next_dir = Dir::Down,
            Keycode::Left if self.dir != Dir::Right => self.next_dir = Dir::Left,
            Keycode::Right if self.dir != Dir::Left => self.next_dir = Dir::Right,
            Keycode::Space | Keycode::R => {
                if self.game_over {
                    *self = Self::new();
                }
            }
            _ => {}
        }
    }

    pub fn update(&mut self) {
        if self.game_over || self.last_step.elapsed() < Duration::from_millis(STEP_MS) {
            return;
        }
        self.last_step = Instant::now();
        self.step();
    }

    fn step(&mut self) {
        self.dir = self.next_dir;
        let head = self.snake[0];
        let new_head = match self.dir {
            Dir::Up => Pos {
                x: head.x,
                y: head.y - 1,
            },
            Dir::Down => Pos {
                x: head.x,
                y: head.y + 1,
            },
            Dir::Left => Pos {
                x: head.x - 1,
                y: head.y,
            },
            Dir::Right => Pos {
                x: head.x + 1,
                y: head.y,
            },
        };

        if new_head.x < 0
            || new_head.y < 0
            || new_head.x >= GRID_W
            || new_head.y >= GRID_H
            || self.snake.contains(&new_head)
        {
            self.game_over = true;
            return;
        }

        self.snake.insert(0, new_head);
        if new_head == self.food {
            self.score += 1;
            self.place_food();
        } else {
            self.snake.pop();
        }
    }

    fn place_food(&mut self) {
        let mut seed = (self.score as i32 * 7 + self.snake[0].x * 3 + self.snake[0].y * 5 + 11)
            .rem_euclid(GRID_W * GRID_H);
        for _ in 0..GRID_W * GRID_H {
            let candidate = Pos {
                x: seed % GRID_W,
                y: seed / GRID_W,
            };
            if !self.snake.contains(&candidate) {
                self.food = candidate;
                return;
            }
            seed = (seed + 17).rem_euclid(GRID_W * GRID_H);
        }
        self.game_over = true;
    }

    pub fn footer_text(&self) -> String {
        if self.game_over {
            format!("Game Over   Score {}", self.score)
        } else {
            format!("Score {}", self.score)
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
        h: u32,
    ) {
        let amber = theme.get_color("foreground");
        let muted = theme.get_color("muted");
        let cell = ((w as i32 - 16) / GRID_W)
            .min((h as i32 - 28) / GRID_H)
            .clamp(4, 18);
        let board_w = GRID_W * cell;
        let board_h = GRID_H * cell;
        let bx = x + (w as i32 - board_w) / 2;
        let by = y + ((h as i32 - board_h) / 2).max(18);

        ui::draw_rect(
            c,
            Rect::new(bx - 2, by - 2, (board_w + 4) as u32, (board_h + 4) as u32),
            amber,
        );
        ui::draw_rect(
            c,
            Rect::new(bx - 4, by - 4, (board_w + 8) as u32, (board_h + 8) as u32),
            muted,
        );

        draw_cell(c, self.food, bx, by, cell, amber, false);
        for (i, pos) in self.snake.iter().enumerate() {
            draw_cell(c, *pos, bx, by, cell, amber, i == 0);
        }

        if self.game_over {
            let msg = "GAME OVER";
            let msg_w = font.measure_text(msg);
            font.draw_text(c, x + (w as i32 - msg_w) / 2, y + 8, msg, amber);
        }
    }
}

fn draw_cell(
    c: &mut Canvas<Window>,
    pos: Pos,
    bx: i32,
    by: i32,
    cell: i32,
    color: Color,
    head: bool,
) {
    let r = Rect::new(
        bx + pos.x * cell + 1,
        by + pos.y * cell + 1,
        (cell - 2).max(1) as u32,
        (cell - 2).max(1) as u32,
    );
    if head {
        c.set_draw_color(color);
        let _ = c.fill_rect(r);
    } else {
        ui::draw_rect(c, r, color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_moves_right() {
        let mut snake = SnakeState::new();
        let x = snake.snake[0].x;
        snake.step();
        assert_eq!(snake.snake[0].x, x + 1);
    }
}
