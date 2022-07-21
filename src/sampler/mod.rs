mod sine;
pub use sine::*;

use crate::fft::*;
use dyn_clone::DynClone;
use rayon::prelude::*;
use rustfft::{num_complex::Complex, FftPlanner};

pub trait Sampler: DynClone + Send + Sync {
    fn sample(&self, t: f64) -> f64;
    fn integral(&self) -> Box<dyn Sampler> {
        unimplemented!();
    }
}

pub type DynSampler = Box<dyn Sampler>;

dyn_clone::clone_trait_object!(Sampler);

#[derive(Clone)]
pub struct Quadratic {
    a0: f64,
    a1: f64,
    a2: f64,
}

impl Quadratic {
    pub fn new(a0: f64, a1: f64, a2: f64) -> DynSampler {
        Box::new(Quadratic { a0, a1, a2 })
    }
}

impl Sampler for Quadratic {
    fn sample(&self, t: f64) -> f64 {
        t * t * self.a2 + t * self.a1 + self.a0
    }
}

#[derive(Clone)]
pub struct Line {
    a0: f64,
    a1: f64,
}

impl Line {
    fn new(a0: f64, a1: f64) -> DynSampler {
        Box::new(Self { a0, a1 })
    }
    fn interpolate(a: (f64, f64), b: (f64, f64)) -> DynSampler {
        let a1 = (b.1 - a.1) / (b.0 - a.0);
        let a0 = a.1 - a.0 * a1;
        Self::new(a0, a1)
    }
}

impl Sampler for Line {
    fn sample(&self, t: f64) -> f64 {
        self.a0 + t * self.a1
    }
    fn integral(&self) -> DynSampler {
        Quadratic::new(0.0, self.a0, self.a1 / 2.0)
    }
}

#[derive(Clone)]
pub struct Limit {
    pub sampler: DynSampler,
    pub start: f64,
    pub end: f64,
}

impl Limit {
    pub fn new(sampler: DynSampler, start: f64, end: f64) -> DynSampler {
        Box::new(Limit {
            sampler,
            start,
            end,
        })
    }
}

impl Sampler for Limit {
    fn sample(&self, t: f64) -> f64 {
        if t >= self.start && t <= self.end {
            self.sampler.sample(t)
        } else {
            0.0
        }
    }
    fn integral(&self) -> DynSampler {
        let int = self.sampler.integral();
        let s0 = int.sample(self.start);
        Limit::new(
            Compound::new(vec![(1.0, int), (-1.0, Const::new(s0))]),
            self.start,
            self.end,
        )
    }
}

#[derive(Clone)]
pub struct Const {
    pub val: f64,
}

impl Const {
    pub fn new(val: f64) -> DynSampler {
        Box::new(Const { val })
    }
}

impl Sampler for Const {
    fn sample(&self, _t: f64) -> f64 {
        self.val
    }
    fn integral(&self) -> DynSampler {
        Line::new(0.0, self.val)
    }
}

#[derive(Clone)]
pub struct Record {
    pub sample_rate: f64,
    pub samples: Vec<f64>,
}

impl Record {
    pub fn record(sampler: DynSampler, sample_rate: f64, duration: f64) -> Self {
        let step = 1f64 / sample_rate;

        Self {
            sample_rate,
            samples: (0..(duration * sample_rate) as usize)
                .into_par_iter()
                .map(|i| sampler.sample(i as f64 * step))
                .collect(),
        }
    }
    pub fn apply_filter(&mut self, filter_fft: &Vec<Complex<f64>>) {
        let mut padded = self.samples.clone();
        let chunklen = (filter_fft.len() - 1) * 2;
        while padded.len() % chunklen != 0 {
            padded.push(0.0);
        }

        for data in padded.chunks_mut(chunklen) {
            let mut data_fft = fft(data);
            data_fft
                .iter_mut()
                .zip(filter_fft.iter())
                .for_each(|(d, f)| *d *= f);
            let filtered = ifft(&data_fft);
            data.copy_from_slice(&filtered);
        }
        self.samples = padded;
    }
}

impl Sampler for Record {
    fn sample(&self, t: f64) -> f64 {
        let ind = (t * self.sample_rate) as usize;
        *self.samples.get(ind).unwrap_or(&0f64)
    }
}

#[derive(Clone)]
pub struct Sawtooth {
    pub freq: f64,
}
impl Sawtooth {
    pub fn new(freq: f64) -> DynSampler {
        Box::new(Sawtooth { freq })
    }
}

impl Sampler for Sawtooth {
    fn sample(&self, t: f64) -> f64 {
        let t = t * self.freq;
        (t - t.floor()) * 2.0 - 1.0
    }
}

#[derive(Clone)]
pub struct Move {
    pub sampler: DynSampler,
    pub low: f64,
    pub high: f64,
}

impl Move {
    pub fn new(sampler: DynSampler, low: f64, high: f64) -> DynSampler {
        Box::new(Move { sampler, low, high })
    }
}

impl Sampler for Move {
    fn sample(&self, t: f64) -> f64 {
        (self.sampler.sample(t) + 1.0) / 2.0 * (self.high - self.low) + self.low
    }
    fn integral(&self) -> DynSampler {
        Compound::new(vec![
            ((self.high - self.low) / 2.0, self.sampler.integral()),
            (1.0, Line::new(0.0, -(self.high - 3.0) / 2.0)),
        ])
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
    amplitude: DynSampler,
    sampler: DynSampler,
}

impl AmplitudeModulator {
    pub fn new(sampler: DynSampler, amplitude: DynSampler) -> DynSampler {
        Box::new(AmplitudeModulator { sampler, amplitude })
    }
}

impl Sampler for AmplitudeModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.amplitude.sample(t)
    }
}

#[derive(Clone)]
pub struct InputShiftModulator {
    modulator: DynSampler,
    sampler: DynSampler,
}

impl Sampler for InputShiftModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.modulator.sample(t))
    }
}

#[derive(Clone)]
pub struct FrequencyModulator {
    frequency_integral: DynSampler,
    sampler: DynSampler,
}
impl FrequencyModulator {
    pub fn new(sampler: DynSampler, frequency_integral: DynSampler) -> DynSampler {
        Box::new(FrequencyModulator {
            sampler,
            frequency_integral,
        })
    }
}

impl Sampler for FrequencyModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(self.frequency_integral.sample(t))
    }
}

#[derive(Clone)]
pub struct Compound {
    pub samplers: Vec<(f64, DynSampler)>,
}

impl Compound {
    pub fn new(samplers: Vec<(f64, DynSampler)>) -> DynSampler {
        Box::new(Compound { samplers })
    }
    pub fn adsr(
        attack_length: f64,
        decay_length: f64,
        sustain_length: f64,
        release_length: f64,
        sustain_level: f64,
    ) -> DynSampler {
        Compound::new(vec![
            (
                1.0,
                Limit::new(
                    Line::interpolate((0.0, 0.0), (attack_length, 1.0)),
                    0.0,
                    attack_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Line::interpolate(
                        (attack_length, 1.0),
                        (attack_length + decay_length, sustain_level),
                    ),
                    attack_length,
                    attack_length + decay_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Const::new(sustain_level),
                    attack_length + decay_length,
                    attack_length + decay_length + sustain_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Line::interpolate(
                        (attack_length + decay_length + sustain_length, sustain_level),
                        (
                            attack_length + decay_length + sustain_length + release_length,
                            0.0,
                        ),
                    ),
                    attack_length + decay_length + sustain_length,
                    attack_length + decay_length + sustain_length + release_length,
                ),
            ),
        ])
    }

    pub fn unison<F>(pitch: f64, count: usize, creator: F) -> DynSampler
    where
        F: Fn(f64) -> DynSampler,
    {
        if count % 2 == 0 {
            panic!("Not supported!");
        }
        let pows = -(count as isize / 2)..(count as isize / 2 + 1);
        Compound::new(
            pows.into_iter()
                .map(|p| pitch * 2f64.powf(p as f64))
                .map(|f| (1.0, creator(f)))
                .collect(),
        )
    }
    pub fn play(events: Vec<(f64, DynSampler)>) -> DynSampler {
        Compound::new(
            events
                .into_iter()
                .map(|(d, s)| -> (f64, DynSampler) { (1.0, Shift::new(s, -d)) })
                .collect(),
        )
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
    fn integral(&self) -> DynSampler {
        Compound::new(
            self.samplers
                .iter()
                .map(|(c, s)| (*c, s.integral()))
                .collect(),
        )
    }
}

#[derive(Clone)]
pub struct Shift {
    pub shift: f64,
    pub sampler: DynSampler,
}

impl Shift {
    pub fn new(sampler: DynSampler, shift: f64) -> DynSampler {
        Box::new(Shift { sampler, shift })
    }
}

impl Sampler for Shift {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t + self.shift)
    }
}

#[derive(Clone)]
pub struct Gain {
    pub gain: f64,
    pub sampler: DynSampler,
}

impl Gain {
    pub fn new(sampler: DynSampler, gain: f64) -> DynSampler {
        Box::new(Gain { sampler, gain })
    }
}

impl Sampler for Gain {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.gain
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
