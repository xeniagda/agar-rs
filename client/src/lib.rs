#![feature(proc_macro, wasm_custom_section, wasm_import_module)]


#[macro_use]
extern crate lazy_static;
extern crate agar_backend;
extern crate serde_json;
extern crate itertools;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;


pub mod ext;

use std::sync::Mutex;
use std::cmp::Ordering;

use agar_backend::{State, IdPlayerCommand, PlayerCommand};
use ext::*;
use itertools::Itertools;

const LINE_SPACE: f64 = 5.;

lazy_static! {
    static ref SIZE: Mutex<(usize, usize)> = Mutex::new((0, 0));
    static ref STATE: Mutex<(State, usize)> = Mutex::new((State::new(), 0)); // State, client_id
    static ref SCALE: Mutex<f64> = Mutex::new(1.);

    //static ref LAST_TICK: Mutex<Option<Instant>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn start(width: usize, height: usize) {
    if let Ok(mut size) = SIZE.lock() {
        *size = (width, height);
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

    let theta = atan2(dx, dy);

    let (dx_norm, dy_norm) = (dx / size.0 as f64 * 2., dy / size.1 as f64 * 2.);
    let r_sq = (dx_norm * dx_norm + dy_norm * dy_norm).sqrt() * 6.;

    if let Ok(mut state) = STATE.lock() {
        let cmd = IdPlayerCommand { id: state.1, command: PlayerCommand::SetDirectionAndSpeed(theta, r_sq) };

        ws_send(serde_json::to_string(&cmd).unwrap());

        state.0.do_command(cmd);

    }

}


#[wasm_bindgen]
pub fn redraw() {
    draw();
}

#[wasm_bindgen]
pub fn scroll(y: f64) {
    if let Ok(mut scale) = SCALE.lock() {
        *scale = (*scale - y / 20.).max(0.4).min(3.);
    }
}

#[wasm_bindgen]
pub fn recv_ws_message(data: String) {
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

    let scale_mul = SCALE.lock();
    if scale_mul.is_err() { return; }
    let scale_mul = *scale_mul.unwrap();


    if let Ok(state) = STATE.lock() {
        let my_pos = state.0.players.get(&state.1).map(|x| x.pos).unwrap_or((0., 0.));
        let my_size = state.0.players.get(&state.1).map(|x| x.show_size).unwrap_or(10.);

        let scale = scale_mul / (my_size.sqrt() + 2.) * 30.;

        // Grid lines
        let x_scroll = (my_pos.0 / LINE_SPACE - ((my_pos.0 / LINE_SPACE) as i64) as f64) * LINE_SPACE;
        let lines = (size.0 as f64 / (scale * LINE_SPACE)) as i64 + 2;
        for x in -lines / 2..lines / 2 + 1 {
            let x = x as f64;
            put_line(
                (x * scale * LINE_SPACE - x_scroll * scale + size.0 as f64 / 2., 0.),
                (x * scale * LINE_SPACE - x_scroll * scale + size.0 as f64 / 2., size.1 as f64),
                1.,
                (200, 200, 200)
            );
        }
        let y_scroll = (my_pos.1 / LINE_SPACE - ((my_pos.1 / LINE_SPACE) as i64) as f64) * LINE_SPACE;
        let lines = (size.1 as f64 / (scale * LINE_SPACE)) as i64 + 2;
        for y in -lines / 2..lines / 2 + 1 {
            let y = y as f64;
            put_line(
                (           0., y * scale * LINE_SPACE - y_scroll * scale + size.1 as f64 / 2.),
                (size.0 as f64, y * scale * LINE_SPACE - y_scroll * scale + size.1 as f64 / 2.),
                1.,
                (200, 200, 200)
            );
        }

        for ball in &state.0.balls {
            put_circle(
                ((ball.pos.0 - my_pos.0) * scale + size.0 as f64 / 2.,
                 (ball.pos.1 - my_pos.1) * scale + size.1 as f64 / 2.),
                scale,
                ball.color
            );
        }

        let sorted_players = &state.0.players.values()
                .sorted_by(|x, y| PartialOrd::partial_cmp(&x.size, &y.size).unwrap_or(Ordering::Less));
        for player in sorted_players {
            put_circle(
                ((player.pos.0 - my_pos.0) * scale + size.0 as f64 / 2.,
                 (player.pos.1 - my_pos.1) * scale + size.1 as f64 / 2.),
                player.show_size * scale,
                (255, 0, 255)
            );
        }
    }
}
