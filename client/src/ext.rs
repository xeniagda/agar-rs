use wasm_bindgen::prelude::*;

#[wasm_bindgen(module="./ext")]
pub extern {
    pub fn put_char_3(x: f64, y: f64, ch: usize, fr: u8, fg: u8, fb: u8);
    pub fn put_circle_3(x: f64, y: f64, r: f64, fr: u8, fg: u8, fb: u8);
    pub fn put_line_3(x1: f64, y1: f64, x2: f64, y2: f64, r: f64, fr: u8, fg: u8, fb: u8);
    pub fn clear();
    pub fn log(text: &str);
    pub fn rand() -> f64;
    pub fn atan2(y: f64, x: f64) -> f64;
    pub fn ws_send(msg: Vec<u8>);
}

pub fn put_char(pos: (f64, f64), ch: usize, col: (u8, u8, u8)) {
    put_char_3(pos.0, pos.1, ch, col.0, col.1, col.2);
}

pub fn put_circle(pos: (f64, f64), r: f64, col: (u8, u8, u8)) {
    put_circle_3(pos.0, pos.1, r, col.0, col.1, col.2);
}

pub fn put_line(start: (f64, f64), end: (f64, f64), r: f64, col: (u8, u8, u8)) {
    put_line_3(start.0, start.1, end.0, end.1, r, col.0, col.1, col.2);
}
