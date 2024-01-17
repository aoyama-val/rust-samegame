use std::{collections::HashMap, time};

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
}

pub struct Game {
    pub is_over: bool,
    pub is_clear: bool,
    pub is_debug: bool,
    pub rng: StdRng,
    pub score: i32,
    pub hover_score: i32,
    pub hover_connected_count: i32,
    pub board: [[Cell; BOARD_W as usize]; BOARD_H as usize],
    pub connected_counts: HashMap<i32, i32>,
    pub hover_component_id: i32,
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
            is_debug: true,
            rng: rng,
            board: [[Cell::default(); BOARD_W as usize]; BOARD_H as usize],
            score: 0,
            hover_score: 0,
            hover_connected_count: 0,
            connected_counts: HashMap::new(),
            hover_component_id: -1,
        };

        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                game.board[y][x] = Cell {
                    exist: true,
                    color: game.rng.gen_range(0..COLORS_COUNT),
                    component_id: -1,
                }
            }
        }

        game.update_components();

        if game.is_debug {
            game.print();
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

    pub fn print(&self) {
        for y in 0..BOARD_H as usize {
            for x in 0..BOARD_W as usize {
                if self.board[y][x].exist {
                    print!("{} ", self.board[y][x].color);
                } else {
                    print!("  ");
                }
            }
            println!("");
        }
    }

    pub fn click(&mut self, x: usize, y: usize) {
        if self.is_debug {
            println!("Click {} {}", x, y);
        }

        let component_id = self.board[y][x].component_id;
        let connected_count = *self.connected_counts.get(&component_id).unwrap();
        if connected_count <= 1 {
            return;
        }

        let score = self.calc_point(x, y);

        // 繋がっている石を全部消す
        for y2 in 0..BOARD_H as usize {
            for x2 in 0..BOARD_W as usize {
                if self.board[y2][x2].component_id == component_id {
                    self.board[y2][x2].exist = false;
                }
            }
        }

        // 下に落とす
        for x2 in 0..BOARD_W as usize {
            let mut y3 = BOARD_H - 1;
            for y2 in (0..BOARD_H as usize).rev() {
                // 上方に存在するマスがあるか探す
                while y3 >= 0 && !self.board[y3 as usize][x2].exist {
                    y3 -= 1;
                }
                if y3 >= 0 {
                    self.board[y2][x2] = self.board[y3 as usize][x2]; // 存在するマスがあったらそれを落とす
                    y3 -= 1; // 次はその1個上から探す
                } else {
                    self.board[y2][x2].exist = false;
                }
            }
        }

        // 左に寄せる
        let mut x3 = 0;
        for x2 in 0..BOARD_W as usize {
            while x3 < BOARD_W as usize && self.is_empty_column(x3) {
                x3 += 1;
            }
            if x3 < BOARD_W as usize {
                self.copy_column(x3, x2);
                x3 += 1;
            } else {
                self.set_column_not_exist(x2);
            }
        }

        self.update_components();

        self.score += score;

        if self.is_debug {
            self.print();
        }
    }

    pub fn is_empty_column(&self, x: usize) -> bool {
        for y2 in (0..BOARD_H as usize).rev() {
            if self.board[y2][x].exist {
                return false;
            }
        }
        return true;
    }

    pub fn copy_column(&mut self, src_x: usize, dest_x: usize) {
        for y in 0..BOARD_H as usize {
            self.board[y][dest_x] = self.board[y][src_x];
        }
    }

    // 1列まるごと空にする
    pub fn set_column_not_exist(&mut self, x: usize) {
        for y in 0..BOARD_H as usize {
            self.board[y][x].exist = false;
        }
    }

    pub fn hover(&mut self, x: usize, y: usize) {
        if !self.board[y][x].exist {
            self.hover_connected_count = 0;
            self.hover_score = -1;
            self.hover_component_id = -1;
            return;
        }
        self.hover_connected_count = *self
            .connected_counts
            .get(&self.board[y][x].component_id)
            .unwrap();
        self.hover_score = self.calc_point(x, y);

        self.hover_component_id = self.board[y][x].component_id;
    }

    pub fn calc_point(&mut self, x: usize, y: usize) -> i32 {
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
        if !self.board[y][x].exist {
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
        if x >= 1 && self.board[y][x - 1].color == color {
            self.set_component_id(x - 1, y, component_id);
        }
        if x + 1 < BOARD_W as usize && self.board[y][x + 1].color == color {
            self.set_component_id(x + 1, y, component_id);
        }
        if y >= 1 && self.board[y - 1][x].color == color {
            self.set_component_id(x, y - 1, component_id);
        }
        if y + 1 < BOARD_H as usize && self.board[y + 1][x].color == color {
            self.set_component_id(x, y + 1, component_id);
        }
    }
}
