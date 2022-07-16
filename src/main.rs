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

pub trait Sampler {
    fn sample(&self, t: f64) -> f64;
}

pub struct Sine {
    pub freq: f64,
}

impl Sampler for Sine {
    fn sample(&self, t: f64) -> f64 {
        (t * 2.0 * std::f64::consts::PI * self.freq).sin()
    }
}

pub struct Move {
    pub sampler: Box<dyn Sampler>,
    pub low: f64,
    pub high: f64,
}

impl Sampler for Move {
    fn sample(&self, t: f64) -> f64 {
        (self.sampler.sample(t) + 1.0) / 2.0 * (self.high - self.low) + self.low
    }
}

pub struct Square {
    pub freq: f64,
}

impl Sampler for Square {
    fn sample(&self, t: f64) -> f64 {
        if (t * self.freq).floor() as i64 % 2 == 0 {
            1.0
        } else {
            -1.0
        }
    }
}

pub struct AmplitudeModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for AmplitudeModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.modulator.sample(t)
    }
}

pub struct InputShiftModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for InputShiftModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.modulator.sample(t))
    }
}

pub struct InputGainModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for InputGainModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t * self.modulator.sample(t))
    }
}

pub struct Compound {
    pub samplers: Vec<(f64, Box<dyn Sampler>)>,
}

impl Compound {
    pub fn play(events: Vec<(f64, Box<dyn Sampler>)>) -> Self {
        Compound {
            samplers: events
                .into_iter()
                .map(|(d, s)| -> (f64, Box<dyn Sampler>) {
                    (
                        1.0,
                        Box::new(Delay {
                            delay: d,
                            sampler: s,
                        }),
                    )
                })
                .collect(),
        }
    }
}

impl Sampler for Compound {
    fn sample(&self, t: f64) -> f64 {
        let mut s = 0f64;
        for (vol, sampler) in self.samplers.iter() {
            s += sampler.sample(t) * vol;
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
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for ADSR {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t)
            * (if t < 0.0 {
                0.0
            } else if t < self.attack_length {
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

pub struct Delay {
    pub delay: f64,
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for Delay {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t - self.delay)
    }
}

pub struct Gain {
    pub gain: f64,
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for Gain {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.gain
    }
}

pub trait Instrument {
    fn play(note: f64) -> Box<dyn Sampler>;
}

struct DummyInstrument;

impl Instrument for DummyInstrument {
    fn play(note: f64) -> Box<dyn Sampler> {
        Box::new(ADSR {
            sampler: Box::new(Compound {
                samplers: vec![
                    (0.5, Box::new(Square { freq: note })),
                    (0.5, Box::new(Sine { freq: note })),
                ],
            }),
            attack_length: 0.1,
            decay_length: 2.0,
            sustain_length: 0.0,
            release_length: 0.2,
            sustain_level: 0.6,
        })
    }
}

fn main() -> Result<(), std::io::Error> {
    const SAMPLE_RATE: usize = 44100;
    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let music = InputShiftModulator {
        sampler: Box::new(Sine { freq: 600.0 }),
        modulator: Box::new(Move {
            sampler: Box::new(Sine { freq: 1.0 }),
            low: 0.8,
            high: 1.2,
        }),
    };
    let mut t = 0f64;
    loop {
        out(music.sample(t))?;
        t += SAMPLE_RATE_STEP;
    }
}
