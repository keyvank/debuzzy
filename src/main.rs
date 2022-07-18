use std::io::Write;

fn out(sample: f64) -> Result<(), std::io::Error> {
    let val = (sample / 2.0 * 32767.0) as i16;
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

fn on_octave(note: f64, octave: u8) -> f64 {
    note * (2f64.powf(((octave as i8) - 4) as f64))
}

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
                        Box::new(Shift {
                            shift: -d,
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
        if t < 0.0 {
            0.0
        } else if t < self.attack_length {
            self.sampler.sample(t) * interpolate(0.0, 0.0, self.attack_length, 1.0, t)
        } else if t < self.attack_length + self.decay_length {
            self.sampler.sample(t)
                * interpolate(
                    self.attack_length,
                    1.0,
                    self.attack_length + self.decay_length,
                    self.sustain_level,
                    t,
                )
        } else if t < self.attack_length + self.decay_length + self.sustain_length {
            self.sampler.sample(t) * self.sustain_level
        } else if t < self.attack_length
            + self.decay_length
            + self.sustain_length
            + self.release_length
        {
            self.sampler.sample(t)
                * interpolate(
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
        }
    }
}

pub struct Shift {
    pub shift: f64,
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for Shift {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.shift)
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
    fn play(note: f64, length: f64) -> Box<dyn Sampler>;
}

struct DummyInstrument;

impl Instrument for DummyInstrument {
    fn play(note: f64, length: f64) -> Box<dyn Sampler> {
        Box::new(ADSR {
            sampler: Box::new(AmplitudeModulator {
                sampler: Box::new(Compound {
                    samplers: vec![
                        (0.3, Box::new(Sine { freq: note })),
                        (0.15, Box::new(Sine { freq: note * 2.0 })),
                        (0.15, Box::new(Sine { freq: note / 2.0 })),
                        (0.075, Box::new(Sine { freq: note * 4.0 })),
                        (0.075, Box::new(Sine { freq: note / 4.0 })),
                    ],
                }),
                modulator: Box::new(Move {
                    sampler: Box::new(Sine { freq: 4.0 }),
                    low: 0.3,
                    high: 1.0,
                }),
            }),
            attack_length: 0.1,
            decay_length: length,
            sustain_length: 0.0,
            release_length: 0.6,
            sustain_level: 0.6,
        })
    }
}

use regex::Regex;

use std::collections::HashMap;

const STAIRWAY_TO_HEAVEN:&'static str = "t75<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c4.<<a8>f8e8<a8>a8>c8e8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2<<a8b8>c8e8g8>e8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8b8>>c8<g8e8>c8g8<b8g8>g8g16f+16f+8f+2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8b8>c8e8g8>c8<d8a8>d8f+8e8e8e2.<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2.<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a16.>e32c8<b8a8<a8>g8>c8<g8e8>c8g8<b8g8>g8g16f+16f+8f+2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8>g8>c8<g8e8>c8f+8d8<a8>f+8e8e8e2,r2<g+2g2f+2f2&f8>c4.<g8a8a4.a2.&a8g+2g2f+4.>d8<f1g8a8a1&a4d2f2<a2>c2<g2>d8>d8d1&d4<d2f2<a1&a2>>c8c8c1&c4<g+2g2f+2f1g8a8a1&a4g+2g2f+2f1g8a8a1&a4d2f2<a4.b8>c2<g2>d8a8a1&a4d2f2.&f8<b8>c2d2>c8c8c2,r1r1r1o2b8a8a1&a1&a1&a2.b8a8a1&a1&a1&a2.&a8>d8d1&d1&d1&d2.f8f8f1&f1&f1&f2.<b8a8a1&a1&a1&a2.b8a8a1&a1&a1&a2.&a8>d8d1&d1&d1&d2.f8f8f2;";

fn main() -> Result<(), std::io::Error> {
    let notes: HashMap<&str, f64> = [
        ("c", C),
        ("c+", C_SHARP_D_FLAT),
        ("d-", C_SHARP_D_FLAT),
        ("d", D),
        ("d+", D_SHARP_E_FLAT),
        ("e-", D_SHARP_E_FLAT),
        ("e", E),
        ("f", F),
        ("f+", F_SHARP_G_FLAT),
        ("g-", F_SHARP_G_FLAT),
        ("g", G),
        ("g+", G_SHARP_A_FLAT),
        ("a-", G_SHARP_A_FLAT),
        ("a", A),
        ("a+", A_SHARP_B_FLAT),
        ("b-", A_SHARP_B_FLAT),
        ("b", B),
        ("b+", C),
        ("c-", B),
        ("p", 0.0),
        ("r", 0.0),
    ]
    .into_iter()
    .collect();

    let mut subsongs: Vec<(f64, Box<dyn Sampler>)> = vec![];
    let mut oct = 4;
    let mut length = 1;
    let mut tempo = 80;
    for subsong_text in STAIRWAY_TO_HEAVEN.replace("#", "+").split(",") {
        let re = Regex::new(r"(\D\+?\-?\#?)(\d*)(\.?)").unwrap();
        let mut music = vec![];
        let mut time = 0f64;
        for cap in re.captures_iter(subsong_text) {
            match cap[1].to_string().as_str() {
                "o" => {
                    oct = cap[2].parse().unwrap();
                }
                "t" => {
                    tempo = cap[2].parse().unwrap();
                }
                "l" => {
                    length = cap[2].parse().unwrap();
                }
                ">" => {
                    oct += 1;
                }
                "<" => {
                    oct -= 1;
                }
                "&" => {}
                note => {
                    if let Some(freq) = notes.get(note) {
                        let dotted = &cap[3] == ".";
                        let freq = on_octave(*freq, oct);
                        let l =
                            320.0 / (tempo as f64) / cap[2].parse::<f64>().unwrap_or(length as f64)
                                * if dotted { 1.5 } else { 1.0 };
                        music.push((time, DummyInstrument::play(freq, l)));
                        time += l;
                    }
                }
            }
        }
        subsongs.push((0.0, Box::new(Compound::play(music))))
    }

    let music = Compound::play(subsongs);

    const SAMPLE_RATE: usize = 44100;
    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let mut t = 0f64;
    loop {
        out(music.sample(t))?;
        t += SAMPLE_RATE_STEP;
    }
}
