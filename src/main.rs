use lazy_static::lazy_static;
use leptos::{ev, leptos_dom::logging::console_log, prelude::*};
use leptos_use::{use_event_listener, use_interval_fn};
use rand::Rng;
use reactive_stores::Store;
use std::{
    cell::RefCell,
    collections::HashSet,
    ops::{Add, Sub},
    sync::Arc,
};

use wasm_bindgen::prelude::*;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Position(pub i32, pub i32);

impl Add<Position> for Position {
    type Output = Position;

    fn add(self, other: Position) -> Position {
        Position(self.0 + other.0, self.1 + other.1)
    }
}

impl Sub<Position> for Position {
    type Output = Position;

    fn sub(self, other: Position) -> Position {
        Position(self.0 - other.0, self.1 - other.1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct TetrominoData {
    pub position: Position,      // absolute position in the grid
    pub data: HashSet<Position>, // relative positions of the blocks
}

#[derive(Debug, Clone)]
pub struct Tetromino {
    pub kind: &'static str,
    pub data: TetrominoData,
    rotation: usize,
}

macro_rules! place_it {
    ($($pos:expr),*) => {
        [$(Position($pos.0, $pos.1)),*].into()
    };
}

lazy_static! {
    static ref S_OPTS: [[HashSet<Position>; 4]; 7] = [
        [
            place_it!((1, 0), (1, 1), (1, 2), (1, 3)),
            place_it!((0, 1), (1, 1), (2, 1), (3, 1)),
            place_it!((2, 0), (2, 1), (2, 2), (2, 3)),
            place_it!((0, 2), (1, 2), (2, 2), (3, 2)),
        ],
        [
            place_it!((1, 0), (0, 1), (1, 1), (2, 1)),
            place_it!((1, 0), (1, 1), (2, 1), (1, 2)),
            place_it!((0, 1), (1, 1), (2, 1), (1, 2)),
            place_it!((1, 0), (0, 1), (1, 1), (1, 2)),
        ],
        [
            place_it!((1, 0), (2, 0), (1, 1), (2, 1)),
            place_it!((1, 0), (2, 0), (1, 1), (2, 1)),
            place_it!((1, 0), (2, 0), (1, 1), (2, 1)),
            place_it!((1, 0), (2, 0), (1, 1), (2, 1)),
        ],
        [
            place_it!((1, 0), (1, 1), (1, 2), (0, 2)),
            place_it!((0, 0), (0, 1), (1, 1), (2, 1)),
            place_it!((1, 0), (2, 0), (1, 1), (1, 2)),
            place_it!((0, 1), (1, 1), (2, 1), (2, 2)),
        ],
        [
            place_it!((1, 0), (1, 1), (1, 2), (2, 2)),
            place_it!((0, 1), (1, 1), (2, 1), (0, 2)),
            place_it!((0, 0), (1, 0), (1, 1), (1, 2)),
            place_it!((0, 1), (1, 1), (2, 1), (2, 0)),
        ],
        [
            place_it!((1, 0), (2, 0), (1, 1), (0, 1)),
            place_it!((1, 0), (1, 1), (2, 1), (2, 2)),
            place_it!((1, 0), (2, 0), (1, 1), (0, 1)),
            place_it!((1, 0), (1, 1), (2, 1), (2, 2)),
        ],
        [
            place_it!((0, 0), (1, 0), (1, 1), (2, 1)),
            place_it!((1, 0), (1, 1), (0, 1), (0, 2)),
            place_it!((0, 0), (1, 0), (1, 1), (2, 1)),
            place_it!((1, 0), (1, 1), (0, 1), (0, 2)),
        ],
    ];
}

impl Tetromino {
    pub fn new_random(pos: Position) -> Self {
        let mut rng = rand::rng();
        let index = rng.random_range(0..7);

        let rotation = 0;
        let kind;
        let mut data = TetrominoData::default();
        match index {
            0 => kind = "I",
            1 => kind = "T",
            2 => kind = "O",
            3 => kind = "J",
            4 => kind = "L",
            5 => kind = "S",
            6 => kind = "Z",
            _ => unreachable!(),
        }
        data.position = pos;
        data.data = Tetromino::get_rotation_data(kind, rotation);
        Tetromino {
            kind,
            data,
            rotation,
        }
    }

    pub fn get_rotation_data(kind: &str, rotation: usize) -> HashSet<Position> {
        match kind {
            "I" => S_OPTS[0][rotation].clone(),
            "T" => S_OPTS[1][rotation].clone(),
            "O" => S_OPTS[2][rotation].clone(),
            "J" => S_OPTS[3][rotation].clone(),
            "L" => S_OPTS[4][rotation].clone(),
            "S" => S_OPTS[5][rotation].clone(),
            "Z" => S_OPTS[6][rotation].clone(),
            _ => unreachable!(),
        }
    }

    pub fn remove_at(&mut self, pos: Position) {
        let pos = pos - self.data.position;
        if !self.data.data.remove(&pos) {
            console_log(&format!("remove_at: position not found: {:?}", pos));
        }
    }

    pub fn fall_down(&mut self, y: i32) {
        let y = y - self.data.position.1;
        let mut new_data = HashSet::new();
        for pos in self.data.data.iter() {
            if pos.1 < y {
                new_data.insert(Position(pos.0, pos.1 + 1));
            } else {
                new_data.insert(*pos);
            }
        }
        self.data.data = new_data;
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
        let rotation = (self.rotation + 1) % 4;
        data.data = Tetromino::get_rotation_data(self.kind, rotation);
        Self {
            kind: self.kind,
            data,
            rotation,
        }
    }
}

#[derive(Debug, Store)]
pub struct Tetris {
    pub width: u32,
    pub height: u32,

    current_tetromino: Option<Tetromino>,
    ghost_tetromino: Option<Tetromino>,
    fixed_blocks: Vec<Tetromino>,
    speed: i32,

    score: i32,
    lost: bool,
    paused: bool,
}

impl Tetris {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            fixed_blocks: vec![],
            speed: 1,
            current_tetromino: Some(Tetromino::new_random(Position((width - 4) as i32 / 2, 0))),
            ghost_tetromino: None,
            score: 0,
            lost: false,
            paused: false,
        }
    }

    pub fn render_view(&self) -> Vec<Vec<&'static str>> {
        let mut output = vec![vec!["B"; self.width as usize]; self.height as usize];
        for block in &self.fixed_blocks {
            for pos in &block.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = block.kind;
            }
        }
        if let Some(tetromino) = &self.ghost_tetromino {
            for pos in tetromino.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = "G";
            }
        }
        if let Some(tetromino) = &self.current_tetromino {
            for pos in tetromino.collect_positions() {
                output[pos.1 as usize][pos.0 as usize] = tetromino.kind;
            }
        }
        output
    }

    pub fn render(&self) -> String {
        let output = self.render_view();
        output
            .into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        std::thread::sleep(std::time::Duration::from_millis(1000 / self.speed as u64));

        if !self.paused {
            self.move_down();
        }
    }

    fn translate(&mut self, pos: Position) {
        let mut new_tetromino = self.current_tetromino.clone().unwrap();
        new_tetromino.data.position = new_tetromino.data.position + pos;
        if self.is_oob(&new_tetromino) || self.is_colliding(&new_tetromino) {
            return;
        }
        self.current_tetromino.replace(new_tetromino);
    }

    pub fn move_left(&mut self) {
        if self.lost {
            return;
        }
        self.translate(Position(-1, 0));
        self.udpate_ghost();
    }

    pub fn move_right(&mut self) {
        if self.lost {
            return;
        }
        self.translate(Position(1, 0));
        self.udpate_ghost();
    }

    // down to the bottom
    pub fn speed_up(&mut self) {
        if self.lost {
            return;
        }
        let mut new_tetromino = self.current_tetromino.clone().unwrap();
        loop {
            let mut next = new_tetromino.clone();
            next.data.position = next.data.position + Position(0, 1);
            if self.is_oob(&next) || self.is_colliding(&next) {
                self.fixed_blocks.push(new_tetromino);
                let next = Tetromino::new_random(Position((self.width - 4) as i32 / 2, 0));
                if self.is_colliding(&next) {
                    self.lost = true;
                }
                self.current_tetromino = Some(next);
                self.udpate_ghost();
                self.clear_lines();
                break;
            }
            new_tetromino = next;
        }
    }

    fn clear_lines(&mut self) {
        if self.lost {
            return;
        }

        loop {
            let mut occupied = vec![vec![false; self.width as usize]; self.height as usize];
            for block in &self.fixed_blocks {
                for pos in &block.collect_positions() {
                    occupied[pos.1 as usize][pos.0 as usize] = true;
                }
            }

            let last_full_line = occupied
                .into_iter()
                .enumerate()
                .filter(|(_, row)| row.iter().all(|&c| c))
                .map(|(i, _)| i)
                .last();

            if last_full_line.is_none() {
                break;
            }
            let last_full_line = last_full_line.unwrap();
            console_log(&format!("last full line: {:?}", last_full_line));
            self.score += 1;

            for block in &mut self.fixed_blocks {
                for pos in block.collect_positions() {
                    if pos.1 == last_full_line as i32 {
                        block.remove_at(pos);
                    }
                }
            }

            for block in &mut self.fixed_blocks {
                block.fall_down(last_full_line as i32);
            }
        }
    }

    pub fn move_down(&mut self) {
        if self.lost {
            return;
        }

        let mut new_tetromino = self.current_tetromino.clone().unwrap();
        new_tetromino.data.position = new_tetromino.data.position + Position(0, self.speed);
        if self.is_oob(&new_tetromino) || self.is_colliding(&new_tetromino) {
            self.fixed_blocks
                .push(self.current_tetromino.take().unwrap());
            let next = Tetromino::new_random(Position((self.width - 4) as i32 / 2, 0));
            if self.is_colliding(&next) {
                self.lost = true;
            }
            self.current_tetromino = Some(next);
            self.ghost_tetromino = None;
        } else {
            self.current_tetromino = Some(new_tetromino);
            self.udpate_ghost();
        }

        self.clear_lines();
    }

    pub fn udpate_ghost(&mut self) {
        let mut next = self.current_tetromino.clone().unwrap();
        loop {
            next.data.position = next.data.position + Position(0, 1);
            if self.is_oob(&next) || self.is_colliding(&next) {
                next.data.position = next.data.position - Position(0, 1);
                self.ghost_tetromino = Some(next);
                break;
            }
        }
    }

    pub fn get_score(&self) -> i32 {
        self.score
    }

    pub fn rotate(&mut self) {
        let current = self.current_tetromino.as_ref().unwrap();

        let new_tetromino = current.rotated();
        if self.is_oob(&new_tetromino) || self.is_colliding(&new_tetromino) {
            return;
        }
        self.current_tetromino.replace(new_tetromino);
        self.udpate_ghost();
    }

    pub fn is_oob(&self, t: &Tetromino) -> bool {
        t.collect_positions()
            .iter()
            .any(|&p| p.0 < 0 || p.0 >= self.width as i32 || p.1 < 0 || p.1 >= self.height as i32)
    }

    pub fn is_colliding(&self, t: &Tetromino) -> bool {
        self.fixed_blocks.iter().any(|b| b.is_colliding(t))
    }

    pub fn pause(&mut self) {
        self.paused = true;
    }

    pub fn resume(&mut self) {
        self.paused = false;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

#[component]
fn TetrisGame(
    restart: ReadSignal<bool>,
    set_score: WriteSignal<i32>,
    btn_pressed: ReadSignal<&'static str>,
) -> impl IntoView {
    let tetris = Arc::new(RefCell::new(Tetris::new(10, 25)));
    let state = RwSignal::new_local(tetris);
    let (board, set_board) = signal(vec![]);
    let (paused, set_paused) = signal(false);

    Effect::new(move || {
        state.with(|st| {
            console_log(&format!("paused changed: {}", paused.get()));
            if paused.get() {
                st.borrow_mut().pause();
            } else {
                st.borrow_mut().resume();
            }
        });
    });

    Effect::new(move || {
        if restart.get() {
            state.with(|st| {
                st.replace(Tetris::new(10, 25));
                set_board.set(st.borrow().render_view());
                set_paused.set(false);
                set_score.set(0);
            });
        }
    });

    // Handle Tauri window focus events
    Effect::new(move |_| {
        let callback_focus_lost = move || set_paused.set(true);
        let callback_focus_gained = move || set_paused.set(false);

        // Register focus change event listeners with Tauri
        match js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("__TAURI__")) {
            Ok(true) => {
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
                    fn listen(event: &str, handler: &JsValue) -> JsValue;

                }

                let callback_focus_lost =
                    Closure::wrap(Box::new(callback_focus_lost) as Box<dyn FnMut()>);
                let callback_focus_gained =
                    Closure::wrap(Box::new(callback_focus_gained) as Box<dyn FnMut()>);

                console_log("tauri detected, registering listeners");
                listen("tauri://blur", callback_focus_lost.as_ref());
                listen("tauri://focus", callback_focus_gained.as_ref());

                // Prevent callbacks from being garbage collected
                callback_focus_lost.forget();
                callback_focus_gained.forget();
            }
            _ => {
                let initial_focus = document().has_focus().unwrap_or_default();
                set_paused.set(!initial_focus);
                console_log(&format!("initial focus: {}", initial_focus));

                let _ =
                    use_event_listener(window(), leptos::ev::blur, move |_| callback_focus_lost());
                let _ = use_event_listener(window(), leptos::ev::focus, move |_| {
                    callback_focus_gained()
                });
            }
        }
    });

    use_interval_fn(
        move || {
            state.with(|st| {
                st.borrow_mut().tick();
                set_score.set(st.borrow().get_score());
                set_board.set(st.borrow().render_view());
            });
        },
        1000,
    );

    let click_handler = move |key: &str| {
        state.with(|st| {
            if st.borrow().is_paused() && key != "KeyP" {
                return;
            }

            match key {
                "ArrowUp" => st.borrow_mut().rotate(),
                "ArrowLeft" => st.borrow_mut().move_left(),
                "ArrowRight" => st.borrow_mut().move_right(),
                "ArrowDown" => st.borrow_mut().tick(),
                "Space" => st.borrow_mut().speed_up(),
                "KeyP" => {
                    if st.borrow().is_paused() {
                        // st.borrow_mut().resume();
                        set_paused.set(false);
                    } else {
                        // st.borrow_mut().pause();
                        set_paused.set(true);
                    }
                }
                _ => return,
            }

            set_score.set(st.borrow().get_score());
            set_board.set(st.borrow().render_view());
        });
    };

    window_event_listener(ev::keydown, move |e| {
        // console_log(&format!("{:?}", e.code()));
        click_handler(e.code().as_str());
    });

    Effect::new(move || {
        if btn_pressed.get() != "" {
            click_handler(btn_pressed.get());
        }
    });

    let kind2color = |c| match c {
        "I" => "blue",
        "T" => "purple",
        "O" => "yellow",
        "J" => "green",
        "L" => "orange",
        "S" => "red",
        "Z" => "cyan",
        "B" => "rgb(119, 119, 119)",
        "G" => "rgba(121, 119, 119, 0.76)",
        _ => unreachable!(),
    };

    view! {
        <div class="flex flex-col items-center justify-center h-full relative">
            {move || {
                board.get().iter().map(move |row| {
                    view! {
                        <div class="row flex flex-row h-[calc(100%/25)]">
                            {row.iter().map(|&c| {
                                view! {
                                    <div
                                        class="cell aspect-square"
                                        style:background-color=move || kind2color(c) >
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}

            // Pause overlay
            <div
                class="absolute top-0 left-0 w-full h-full bg-black bg-opacity-70 flex items-center justify-center"
                style:display=move || if paused.get() { "flex" } else { "none" }
            >
                <div class="text-white text-2xl font-bold">PAUSED</div>
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (restart, set_restart) = signal(false);
    let (score, set_score) = signal(0);
    let (btn_pressed, set_btn_pressed) = signal("");

    view! {
        <div class="flex flex-row h-screen w-screen place-content-center gap-4">
            <TetrisGame restart=restart set_score=set_score  btn_pressed=btn_pressed/>
            <div class="flex flex-col h-full justify-between py-4">
                <div class="flex flex-col gap-4 items-center">
                    <div class="badge badge-soft badge-accent"> Scores: {score} </div>
                    <div class="badge badge-soft badge-primary"> Level: 1 </div>
                </div>
                <div class="grid grid-cols-3 gap-0">
                    <div class="btn btn-sm col-span-1 col-start-2" on:click=move |_| set_btn_pressed.set("ArrowUp")>U</div>
                    <div class="btn btn-sm col-span-1 col-start-1" on:click=move |_| set_btn_pressed.set("ArrowLeft")>L</div>
                    <div class="btn btn-sm col-span-1" on:click=move |_| set_btn_pressed.set("ArrowDown")>D</div>
                    <div class="btn btn-sm col-span-1" on:click=move |_| set_btn_pressed.set("ArrowRight")>R</div>
                    <div class="btn btn-sm col-span-3" on:click=move |_| set_btn_pressed.set("Space")>Space</div>
                </div>
                <div class="btn btn-neutral" on:click=move |_| set_restart.set(true)> Restart </div>
            </div>
        </div>
    }
}

#[cfg(target_arch = "wasm32")]
fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    console_error_panic_hook::set_once();
    let mut tetris = Tetris::new(10, 25);
    println!("{:?}", tetris.current_tetromino);
    clear_screen();
    println!("\n{}", tetris.render());

    loop {
        tetris.tick();
        clear_screen();
        println!("{}", tetris.render());
    }
}

#[allow(unused)]
fn clear_screen() {
    print!("\x1b[2J");
}
