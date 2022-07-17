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
                    samplers: vec![(0.5, Box::new(Sine { freq: note }))],
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

    const GODFATHER:&'static str = "v110o4t80l8<c>cd+cf2<c>cd+cf2cd+gd+fg+b+g+cd+gd+cd+gd+cd+gd+cd+gd+fg+b+g+fg+b+g+fg+b+g+fg+b+g+cd+gd+cd+gd+<g>cd<bgb>dc-cd+gd+cd+gd+<a+>dfd<a+>dfdd+ga+gd+ga+gc+fg+ffg+b+g+gb>d<bgb>d<bcd+gd+fg+b+g+cd+gd+cd+gd+cd+gd+fg+b+g+fg+b+g+fg+b+g+fg+b+g+fg+b+g+cd+gd+cd+gd+<g>cd<bgb>dc-cd+gd+c4,v120o4r1r2l8r>g>cd+dcd+cdc<g+a+g2rg>cd+dcd+cdc<gf+f2rfg+>cd2r<fg+bb+2rcd+a+g+ga+g+g+ggc-c2r>cc<ba+2>d4c<g+g2rga+gf2rfg+f+g2rg>cd+dcd+cdc.<g+16a+g2rg>cd+dcd+cdc.<g16f+f2rfg+b>d2r<fg+bb+2rcd+a+g+ga+g+g+g.g16b>c2.,v90o4l2<gg+gg+gg+l4gfd+2g2b+d+g+cfl8cd<f>cfg+b+4cd<cg>cd+l2ggfd+4<g4cl16>d.fa+.f4<a+.>fg+.a+4l8<d+a+4.d+>d+gd+l16<f.>c+f.g+4<g+.>df.g+4l8<g>dgdl4grg2g+2gfd+2g2g+d+g+gg+l8cd<f>cfg+b+4cd<cg>cd+l4grg2g2c<gc";

    let mut subsongs: Vec<(f64, Box<dyn Sampler>)> = vec![];
    for subsong_text in GODFATHER.split(",") {
        let mut oct = 4;
        let mut length = 1;
        let mut tempo = 80;
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
                note => {
                    if let Some(freq) = notes.get(note) {
                        let dotted = &cap[3] == ".";
                        let freq = on_octave(*freq, oct);
                        let l = 320.0 / tempo as f64 / cap[2].parse().unwrap_or(length) as f64
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

    Ok(())
}
