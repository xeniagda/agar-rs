use std::f64::consts::PI;

const TAU: f64 = PI * 2.;
const PREC: usize = 5;

pub fn sin(v: f64) -> f64 {
    let v = v - ((v / TAU) as u64) as f64 * TAU; // Ugly version of v % TAU
    if v > TAU / 2. { return -sin(v - TAU / 2.); }
    if v > TAU / 4. { return sin(TAU / 2. - v); }

    let mut sum = 0.;
    let mut fact = 1.;
    let mut v_pow = v;
    for i in 0..PREC {
        let mul_idx = (2 * i + 2) as f64;

        sum += v_pow / fact;

        v_pow *= v * v;
        fact *= -mul_idx * (mul_idx + 1.);
    }

    sum
}

pub fn cos(v: f64) -> f64 {
    sin(v + TAU / 4.)
}

#[test]
fn test_sin() {
    for x in 0..30 {
        let v = x as f64 / 30. * TAU;
        assert!((sin(v) - v.sin()).abs() < 4e-6);
    }
}

#[test]
fn test_cos() {
    for x in 0..30 {
        let v = x as f64 / 30. * TAU;
        assert!((cos(v) - v.cos()).abs() < 4e-6);
    }
}
