use dyn_clone::DynClone;
use rayon::prelude::*;
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

pub trait Sampler: DynClone + Send + Sync {
    fn sample(&self, t: f64) -> f64;
}

dyn_clone::clone_trait_object!(Sampler);

#[derive(Clone)]
pub struct Const {
    pub val: f64,
}

impl Sampler for Const {
    fn sample(&self, _t: f64) -> f64 {
        self.val
    }
}

#[derive(Clone)]
pub struct Record {
    pub sample_rate: f64,
    pub samples: Vec<f64>,
}

impl Record {
    fn record(sampler: Box<dyn Sampler>, sample_rate: f64, duration: f64) -> Self {
        let mut samples = Vec::new();
        let mut t = 0f64;
        let step = 1f64 / sample_rate;
        for i in 0..(duration * sample_rate) as usize {
            samples.push(sampler.sample(t));
            t += step;
        }
        Self {
            sample_rate,
            samples,
        }
    }
}

impl Sampler for Record {
    fn sample(&self, t: f64) -> f64 {
        let ind = (t * self.sample_rate) as usize;
        *self.samples.get(ind).unwrap_or(&0f64)
    }
}

#[derive(Clone)]
pub struct Sine {
    pub freq: f64,
}

impl Sampler for Sine {
    fn sample(&self, t: f64) -> f64 {
        (t * 2.0 * std::f64::consts::PI * self.freq).sin()
    }
}

#[derive(Clone)]
pub struct Sawtooth {
    pub freq: f64,
}

impl Sampler for Sawtooth {
    fn sample(&self, t: f64) -> f64 {
        let t = t * self.freq;
        (t - t.floor()) * 2.0 - 1.0
    }
}

#[derive(Clone)]
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct AmplitudeModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for AmplitudeModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.modulator.sample(t)
    }
}

#[derive(Clone)]
pub struct InputShiftModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for InputShiftModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.modulator.sample(t))
    }
}

#[derive(Clone)]
pub struct InputGainModulator {
    modulator: Box<dyn Sampler>,
    sampler: Box<dyn Sampler>,
}

impl Sampler for InputGainModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t * self.modulator.sample(t))
    }
}

#[derive(Clone)]
pub struct Compound {
    pub samplers: Vec<(f64, Box<dyn Sampler>)>,
}

impl Compound {
    pub fn reverb(sampler: Box<dyn Sampler>, count: usize, pow: f64, length: f64) -> Self {
        Compound {
            samplers: (0..count)
                .into_iter()
                .map(|i| -> (f64, Box<dyn Sampler>) {
                    (
                        pow.powf(i as f64),
                        Box::new(Shift {
                            shift: -(i as f64 / count as f64 * length),
                            sampler: sampler.clone(),
                        }),
                    )
                })
                .collect(),
        }
    }

    pub fn unison<F>(pitch: f64, count: usize, creator: F) -> Self
    where
        F: Fn(f64) -> Box<dyn Sampler>,
    {
        if count % 2 == 0 {
            panic!("Not supported!");
        }
        let pows = -(count as isize / 2)..(count as isize / 2 + 1);
        Compound {
            samplers: pows
                .into_iter()
                .map(|p| pitch * 2f64.powf(p as f64))
                .map(|f| (1.0, creator(f)))
                .collect(),
        }
    }
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

#[derive(Clone)]
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

#[derive(Clone)]
pub struct Shift {
    pub shift: f64,
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for Shift {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.shift)
    }
}

#[derive(Clone)]
pub struct Gain {
    pub gain: f64,
    pub sampler: Box<dyn Sampler>,
}

impl Sampler for Gain {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.gain
    }
}

#[derive(Clone)]
pub struct Filter {
    pub step: f64,
    pub sampler: Box<dyn Sampler>,
}

lazy_static::lazy_static! {
     static ref CONCERT_HALL_FILTER: Vec<f64> = std::fs::read("hall.raw")
         .unwrap()
         .chunks(2)
         .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
         .map(|i| (i as f64) / 32767.0)
         .collect();
}

impl Filter {
    fn read(sampler: Box<dyn Sampler>, path: &str) -> Self {
        Self {
            sampler,
            step: 1.0 / 44100.0,
        }
    }
}
impl Sampler for Filter {
    fn sample(&self, t: f64) -> f64 {
        if t > -2.0 && t < 2.0 {
            let sm: f64 = CONCERT_HALL_FILTER
                .par_iter()
                .enumerate()
                .map(|(i, v)| v * self.sampler.sample(t - (i as f64 * self.step)))
                .sum();
            sm
        } else {
            0.0
        }
    }
}

#[derive(Clone)]
pub struct Impulse;
impl Sampler for Impulse {
    fn sample(&self, t: f64) -> f64 {
        if t > 0.01 && t < 0.01 {
            1.0
        } else {
            0.0
        }
    }
}

pub trait Instrument {
    fn play(note: f64, length: f64, volume: f64) -> Box<dyn Sampler>;
}

struct DummyInstrument;

impl Instrument for DummyInstrument {
    fn play(note: f64, length: f64, volume: f64) -> Box<dyn Sampler> {
        let snd = Box::new(ADSR {
            sampler: Box::new(Sine { freq: note }),
            attack_length: 0.1,
            decay_length: 0.1,
            sustain_length: 0.0,
            release_length: 0.1,
            sustain_level: 1.0,
        });
        Box::new(Filter::read(snd, "example.raw"))
    }
}

struct LegitInstrument;

impl Instrument for LegitInstrument {
    fn play(note: f64, length: f64, volume: f64) -> Box<dyn Sampler> {
        Box::new(Gain {
            sampler: Box::new(ADSR {
                sampler: Box::new(AmplitudeModulator {
                    sampler: Box::new(Compound {
                        samplers: vec![(
                            0.1,
                            Box::new(Compound::unison(note, 7, |f| Box::new(Sine { freq: f }))),
                        )],
                    }),
                    modulator: Box::new(Move {
                        sampler: Box::new(Sine { freq: 4.0 }),
                        low: 0.3,
                        high: 1.0,
                    }),
                }),
                attack_length: 0.5,
                decay_length: length / 3.0,
                sustain_length: length / 3.0,
                release_length: length / 3.0,
                sustain_level: 0.5,
            }),
            gain: volume,
        })
    }
}

use regex::Regex;

use std::collections::HashMap;

const AIR_ON_G_STRING: &'static str = "t33>e1&e8a16f16e32d32c16<b16>c16<b4a16g8.>g2&g16e16<a+16a16>d16c+16g16f16f2&f16d16<a16g16>c16<b16>f16e16e4.f+16g16c8c32d32e8d16d16c16<b16a16a32b32>c8.<b16a16g2>e1&e8a16f16e32d32c16<b16>c16<b4a16g8.>g2&g16e16<a+16a16>d16c+16g16f16f2&f16d16<a16g16>c16<b16>f16e16e4.f+16g16c8c32d32e8d16d16c16<b16a16a32b32>c8.<b16a16g4.&g16,<c8>c8<b8<b8a8>a8g8<g8f8>f8f+8<f+8g8>g8f8<f8e8>e8d8<d8c+8>c+8<a8>a8<d8>d8c8<c8<b8>b8g8>g8c8>c8<b8<b8a8>a8f+8d8g8c8d8<d8g16a16b16>c16d16f16e16d16c8>c8<b8<b8a8>a8g8<g8f8>f8f+8<f+8g8>g8f8<f8e8>e8d8<d8c+8>c+8<a8>a8<d8>d8c8<c8<b8>b8g8>g8c8>c8<b8<b8a8>a8f+8d8g8c8d8<d8g4.&g16";

const MARIO:&'static str = "T180V110L16>c8dre-rfrgrr8>crr8<b-rr8grr8ab-a4.b-8r8grarb-8r8arfrdrr8e-rr8de-d4r8c8dre-rfre-8r8dre-rf4e-rdrc2f8r8e-rdrc8r8dre-rf8r8e-rfrg2,O4crrrgrrrcrrrgrrrcrrrgrrrfrrr>crrr<<e-rrrb-rrre-rrrb-rrrfrrr>crrr<b-rrr>frrr<a-rrr>e-rrr<a-rrr>e-rrr<a-rrr>e-rrr<a-rrr>e-rrr<b-rrr>frrr<b-rrr>frrr<b-rrr>frrr<grrr>drrr";

const STAIRWAY_TO_HEAVEN:&'static str = "t75<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c4.<<a8>f8e8<a8>a8>c8e8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2<<a8b8>c8e8g8>e8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8b8>>c8<g8e8>c8g8<b8g8>g8g16f+16f+8f+2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8b8>c8e8g8>c8<d8a8>d8f+8e8e8e2.<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2.<a8>c8e8a8b8e8c8b8>c8<e8c8>c8<f+8d8<a8>f+8e8c8<a8>c4e8c8<a8b8>c8c2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a16.>e32c8<b8a8<a8>g8>c8<g8e8>c8g8<b8g8>g8g16f+16f+8f+2<<a8b8>c8e8g8>c8f+8d8<a8>f+8e8c8<a8>e8<b8a8<a8>g8>c8<g8e8>c8f+8d8<a8>f+8e8e8e2,r2<g+2g2f+2f2&f8>c4.<g8a8a4.a2.&a8g+2g2f+4.>d8<f1g8a8a1&a4d2f2<a2>c2<g2>d8>d8d1&d4<d2f2<a1&a2>>c8c8c1&c4<g+2g2f+2f1g8a8a1&a4g+2g2f+2f1g8a8a1&a4d2f2<a4.b8>c2<g2>d8a8a1&a4d2f2.&f8<b8>c2d2>c8c8c2,r1r1r1o2b8a8a1&a1&a1&a2.b8a8a1&a1&a1&a2.&a8>d8d1&d1&d1&d2.f8f8f1&f1&f1&f2.<b8a8a1&a1&a1&a2.b8a8a1&a1&a1&a2.&a8>d8d1&d1&d1&d2.f8f8f2;";

const SMOKE_ON_THE_WATER:&'static str = "v127l8t112gra#r>c4<rgra#r>c#c4<r4gra#r>c4<ra#rg2r4.gra#r>c4<rgra#r>c#c4<r4gra#r>c4<ra#rg2r4.gra#r>c4<rgra#r>c#c4<r4gra#r>c4<ra#rg2r4.gra#r>c4<rgra#r>c#c4<r4gra#r>c4<ra#rg2r4.,r1r1r1r1l8v127t112drfrg4rdrfrg#g4r4drfrg4rfrd2r4.drfrg4rdrfrg#g4r4drfrg4rfrd2r4.drfrg4rdrfrg#g4r4drfrg4rfrd2r4.,v127t112r1r1r1r1r1r1r1r2l8r<<<eff#gggggggggggggggggggg>ccc<a#4>c<ggggff#gggggggggggggggggggg>ccc<a#4>c<ggggg4";

const CREEP_RADIOHEAD:&'static str = "t93l8r1r1r1r4.d+4r1r1r1r1r1r1r1r1r1r1r1r1r1r4.g4r1f+r1r1r1r1r1r1r1r1r1r1r1r2rd+1&d+1r1r1r1r1r1r1r1r1r1r1r1r1r1r1rg4,r1d<b4b4b>d4r4d+4<b>d+4.e4.<b4r>d+<br1r1r1r1r1r1r1>d+<b4b4.>d+4r1r1r1r1r1d4<b>d4<b>g4r4d+<b>d+f+d+f+bf+d+f+4d+b4r4ececge>c<geg4e>c<grgd+g4gd+crd+4rd+2r1r1r2.d+16e16d+4r1r2re16f16e4r2.rc1&c1r1r1r1r1r1r1r1r1r1r1r1bf+d+<b4>d+<b>d+rcecee4c>c<gec4ec4rd+4rfd+4cgd+4d+2&d+r1r1r2.d+16e16d+4r1r2re16f16e4r1r2rd+16f16d+1&d+r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1d+2.&d+.r16d+4.r1r1b4.a4,o2g4.&g16g16gg4gg4.&g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<b>c4.&c16<b16>cc4<b>c4d4d+4<f4g4.&g16g16gg4gg4.g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<b>c4.&c16<b16>cc4<b>c4d4d+4f4<g4.&g16g16gg4gg4.&g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<b>c4.&c16<b16>cc4<b>c4d4d+4f4<g4.&g16g16gg4gg4.&g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<g>c1<g4.&g16g16gg4gg4.&g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<b>c4.&c16<b16>cc4<b>c4d4d+4<f4g4.&g16g16gg4gg4.g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<b>c4.&c16<b16>cc4<b>c4d4d+4f4<g4.&g16g16gg4gg4.&g16g16gg4gb4.&b16f+16bb4f+b4.&b16f+16bb4b>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<g>c4.&c16<g16>cc4<g>c4d4d+4f4<g4.&g16g16gg4>d<g4.g16g16g16a16g4f+b4.&b16f+16bb4f+b4.f+f+16g+16f+4f>c4.c16<g16>cc4<g>ccccc16d16cc4ccccccccccddd+d+ffg4.&g16g16gg4gg4.g16g16g16a16g4.<b4b.b16bb4.b4b.b16bbb>dc4c.c16ccccc4c.c16cccdc1&c1<g2.&ggg1b2.&bbb1>c2.&ccc1c2.&ccc1<g2.&ggg2.&ggb2.&bbb2.&bb>c2.&ccc2.&ccc1&c1ga4a16b1&b2&b16,t93l8v115r1r1r1r1r1r1r1r2a16a16gf+g4.r1rdaggf+4.r1r.d16agf+g4e4.r1agf+g4.r1rcagf+g4d4c16<b4&b16r2rb16b16>a16a16g4f+2r1r16a16aaga+4g4.r2.rcgaga+4g4.r2.r.g16gb4b4.r1r4gb4b4f+4e16d+4&d+16r2bb16>c.<bb16ab4&b16r1r4gaga+4g2r2.r16c16agf+g4.r1r.d16a16a16ggf+4.r1r.d16a16a16gf+g4e4d16c4.r2rd16a16a16gf+g4.r1rcagf+g4d4c16<b4&b16r2r.>d16aggf+4.r1r4gagb4g4.r2.rggaga+4g2r2.r16g16gb4b4.r1r4gb4b4f+4e16d+4&d+16r2r16bb16>c<bb16ab2.&b16r2.gaga+4g2r4>d4c4d4cr4g2.r4a4gf+r4dd+4&d+16f+.b2f+16e16d+4r2.g2.r4a4gg4.rdd+4r4f4r4g4r4ga1&a4.g1f+4r1r2.a1g2f+2g4r1r.<d16agf+g4d4r1r16a16a16a16g4f+4.r1rdgagb4g4.r2.rggaga+4g2r2.r16g16gb4b4.r1r4gb4b4f+4e16d+4&d+16r2bb16>c.<bb16ab4&b16r1r4gaga+4g2r2.rgagb4g2,t93l8r1r1r1r4.d+4l1rrrrrrrrrrrrrr4l8.g4r1f+l1rrrrrrrrrrrr2l8rd+1&d+l1rrrrrrrrrrrrrrrl8g4,r1d<b4b4b>d4r4d+4<b>d+4.e4.<b4r>d+<br1r1r1r1r1r1r1>d+<b4b4.>d+4r1r1r1r1r1d4<b>d4<b>g4r4d+<b>d+f+d+f+bf+d+f+4d+b4r4ececge>c<geg4e>c<grgd+g4gd+crd+4rd+2r1r1r2.d+16e16d+4r1r2re16f16e4r2.rc1&c1r1r1r1r1r1r1r1r1r1r1r1bf+d+<b4>d+<b>d+rcecee4c>c<gec4ec4rd+4rfd+4cgd+4d+2&d+r1r1r2.d+16e16d+4r1r2re16f16e4r1r2rd+16f16d+1&d+r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1d+2.&d+.r16d+4.r1r1b4.a4,r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1<d2.r1r4f+2.r1r4g2.r1r4g1&g1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1d2.r1r4f+2.r1r4g2.r1r4g2.r1r4ddddddddddddddddf+f+f+f+f+f+f+f+f+f+f+f+f+f+f+f+ggggggggggggggggggggggggggggggggddddddddddddddddf+f+f+f+f+f+f+f+f+f+f+f+f+f+f+f+ggggggggggggggggg1&g1r1r1r1r1r1r1r1r2g4r4g2.&g.r16g4.g4gg4b2.&b.r16b4.r2re2&e.r16e4e4.b4.a4d+2.&d+16r.d+4.,r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1<g2.r1r4b2.r1r4>c2.r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r4<g2.r1r4b2.r1r4>c2.r1r4c2.r1r4<ggggggggggggggggbbbbbbbbbbbbbbbb>cccccccccccccccccccccccccccccccc<ggggggggggggggggbbbbbbbbbbbbbbbb>ccccccccccccccccc1&c1r1r1r1r1r1r1r1r2c4r4<b2.&b.r16>c4.<b4bb4r1r1g2&g.r16a4g4.r2rg2.&g16,r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1<b2.r1r4>d+2.r1r4e2.r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r1r4<b2.r1r4>d+2.r1r4e2.r1r4d+2.r1r4<bbbbbbbbbbbbbbbb>d+d+d+d+d+d+d+d+d+d+d+d+d+d+d+d+eeeeeeeeeeeeeeeed+d+d+d+d+d+d+d+d+d+d+d+d+d+d+d+<bbbbbbbbbbbbbbb>d+d+d+d+d+d+d+d+d+d+d+d+d+d+d+d+eeeeeeeeeeeeeeeed+1&d+1,o3g>dg4gb4.r1<b>f+r1r4.f+r4cg>e<g>ce4<g>f4<g>e4<g>ec<g>d+c<g>d+c<g>c<g>d+c<g>d+c<g4<g>dgdgg4db4gd4db4>d+<f+b4f+b4f+r1cg>c<cg>c4<g>ec<g>c4<g>cd+<c>c<g>cd+c<g>cd+<g>c<g4>d+c4<<g>dbdgb4br1f+br1r2.cgr1r2.cr2.rgr4gr2<g2.>b16>c16<b4g4d4<g4.b2.r4.>b4f+4<b4.>c2.r4.>c4<g4c4.c1&c1<g4>d4gb4g4<g>gd2f+4b4f+b4f+r>d+<bf+b4f+b4cg>c4<g>c4<g>ec<g>c4<g>ec<cg>c<g>d+c<g>c<cg>c<g>d+c<g4<g>dgdgb4d>gd<bg4gb4f+b>d+<b>d+4<br1rcr1r2.rcr4gr1r2<g2.>b16>c16<b4g4d4<g4.b2.r4.>b4f+4<b4.>c2.r4.>c4<g4c4.c2.r1r4<ggggggggggggggggbbbbbbbbbbbbbbbb>cccccccccccccccccccccccccccccccc<ggggggggggggggggbbbbbbbbbbbbbbbb>ccccccccccccccccc1&c1<g>dgdgbgd<g>dgdgb4.<b>f+bf+b>d+4<b4<b>f+bf+>d+4<f+cg>c<g>ce4c4<g>c<g>ce4c<cg>c<g>cd+4c4<cg4c4r4d2.&d.r16d4.d4dd4f+2.&f+.r16f+4.b4a<b>f+c2&c.r16c4c4.g4.g4c2.&c16r16cc4.b4.a4";

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
    let mut volume = 120;
    for subsong_text in AIR_ON_G_STRING.replace("#", "+").to_lowercase().split(",") {
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
                "v" => {
                    volume = cap[2].parse().unwrap();
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
                        music.push((time, LegitInstrument::play(freq, l, volume as f64 / 200.0)));
                        time += l;
                    }
                }
            }
        }
        subsongs.push((0.0, Box::new(Compound::play(music))))
    }

    let music = Box::new(Compound::play(subsongs));

    const SAMPLE_RATE: usize = 44100;

    let music = Record::record(music, SAMPLE_RATE as f64, 10.0);

    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let mut t = 0f64;
    loop {
        out(music.sample(t))?;
        t += SAMPLE_RATE_STEP;
    }
}
