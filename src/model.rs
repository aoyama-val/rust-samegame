use std::time;

use rand::{rngs::StdRng, Rng, SeedableRng};

pub const FPS: i32 = 30;
pub const BOARD_W: i32 = 20;
pub const BOARD_H: i32 = 10;
pub const COLORS_COUNT: i32 = 5;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Command {
    None,
    Hover(usize, usize),
    Click(usize, usize),
}

#[derive(Debug, Default, Copy, Clone)]
pub struct Cell {
    pub exist: bool,
    pub color: i32,
}

pub struct Game {
    pub is_over: bool,
    pub is_clear: bool,
    pub rng: StdRng,
    pub score: i32,
    pub hover_score: i32,
    pub board: [[Cell; BOARD_W as usize]; BOARD_H as usize],
}

impl Game {
    pub fn new() -> Self {
        let now = time::SystemTime::now();
        let timestamp = now
            .duration_since(time::UNIX_EPOCH)
            .expect("SystemTime before UNIX EPOCH!")
            .as_secs();
        let rng = StdRng::seed_from_u64(timestamp);
        println!("random seed = {}", timestamp);
        // let rng = StdRng::seed_from_u64(0);

        let mut game = Game {
            is_over: false,
            is_clear: false,
            rng: rng,
            board: [[Cell::default(); BOARD_W as usize]; BOARD_H as usize],
            score: 0,
            hover_score: 0,
        };

        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                game.board[y][x] = Cell {
                    exist: true,
                    color: game.rng.gen_range(0..COLORS_COUNT),
                }
            }
        }
        game
    }

    pub fn update(&mut self, command: Command) {
        if self.is_over || self.is_clear {
            return;
        }

        match command {
            Command::None => {}
            Command::Click(x, y) => {
                self.click(x, y);
            }
            Command::Hover(x, y) => {
                self.hover(x, y);
            }
        }
    }

    pub fn click(&mut self, x: usize, y: usize) {}
    pub fn hover(&mut self, x: usize, y: usize) {
        self.hover_score = self.calc_score(x, y);
    }

    pub fn calc_score(&mut self, x: usize, y: usize) -> i32 {
        if self.board[y][x].exist {
            self.board[y][x].color
        } else {
            -1
        }
    }

    pub fn is_valid_cell(&self, x: usize, y: usize) -> bool {
        if 0 <= x && x < BOARD_W as usize && 0 <= y && y < BOARD_H as usize {
            true
        } else {
            false
        }
    }
}
