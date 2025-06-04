// Import the core logic from the tetris_core library
use tetris_core::*;

use leptos::{ev, leptos_dom::logging::console_log, prelude::*};
use leptos_use::{use_event_listener, use_interval_fn};
// No need for rand::Rng here if Tetris::new_random is in lib.rs
// No need for reactive_stores::Store here if Tetris struct (with derive) is in lib.rs
use web_sys::window; // Still needed for some UI logic if not fully abstracted
use std::{
    // cmp::Reverse, // Moved to lib.rs
    cell::RefCell,
    // collections::HashSet, // Moved to lib.rs
    // ops::{Add, Sub}, // Moved to lib.rs (Position ops)
    sync::Arc,
};

use wasm_bindgen::prelude::*; // Still needed for JsValue, Closure, etc. in UI/WASM part

// const ANIMATION_DURATION: u32 = 3; // This was moved to lib.rs, then determined unused and removed from there.

// All game logic structs (Position, TetrominoData, Tetromino, Tetris)
// and their impl blocks, S_OPTS, and FFI functions have been moved to src/lib.rs.

#[component]
fn TetrisGame(
    restart: ReadSignal<bool>,
    set_score: WriteSignal<i32>,
    btn_pressed: ReadSignal<&'static str>,
) -> impl IntoView {
    // Tetris struct now comes from tetris_core
    let tetris_instance = Arc::new(RefCell::new(Tetris::new(10, 25)));
    let state = RwSignal::new_local(tetris_instance); // RwSignal expects the argument to be Send + Sync if used across threads, check Tetris if it is. For single-threaded wasm, this is fine.
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
                // Tetris::new is from tetris_core
                st.replace(Tetris::new(10, 25));
                set_board.set(st.borrow().render_view());
                set_paused.set(false);
                set_score.set(0);
            });
        }
    });

    Effect::new(move |_| {
        let callback_focus_lost = move || set_paused.set(true);
        let callback_focus_gained = move || set_paused.set(false);

        match js_sys::Reflect::has(&js_sys::global(), &JsValue::from_str("__TAURI__")) {
            Ok(true) => {
                #[wasm_bindgen]
                extern "C" {
                    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "event"])]
                    fn listen(event: &str, handler: &JsValue) -> JsValue;
                }

                let callback_focus_lost_closure =
                    Closure::wrap(Box::new(callback_focus_lost.clone()) as Box<dyn FnMut()>);
                let callback_focus_gained_closure =
                    Closure::wrap(Box::new(callback_focus_gained.clone()) as Box<dyn FnMut()>);

                console_log("tauri detected, registering listeners");
                listen("tauri://blur", callback_focus_lost_closure.as_ref());
                listen("tauri://focus", callback_focus_gained_closure.as_ref());

                callback_focus_lost_closure.forget();
                callback_focus_gained_closure.forget();
            }
            _ => {
                let initial_focus = document().has_focus().unwrap_or_default();
                set_paused.set(!initial_focus);
                console_log(&format!("initial focus: {}", initial_focus));

                let _ = use_event_listener(window(), leptos::ev::blur, move |_| callback_focus_lost());
                let _ = use_event_listener(window(), leptos::ev::focus, move |_| callback_focus_gained());
            }
        }
    });

    use_interval_fn(
        move || {
            state.with(|st| {
                st.borrow_mut().tick(); // tick() is from tetris_core::Tetris
                set_score.set(st.borrow().get_score()); // get_score() from tetris_core::Tetris
                set_board.set(st.borrow().render_view()); // render_view() from tetris_core::Tetris
            });
        },
        1000,
    );

    let click_handler = move |key: &str| {
        state.with(|st| {
            if st.borrow().is_paused() && key != "KeyP" { // is_paused() from tetris_core::Tetris
                return;
            }

            if key == "KeyP" {
                if st.borrow().is_paused() {
                    set_paused.set(false);
                } else {
                    set_paused.set(true);
                }
                set_score.set(st.borrow().get_score());
                set_board.set(st.borrow().render_view());
                return;
            }

            // Accessing lines_being_cleared from tetris_core::Tetris (made pub in lib.rs)
            if st.borrow().lines_being_cleared.is_some() {
                return;
            }

            match key {
                // Methods like rotate, move_left etc are from tetris_core::Tetris
                "ArrowUp" => st.borrow_mut().rotate(),
                "ArrowLeft" => st.borrow_mut().move_left(),
                "ArrowRight" => st.borrow_mut().move_right(),
                "ArrowDown" => st.borrow_mut().tick(),
                "Space" => st.borrow_mut().speed_up(),
                _ => return,
            }

            set_score.set(st.borrow().get_score());
            set_board.set(st.borrow().render_view());
        });
    };

    window_event_listener(ev::keydown, move |e| {
        click_handler(e.code().as_str());
    });

    Effect::new(move || {
        if btn_pressed.get() != "" {
            click_handler(btn_pressed.get());
        }
    });

    let kind2color = |c| match c {
        "I" => "blue", "T" => "purple", "O" => "yellow", "J" => "green",
        "L" => "orange", "S" => "red", "Z" => "cyan",
        "B" => "rgb(119, 119, 119)", "G" => "rgba(121, 119, 119, 0.76)",
        _ => unreachable!(),
    };

    view! {
        <div class="flex flex-col items-center justify-center h-full relative">
            {move || {
                board.get().iter().enumerate().map(move |(row_idx, row_data)| {
                    view! {
                        <div class="row flex flex-row h-[calc(100%/25)]">
                            {row_data.iter().map(|&c| {
                                let cell_class = {
                                    let base_class = "cell aspect-square";
                                    // Accessing lines_being_cleared from tetris_core::Tetris
                                    let is_clearing = state.with(|s| {
                                        if let Some(clearing_lines) = &s.borrow().lines_being_cleared {
                                            clearing_lines.contains(&row_idx)
                                        } else {
                                            false
                                        }
                                    });
                                    if is_clearing {
                                        format!("{} line-clearing-animation", base_class)
                                    } else {
                                        base_class.to_string()
                                    }
                                };
                                view! {
                                    <div
                                        class=cell_class
                                        style:background-color=move || kind2color(c) >
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    }
                }).collect::<Vec<_>>()
            }}
            <div
                class="absolute top-0 left-0 w-full h-full bg-black bg-opacity-70 flex items-center justify-center"
                style:display=move || if paused.get() { "flex" } else { "none" } >
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
    // Tetris::new and other methods now from tetris_core
    let mut tetris = Tetris::new(10, 25);
    println!("{:?}", tetris.current_tetromino); // Accessing pub field from lib
    clear_screen();
    println!("\n{}", tetris.render()); // render from lib

    loop {
        tetris.tick(); // tick from lib
        clear_screen();
        println!("{}", tetris.render());
        if tetris.lost { // lost field from lib
            println!("Game Over! Score: {}", tetris.score); // score field from lib
            break;
        }
    }
}

#[allow(unused)]
fn clear_screen() {
    print!("\x1b[2J");
}

#[cfg(test)]
mod tests {
    use super::*; // This will bring tetris_core types into scope
    use std::collections::HashSet; // Keep this for test-local HashSet usage if any

    // Helper functions now use Tetris from tetris_core
    fn count_blocks_at_line(tetris: &Tetris, line_y: i32) -> usize {
        tetris
            .fixed_blocks // fixed_blocks is pub in tetris_core::Tetris
            .iter()
            .flat_map(|t| t.collect_positions()) // collect_positions is pub
            .filter(|p| p.1 == line_y)
            .count()
    }

    fn get_block_positions(tetris: &Tetris) -> HashSet<Position> { // Position is from tetris_core
        tetris
            .fixed_blocks
            .iter()
            .flat_map(|t| t.collect_positions())
            .collect()
    }

    #[test]
    fn test_line_clearing_animation() {
        let width = 10;
        let height = 5;
        let mut tetris = Tetris::new(width, height); // tetris_core::Tetris
        tetris.current_tetromino = None;
        tetris.fixed_blocks.clear();
        tetris.score = 0;

        let shifting_block_orig_pos = Position(0, height as i32 - 2);
        tetris.fixed_blocks.push(Tetromino { // tetris_core::Tetromino
            kind: "S",
            data: TetrominoData { // tetris_core::TetrominoData
                position: shifting_block_orig_pos,
                data: [Position(0,0)].into(),
            },
            rotation: 0,
        });

        let line_to_clear_y = height as i32 - 1;
        for i in 0..width {
            tetris.fixed_blocks.push(Tetromino {
                kind: "I",
                data: TetrominoData {
                    position: Position(i as i32, line_to_clear_y),
                    data: [Position(0,0)].into(),
                },
                rotation: 0,
            });
        }

        assert_eq!(tetris.fixed_blocks.len(), width as usize + 1);
        assert!(get_block_positions(&tetris).contains(&shifting_block_orig_pos));
        assert_eq!(count_blocks_at_line(&tetris, line_to_clear_y), width as usize);

        tetris.clear_lines(); // clear_lines() from tetris_core::Tetris (made pub for tests)

        assert_eq!(tetris.lines_being_cleared, Some(vec![line_to_clear_y as usize]));
        assert!(tetris.animation_start_time.is_some());
        assert_eq!(tetris.score, 1);
        assert_eq!(tetris.fixed_blocks.len(), width as usize + 1);
        assert!(get_block_positions(&tetris).contains(&shifting_block_orig_pos));
        assert_eq!(count_blocks_at_line(&tetris, line_to_clear_y), width as usize);

        tetris.tick(); // tick() from tetris_core::Tetris

        assert_eq!(tetris.lines_being_cleared, None);
        assert!(tetris.animation_start_time.is_none());
        assert_eq!(count_blocks_at_line(&tetris, line_to_clear_y), 0);
        
        let shifted_block_new_pos = Position(shifting_block_orig_pos.0, shifting_block_orig_pos.1 + 1);
        let current_positions = get_block_positions(&tetris);
        assert!(current_positions.contains(&shifted_block_new_pos), "Shifting block should have moved down to {:?}, current positions: {:?}", shifted_block_new_pos, current_positions);
        assert!(!current_positions.contains(&shifting_block_orig_pos));
        assert_eq!(tetris.fixed_blocks.len(), 1);
        assert_eq!(tetris.fixed_blocks[0].kind, "S");

        let old_score = tetris.score;
        tetris.clear_lines();
        assert_eq!(tetris.lines_being_cleared, None);
        assert_eq!(tetris.score, old_score);
    }
}

// FFI C-compatible API (GameState struct, tetris_create, tetris_destroy, etc.)
// has been moved to src/lib.rs and is part of the tetris_core library.
// It does not need to be redefined here.
// The `use tetris_core::*;` will bring in `GameState` if it were needed by main.rs code,
// but it's primarily for the FFI consumers, not for the Rust UI code in main.rs.
// The FFI functions themselves are also in lib.rs.
