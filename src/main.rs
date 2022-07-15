use std::io::Write;

fn out(sample: f64) -> Result<(), std::io::Error> {
    let val = (sample * 32767.0) as i16;
    std::io::stdout().write(&val.to_le_bytes())?;
    Ok(())
}

const C: f64 = 261.63;
const C_SHARP_D_FLAT: f64 = 277.18;
const D: f64 = 293.66;
const D_SHARP_E_FLAT: f64 = 311.13;
const E: f64 = 329.63;
const F: f64 = 349.23;
const F_SHARP_G_FLAT: f64 = 369.99;
const G: f64 = 392.0;
const G_SHARP_A_FLAT: f64 = 415.30;
const A: f64 = 440.0;
const A_SHARP_B_FLAT: f64 = 466.16;
const B: f64 = 493.88;

fn main() -> Result<(), std::io::Error> {
    const SAMPLE_RATE: usize = 44100;
    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let mut t = 0f64;
    loop {
        for v in [C, G, G, A, A, G, F, F, E, E, D, D, C] {
            for _ in 0..SAMPLE_RATE / 2 {
                out((t * 6.28 * v).sin())?;
                t += SAMPLE_RATE_STEP;
            }
        }
    }
}
