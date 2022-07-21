pub const C: f64 = 261.63;
pub const C_SHARP_D_FLAT: f64 = 277.18;
pub const D: f64 = 293.66;
pub const D_SHARP_E_FLAT: f64 = 311.13;
pub const E: f64 = 329.63;
pub const F: f64 = 349.23;
pub const F_SHARP_G_FLAT: f64 = 369.99;
pub const G: f64 = 392.0;
pub const G_SHARP_A_FLAT: f64 = 415.30;
pub const A: f64 = 440.0;
pub const A_SHARP_B_FLAT: f64 = 466.16;
pub const B: f64 = 493.88;

pub fn on_octave(note: f64, octave: u8) -> f64 {
    note * (2f64.powf(((octave as i8) - 4) as f64))
}
