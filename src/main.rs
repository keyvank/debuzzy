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

pub trait Sound {
    fn sample(&self, t: f64) -> f64;
}

pub struct Sine {
    pub phase: f64,
    pub freq: f64,
}

impl Sound for Sine {
    fn sample(&self, t: f64) -> f64 {
        (t * 2.0 * std::f64::consts::PI * self.freq).sin()
    }
}

pub struct Compound {
    pub sounds: Vec<Box<dyn Sound>>,
}

impl Sound for Compound {
    fn sample(&self, t: f64) -> f64 {
        let mut s = 0f64;
        for sounds in self.sounds.iter() {
            s += sounds.sample(t) * 0.33;
        }
        s
    }
}

fn interpolate(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    (y2 - y1) / (x2 - x1) * (x - x1) + y1
}

pub struct ADSR {
    pub attack_length: f64,
    pub decay_length: f64,
    pub sustain_length: f64,
    pub release_length: f64,
    pub sustain_level: f64,
    pub sound: Box<dyn Sound>,
}

impl Sound for ADSR {
    fn sample(&self, t: f64) -> f64 {
        self.sound.sample(t)
            * (if t < self.attack_length {
                interpolate(0.0, 0.0, self.attack_length, 1.0, t)
            } else if t < self.attack_length + self.decay_length {
                interpolate(
                    self.attack_length,
                    1.0,
                    self.attack_length + self.decay_length,
                    self.sustain_level,
                    t,
                )
            } else if t < self.attack_length + self.decay_length + self.sustain_length {
                self.sustain_level
            } else if t < self.attack_length
                + self.decay_length
                + self.sustain_length
                + self.release_length
            {
                interpolate(
                    self.attack_length + self.decay_length + self.sustain_length,
                    self.sustain_level,
                    self.attack_length
                        + self.decay_length
                        + self.sustain_length
                        + self.release_length,
                    0.0,
                    t,
                )
            } else {
                0.0
            })
    }
}

fn main() -> Result<(), std::io::Error> {
    const SAMPLE_RATE: usize = 44100;
    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let music = ADSR {
        sound: Box::new(Compound {
            sounds: vec![
                Box::new(Sine {
                    phase: 0.0,
                    freq: A,
                }),
                Box::new(Sine {
                    phase: 0.0,
                    freq: C,
                }),
                Box::new(Sine {
                    phase: 0.0,
                    freq: D,
                }),
            ],
        }),
        attack_length: 0.1,
        decay_length: 2.0,
        sustain_length: 0.0,
        release_length: 0.2,
        sustain_level: 0.6,
    };
    let mut t = 0f64;
    loop {
        out(music.sample(t))?;
        t += SAMPLE_RATE_STEP;
    }
}
