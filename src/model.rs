use std::{collections::HashMap, hash::Hash, time};

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
    pub component_id: i32,
    // pub connected_count: i32,
}

pub struct Game {
    pub is_over: bool,
    pub is_clear: bool,
    pub rng: StdRng,
    pub score: i32,
    pub hover_score: i32,
    pub hover_connected_count: i32,
    pub board: [[Cell; BOARD_W as usize]; BOARD_H as usize],
    pub connected_counts: HashMap<i32, i32>,
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
            hover_connected_count: 0,
            connected_counts: HashMap::new(),
        };

        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                game.board[y][x] = Cell {
                    exist: true,
                    color: game.rng.gen_range(0..COLORS_COUNT),
                    component_id: -1,
                    // connected_count: 0,
                }
            }
        }

        game.update_components();

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
        self.hover_connected_count = *self
            .connected_counts
            .get(&self.board[y][x].component_id)
            .unwrap();
        self.hover_score = self.calc_score(x, y);
    }

    pub fn calc_score(&mut self, x: usize, y: usize) -> i32 {
        if self.board[y][x].exist {
            let c = self
                .connected_counts
                .get(&self.board[y][x].component_id)
                .unwrap();
            if *c >= 2 {
                (c - 2) * (c - 2)
            } else {
                0
            }
        } else {
            -1
        }
    }

    pub fn is_valid_cell(&self, x: usize, y: usize) -> bool {
        if x < BOARD_W as usize && y < BOARD_H as usize {
            true
        } else {
            false
        }
    }

    pub fn update_components(&mut self) {
        self.connected_counts = HashMap::new();

        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                self.board[y][x].component_id = -1;
            }
        }

        let mut component_id = 0;
        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                self.set_component_id(x, y, component_id);
                component_id += 1;
            }
        }
    }

    pub fn set_component_id(&mut self, x: usize, y: usize, component_id: i32) {
        if self.board[y][x].component_id != -1 {
            return;
        }

        self.board[y][x].component_id = component_id;

        if self.connected_counts.contains_key(&component_id) {
            let count = self.connected_counts.get(&component_id).unwrap();
            self.connected_counts.insert(component_id, count + 1);
        } else {
            self.connected_counts.insert(component_id, 1);
        }

        let color = self.board[y][x].color;
        if x >= 1 && self.board[y][x - 1].exist && self.board[y][x - 1].color == color {
            self.set_component_id(x - 1, y, component_id);
        }
        if x + 1 < BOARD_W as usize
            && self.board[y][x + 1].exist
            && self.board[y][x + 1].color == color
        {
            self.set_component_id(x + 1, y, component_id);
        }
        if y >= 1 && self.board[y - 1][x].exist && self.board[y - 1][x].color == color {
            self.set_component_id(x, y - 1, component_id);
        }
        if y + 1 < BOARD_H as usize
            && self.board[y + 1][x].exist
            && self.board[y + 1][x].color == color
        {
            self.set_component_id(x, y + 1, component_id);
        }
    }
}
