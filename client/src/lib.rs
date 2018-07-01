#![feature(proc_macro, wasm_custom_section, wasm_import_module, nll)]


#[macro_use]
extern crate lazy_static;
extern crate agar_backend;
extern crate itertools;

extern crate wasm_bindgen;

#[cfg(feature = "serde_cbor")]
extern crate serde_cbor as serde_impl;

#[cfg(feature = "serde_json")]
extern crate serde_json as serde_impl;

use wasm_bindgen::prelude::*;


pub mod ext;

use std::sync::Mutex;
use std::cmp::Ordering;

use agar_backend::{State, IdPlayerCommand, PlayerCommand};
use ext::*;
use itertools::Itertools;

const LINE_SPACE: f64 = 5.;
const ZOOM_SPEED: f64 = 20.;

lazy_static! {
    static ref SIZE: Mutex<(usize, usize)> = Mutex::new((0, 0));
    static ref STATE: Mutex<(State, usize)> = Mutex::new((State::new(), 0)); // State, client_id
    static ref ZOOM: Mutex<(f64, f64)> = Mutex::new((1., 1.)); // (wanted, current)

    static ref LAST_TICK: Mutex<Option<f64>> = Mutex::new(None);
}

#[wasm_bindgen]
pub fn start(width: usize, height: usize) {
    if let Ok(mut size) = SIZE.lock() {
        *size = (width, height);
    }

    draw();
}

#[wasm_bindgen]
pub fn tick(now: f64) {

    let dt =
        if let Ok(mut last_tick) = LAST_TICK.lock() {
            match *last_tick {
                Some(ref mut last) => {
                    let dt = now - *last;
                    *last = now;
                    dt
                }
                None => {
                    *last_tick = Some(now);
                    return;
                }
            }
        } else {
            1. / 60.
        };

    draw();
    if let Ok(mut state) = STATE.lock() {
        state.0.tick(dt);
    }

    if let Ok(mut zoom) = ZOOM.lock() {
        zoom.1 = (zoom.1 - zoom.0) * (1. / ZOOM_SPEED).powf(dt) + zoom.0;
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

    let to_x = to_x.min(size.0).max(0);
    let to_y = to_y.min(size.1).max(0);

    let (dx, dy) = (to_x as f64 - size.0 as f64 / 2., to_y as f64 - size.1 as f64 / 2.);

    let theta = atan2(dx, dy);

    let (dx_norm, dy_norm) = (dx / size.0 as f64 * 2., dy / size.1 as f64 * 2.);
    let r_sq = (dx_norm * dx_norm + dy_norm * dy_norm).sqrt() * 6.;

    if let Ok(mut state) = STATE.lock() {
        let cmd = IdPlayerCommand { id: state.1, command: PlayerCommand::SetDirectionAndSpeed(theta, r_sq) };

        ws_send(serde_impl::to_vec(&cmd).unwrap());

        state.0.do_command(cmd);

    }

}


#[wasm_bindgen]
pub fn redraw() {
    draw();
}

#[wasm_bindgen]
pub fn scroll(y: f64) {
    if let Ok(mut zoom) = ZOOM.lock() {
        zoom.0 = (zoom.0 - y / 20.).max(0.4).min(3.);
    }
}

#[wasm_bindgen]
pub fn recv_ws(data: Vec<u8>) {
    if let Ok(mut state) = STATE.lock() {
        match serde_impl::from_slice::<(State, usize)>(&data) {
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

    let zoom_mul = ZOOM.lock();
    if zoom_mul.is_err() { return; }
    let zoom_mul = zoom_mul.unwrap().1;


    if let Ok(state) = STATE.lock() {
        let mut me_id = state.1;
        let mut is_me = true;
        while let Some(id) = state.0.eaten_by.get(&me_id) {
            me_id = *id;
            is_me = false;
        }

        if is_me {
            put_bg((255, 255, 255));
        } else {
            put_bg((25, 25, 25));
        }

        let my_pos = state.0.players.get(&me_id).map(|x| x.pos).unwrap_or((0., 0.));
        let my_size = state.0.players.get(&me_id).map(|x| x.show_size).unwrap_or(10.);

        let zoom = 0.015 * zoom_mul / (my_size.sqrt() + 2.) * (size.0 + size.1) as f64;

        // Grid lines
        let x_scroll = (my_pos.0 / LINE_SPACE - ((my_pos.0 / LINE_SPACE) as i64) as f64) * LINE_SPACE;
        let lines = (size.0 as f64 / (zoom * LINE_SPACE)) as i64 + 2;
        for x in -lines / 2..lines / 2 + 1 {
            let x = x as f64;
            put_line(
                (x * zoom * LINE_SPACE - x_scroll * zoom + size.0 as f64 / 2., 0.),
                (x * zoom * LINE_SPACE - x_scroll * zoom + size.0 as f64 / 2., size.1 as f64),
                1.,
                if is_me { (200, 200, 200) }
                    else { (100, 100, 100) }
            );
        }
        let y_scroll = (my_pos.1 / LINE_SPACE - ((my_pos.1 / LINE_SPACE) as i64) as f64) * LINE_SPACE;
        let lines = (size.1 as f64 / (zoom * LINE_SPACE)) as i64 + 2;
        for y in -lines / 2..lines / 2 + 1 {
            let y = y as f64;
            put_line(
                (           0., y * zoom * LINE_SPACE - y_scroll * zoom + size.1 as f64 / 2.),
                (size.0 as f64, y * zoom * LINE_SPACE - y_scroll * zoom + size.1 as f64 / 2.),
                1.,
                if is_me { (200, 200, 200) }
                    else { (100, 100, 100) }
            );
        }


        // Draw west wall
        put_line(
            (-(my_pos.0 * zoom) + size.0 as f64 / 2., 0.),
            (-(my_pos.0 * zoom) + size.0 as f64 / 2., size.1 as f64),
            2.,
            if is_me { (100, 100, 100) }
                else { (200, 200, 200) }
        );
        // Draw east wall
        put_line(
            (-((my_pos.0 - state.0.size.0) * zoom) + size.0 as f64 / 2., 0.),
            (-((my_pos.0 - state.0.size.0) * zoom) + size.0 as f64 / 2., size.1 as f64),
            2.,
            if is_me { (100, 100, 100) }
                else { (200, 200, 200) }
        );

        // Draw north wall
        put_line(
            (           0., -(my_pos.1 * zoom) + size.1 as f64 / 2.),
            (size.0 as f64, -(my_pos.1 * zoom) + size.1 as f64 / 2.),
            2.,
            if is_me { (100, 100, 100) }
                else { (200, 200, 200) }
        );
        // Draw east wall
        put_line(
            (           0., -((my_pos.1 - state.0.size.1) * zoom) + size.1 as f64 / 2.),
            (size.0 as f64, -((my_pos.1 - state.0.size.1) * zoom) + size.1 as f64 / 2.),
            2.,
            if is_me { (100, 100, 100) }
                else { (200, 200, 200) }
        );


        for ball in &state.0.balls {
            put_circle(
                ((ball.pos.0 - my_pos.0) * zoom + size.0 as f64 / 2.,
                 (ball.pos.1 - my_pos.1) * zoom + size.1 as f64 / 2.),
                zoom,
                ball.color,
                if is_me { (0, 0, 0) }
                    else { (255, 255, 255) }
            );
        }

        let sorted_players = &state.0.players.values()
                .sorted_by(|x, y| PartialOrd::partial_cmp(&x.size, &y.size).unwrap_or(Ordering::Less));
        for player in sorted_players {
            put_circle(
                ((player.pos.0 - my_pos.0) * zoom + size.0 as f64 / 2.,
                 (player.pos.1 - my_pos.1) * zoom + size.1 as f64 / 2.),
                player.show_size * zoom,
                player.color,
                if is_me { (0, 0, 0) }
                    else { (255, 255, 255) }
            );
        }
    }
}
