extern crate alloc;

#[cfg(target_arch = "wasm32")]
use lol_alloc::{FreeListAllocator, LockedAllocator};

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOCATOR: LockedAllocator<FreeListAllocator> =
    LockedAllocator::new(FreeListAllocator::new());

use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/snake.mjs")]
extern "C" {
    fn canvas_set_fill_style(color: u32);
    fn canvas_fill_rect(x: usize, y: usize, width: usize, height: usize);
    fn canvas_fill();
    fn snake_score_changed(s: i32);
    fn snake_step_period_updated(period: i32);
    fn snake_game_over();
    fn js_random(max: usize) -> i32;
}

const COLOR_BACKGROUND: u32 = 0x00000000;
const COLOR_SNAKE: u32 = 0x00ff00;
const COLOR_APPLE: u32 = 0xff0000;
const KEY_CODE_ARROW_UP: u32 = 0;
const KEY_CODE_ARROW_DOWN: u32 = 1;
const KEY_CODE_ARROW_LEFT: u32 = 2;
const KEY_CODE_ARROW_RIGHT: u32 = 3;
const CELL_SIZE: usize = 10;
const GRID_WIDTH: usize = 40;
const GRID_HEIGHT: usize = 40;

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

fn direction_is_opposite(dir: Direction, other: Direction) -> bool {
    return dir == Direction::Up && other == Direction::Down
        || dir == Direction::Down && other == Direction::Up
        || dir == Direction::Left && other == Direction::Right
        || dir == Direction::Right && other == Direction::Left;
}

#[derive(Clone, Copy, PartialEq)]
struct Position {
    x: i32,
    y: i32,
}

fn position_moved(mut pos: Position, direction: Direction) -> Position {
    match direction {
        Direction::Up => pos.y -= 1,
        Direction::Down => pos.y += 1,
        Direction::Left => pos.x -= 1,
        Direction::Right => pos.x += 1,
    }
    pos
}

struct Snake {
    segments: [Position; GRID_WIDTH * GRID_HEIGHT],
    length: usize,
    head_index: usize,
    direction: Direction,
}

impl Snake {
    fn head_position(self: &Snake) -> Position {
        return self.segments[self.head_index];
    }

    fn next_head_position(self: &Snake) -> Position {
        return position_moved(self.segments[self.head_index], self.direction);
    }

    fn eats_himself(self: &Snake) -> bool {
        for i in 0..self.length {
            if i == self.head_index {
                continue;
            }
            if self.head_position() == self.segments[i] {
                return true;
            }
        }
        false
    }

    fn is_out_of_bounds(self: &Snake, width: usize, height: usize) -> bool {
        let head_position = self.head_position();
        head_position.x < 0
            || head_position.x >= width as i32
            || head_position.y < 0
            || head_position.y >= height as i32
    }

    fn move_ahead(self: &mut Snake) {
        let next_head_position = self.next_head_position();
        if self.head_index == self.length - 1 {
            self.head_index = 0;
        } else {
            self.head_index += 1;
        }
        self.segments[self.head_index] = next_head_position;
    }

    fn grow(self: &mut Snake) {
        let next_head_position = self.next_head_position();
        if self.head_index == self.length {
            self.segments[self.length] = next_head_position;
        } else {
            for i in (self.head_index..self.length).rev() {
                self.segments[i + 1] = self.segments[i];
            }
            self.segments[self.head_index + 1] = next_head_position;
        }
        self.length += 1;
    }
}

fn paint_background() {
    canvas_set_fill_style(COLOR_BACKGROUND);
    canvas_fill_rect(0, 0, GRID_WIDTH * CELL_SIZE, GRID_HEIGHT * CELL_SIZE);
}

fn paint_snake(s: &Snake) {
    canvas_set_fill_style(COLOR_SNAKE);
    for i in 0..s.length {
        let segment = s.segments[i];
        canvas_fill_rect(
            segment.x as usize * CELL_SIZE,
            segment.y as usize * CELL_SIZE,
            CELL_SIZE,
            CELL_SIZE,
        );
    }
}

fn paint_apple(apple: Position) {
    canvas_set_fill_style(COLOR_APPLE);
    canvas_fill_rect(
        apple.x as usize * CELL_SIZE,
        apple.y as usize * CELL_SIZE,
        CELL_SIZE,
        CELL_SIZE,
    );
}

#[wasm_bindgen]
pub struct GameState {
    snake: Snake,
    apple: Position,
    step_period: i32,
    score: i32,
    next_reward: i32,
}

#[wasm_bindgen]
impl GameState {
    #[wasm_bindgen(constructor)]
    pub fn new() -> GameState {
        let mut game_state = GameState {
            snake: Snake {
                segments: [Position { x: 0, y: 0 }; GRID_WIDTH * GRID_HEIGHT],
                length: 4,
                head_index: 3,
                direction: Direction::Right,
            },
            apple: Position { x: 0, y: 0 },
            step_period: 300,
            score: 0,
            next_reward: 10,
        };
        game_state.teleport_apple();
        game_state.snake.segments[1].x = 1;
        game_state.snake.segments[2].x = 2;
        game_state.snake.segments[3].x = 3;
        game_state.repaint();
        snake_score_changed(0);
        game_state
    }

    pub fn on_key_down(&mut self, code: u32) {
        match code {
            KEY_CODE_ARROW_UP => self.change_snake_direction(Direction::Up),
            KEY_CODE_ARROW_DOWN => self.change_snake_direction(Direction::Down),
            KEY_CODE_ARROW_LEFT => self.change_snake_direction(Direction::Left),
            KEY_CODE_ARROW_RIGHT => self.change_snake_direction(Direction::Right),
            _ => todo!(),
        }
    }

    pub fn step(&mut self, _timestamp: i32) {
        if self.snake_will_eat_apple() {
            self.snake.grow();
            self.teleport_apple();
            self.speedup_game();
            self.update_score();
            snake_score_changed(self.score);
        } else {
            self.snake.move_ahead();
        }
        if self.snake.is_out_of_bounds(GRID_WIDTH, GRID_HEIGHT) || self.snake.eats_himself()
        {
            snake_game_over();
        }
        self.repaint();
    }

    fn repaint(&self) {
        paint_background();
        paint_snake(&self.snake);
        paint_apple(self.apple);
        canvas_fill();
    }

    fn change_snake_direction(&mut self, d: Direction) {
        if direction_is_opposite(self.snake.direction, d) {
            return;
        }
        self.snake.direction = d;
    }

    fn speedup_game(&mut self) {
        if self.step_period > 50 {
            self.step_period -= 25;
            snake_step_period_updated(self.step_period);
        }
    }

    fn snake_will_eat_apple(&self) -> bool {
        self.snake.next_head_position() == self.apple
    }

    fn update_score(&mut self) {
        self.score += self.next_reward;
        self.next_reward += 10;
    }

    fn teleport_apple(&mut self) {
        self.apple.x = js_random(GRID_WIDTH);
        self.apple.y = js_random(GRID_HEIGHT);
    }
}
