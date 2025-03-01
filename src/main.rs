use leptos::{leptos_dom::logging::console_log, prelude::*};
use rand::Rng;
use reactive_stores::Store;
use std::{cell::RefCell, collections::HashSet, ops::Add, rc::Rc, sync::Arc, time::Duration};
use web_sys::{
    self,
    wasm_bindgen::{JsCast, prelude::Closure},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(pub i32, pub i32);

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position(self.0 + other.0, self.1 + other.1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TetrominoData {
    pub position: Position,      // absolute position in the grid
    pub data: HashSet<Position>, // relative positions of the blocks
    pub anchor: Position,        // relative position of the anchor block
}

#[derive(Debug, Clone)]
pub struct Tetromino {
    pub kind: &'static str,
    pub data: TetrominoData,
}

impl Tetromino {
    pub fn new_random(pos: Position) -> Self {
        let mut rng = rand::rng();
        let index = rng.random_range(0..7);

        let kind;
        let mut data = TetrominoData::default();
        match index {
            0 => {
                kind = "I";
                data.anchor = Position(1, 0);
                data.data = [
                    Position(0, 0),
                    Position(0, 1),
                    Position(0, 2),
                    Position(0, 3),
                ]
                .into();
            }
            1 => {
                kind = "T";
                data.anchor = Position(0, 0);
                data.data = [
                    Position(0, 0),
                    Position(1, 0),
                    Position(2, 0),
                    Position(1, 1),
                ]
                .into();
            }
            2 => {
                kind = "O";
                data.anchor = Position(0, 0);
                data.data = [
                    Position(0, 0),
                    Position(1, 0),
                    Position(0, 1),
                    Position(1, 1),
                ]
                .into();
            }
            3 => {
                kind = "J";
                data.anchor = Position(1, 1);
                data.data = [
                    Position(1, 0),
                    Position(1, 1),
                    Position(1, 2),
                    Position(0, 2),
                ]
                .into();
            }
            4 => {
                kind = "L";
                data.anchor = Position(0, 1);
                data.data = [
                    Position(0, 0),
                    Position(0, 1),
                    Position(0, 2),
                    Position(1, 2),
                ]
                .into();
            }
            5 => {
                kind = "S";
                data.anchor = Position(0, 0);
                data.data = [
                    Position(1, 0),
                    Position(2, 0),
                    Position(1, 1),
                    Position(0, 1),
                ]
                .into();
            }
            6 => {
                kind = "Z";
                data.anchor = Position(0, 0);
                data.data = [
                    Position(0, 0),
                    Position(1, 0),
                    Position(1, 1),
                    Position(2, 1),
                ]
                .into();
            }
            _ => unreachable!(),
        }
        data.position = pos;
        Tetromino { kind, data }
    }

    fn get_emoji(&self) -> char {
        match self.kind {
            "I" => '🟦',
            "T" => '🟧',
            "O" => '🟨',
            "J" => '🟩',
            "L" => '🟪',
            "S" => '🟫',
            "Z" => '🟥',
            _ => unreachable!(),
        }
    }

    pub fn collect_positions(&self) -> Vec<Position> {
        self.data
            .data
            .iter()
            .map(|p| self.data.position + *p)
            .collect()
    }

    pub fn is_colliding(&self, other: &Tetromino) -> bool {
        let other_positions = other.collect_positions();
        self.collect_positions()
            .iter()
            .any(|p| other_positions.contains(p))
    }

    pub fn rotated(&self) -> Self {
        let mut data = self.data.clone();
        data.data = data.data.into_iter().map(|p| Position(-p.1, p.0)).collect();
        Self {
            kind: self.kind,
            data,
        }
    }
}

#[derive(Debug, Store)]
pub struct Tetris {
    pub width: u32,
    pub height: u32,

    current_tetromino: Option<Tetromino>,
    fixed_blocks: Vec<Tetromino>,
    speed: i32,
}

impl Tetris {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fixed_blocks: vec![],
            speed: 1,
            current_tetromino: Some(Tetromino::new_random(Position((width - 4) as i32 / 2, 2))),
        }
    }

    pub fn render_view(&self) -> Vec<Vec<char>> {
        let mut output = vec![vec![' '; self.width as usize]; self.height as usize];
        for block in &self.fixed_blocks {
            for pos in &block.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = block.get_emoji();
            }
        }
        if let Some(tetromino) = &self.current_tetromino {
            for pos in tetromino.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = tetromino.get_emoji();
            }
        }
        output
    }

    pub fn render(&self) -> String {
        let mut output = vec![vec!['.'; self.width as usize]; self.height as usize];
        for block in &self.fixed_blocks {
            for pos in &block.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = block.get_emoji();
            }
        }
        if let Some(tetromino) = &self.current_tetromino {
            for pos in tetromino.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = tetromino.get_emoji();
            }
        }
        output
            .into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(1000 / self.speed as u64));
        self.move_down();
        // self.rotate();
    }

    pub fn move_down(&mut self) {
        let mut new_tetromino = self.current_tetromino.clone().unwrap();
        new_tetromino.data.position = new_tetromino.data.position + Position(0, self.speed);
        if self.is_oob(&new_tetromino) || self.is_colliding(&new_tetromino) {
            self.fixed_blocks
                .push(self.current_tetromino.take().unwrap());
            self.current_tetromino = Some(Tetromino::new_random(Position(
                (self.width - 4) as i32 / 2,
                0,
            )));
        } else {
            self.current_tetromino = Some(new_tetromino);
        }
    }

    pub fn rotate(&mut self) {
        let current = self.current_tetromino.as_ref().unwrap();

        let new_tetromino = current.rotated();
        if self.is_oob(&new_tetromino) || self.is_colliding(&new_tetromino) {
            return;
        }
        self.current_tetromino.replace(new_tetromino);
    }

    pub fn is_oob(&self, t: &Tetromino) -> bool {
        t.collect_positions()
            .iter()
            .any(|&p| p.0 < 0 || p.0 >= self.width as i32 || p.1 < 0 || p.1 >= self.height as i32)
    }

    pub fn is_colliding(&self, t: &Tetromino) -> bool {
        self.fixed_blocks.iter().any(|b| b.is_colliding(t))
    }
}

#[component]
fn App() -> impl IntoView {
    let tetris = Arc::new(RefCell::new(Tetris::new(10, 20)));
    let state = RwSignal::new_local(tetris);

    let (board, set_board) = signal(vec![]);
    let cb = move || {
        state.with(|st| {
            st.borrow_mut().tick();
            set_board.set(st.borrow().render_view());
        });
    };
    set_interval_with_handle(cb, Duration::from_millis(1000)).expect("failed to set interval");

    Effect::new(move || {
        // let window = web_sys::window().unwrap();
        // let document = window.document().unwrap();
    });

    view! {
        <div>
        {move || {
            board.get().iter().map(move |row| {
                view! {
                    <div class="row">
                        {row.iter().map(|&c| { view! { <div class="cell">{c}</div> } }).collect::<Vec<_>>()}
                    </div>
                }
            }).collect::<Vec<_>>()
        }}
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    mount_to_body(|| view! { <App /> });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let mut tetris = Tetris::new(10, 20);
    println!("{:?}", tetris.current_tetromino);
    clear_screen();
    println!("\n{}", tetris.render());

    loop {
        tetris.tick();
        clear_screen();
        println!("{}", tetris.render());
    }
}

fn clear_screen() {
    print!("\x1b[2J");
}
