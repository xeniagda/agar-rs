#![feature(proc_macro, wasm_custom_section, wasm_import_module)]


#[macro_use]
extern crate lazy_static;
extern crate agar_backend;
extern crate serde_json;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;


pub mod ext;

use std::sync::Mutex;
use agar_backend::{State, Player, IdPlayerCommand, PlayerCommand};
use ext::*;

const SCALE: f64 = 10.;

lazy_static! {
    static ref SIZE: Mutex<(usize, usize)> = Mutex::new((0, 0));
    static ref STATE: Mutex<(State, usize)> = Mutex::new((State::new(), 0)); // State, client_id
    //static ref LAST_TICK: Mutex<Option<Instant>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn start(width: usize, height: usize) {
    if let Ok(mut size) = SIZE.lock() {
        *size = (width, height);
    }
    if let Ok(mut state) = STATE.lock() {
        state.0.players.insert(0, Player { pos: (30., 30.), direction: 1., speed: 3., size: 1. });
    }

    draw();
}

#[wasm_bindgen]
pub fn tick() {
    draw();
    if let Ok(mut state) = STATE.lock() {
        state.0.tick(1. / 60.);
    }
}

#[wasm_bindgen]
pub fn resize(width: usize, height: usize) {
    if let Ok(mut size) = SIZE.lock() {
        *size = (width, height);
    }
}


#[wasm_bindgen]
pub fn mouse_moved(to_x: usize, to_y: usize) {
    let size = SIZE.lock();
    if size.is_err() { return; }
    let size = size.unwrap();

    let (dx, dy) = (to_x as f64 - size.0 as f64 / 2., to_y as f64 - size.1 as f64 / 2.);
    log(&format!("dx = {}, dy = {}", dx, dy));

    let theta = atan2(dx, dy);

    if let Ok(mut state) = STATE.lock() {
        let cmd = IdPlayerCommand { id: state.1, command: PlayerCommand::SetDirectionAndSpeed(theta, 5.) };

        ws_send(serde_json::to_string(&cmd).unwrap());

        log(&format!("Cmd: {:?}", cmd));
        state.0.do_command(cmd);

    }

}


#[wasm_bindgen]
pub fn redraw() {
    draw();
}

#[wasm_bindgen]
pub fn recv_ws_message(data: String) {
    log(&format!("Received {:?}", data));
    if let Ok(mut state) = STATE.lock() {
        match serde_json::from_str::<(State, usize)>(&data) {
            Ok(new_state) => { *state = new_state }
            Err(e) => { log(&format!("Decoding error: {:?}", e)) }
        }
    }
}

fn draw() {
    clear();

    let size = SIZE.lock();
    if size.is_err() { return; }
    let size = size.unwrap();

    if let Ok(state) = STATE.lock() {
        for (_, player) in &state.0.players {
            put_circle((player.pos.0 * SCALE, player.pos.1 * SCALE), player.size * SCALE, (255, 0, 255));
        }
    }
}
