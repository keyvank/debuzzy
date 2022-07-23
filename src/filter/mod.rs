use crate::fft::fft;
use rustfft::num_complex::Complex;

lazy_static::lazy_static! {
     static ref CONCERT_HALL_FILTER_FFT: Vec<Complex<f64>> ={
         let v:Vec<f64> = std::fs::read("hall.raw")
         .unwrap()
         .chunks(2)
         .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
         .map(|i| (i as f64) / 32767.0)
         .collect();
         fft(&v)
     };
}

pub trait Filter {
    fn apply(&mut self, sample: f64) -> f64;
}

pub struct Integrator {
    value: f64,
}

impl Integrator {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
}

impl Filter for Integrator {
    fn apply(&mut self, sample: f64) -> f64 {
        self.value += sample;
        self.value
    }
}

pub struct Differentiator {
    value: f64,
}

impl Filter for Differentiator {
    fn apply(&mut self, sample: f64) -> f64 {
        let diff = sample - self.value;
        self.value = sample;
        diff
    }
}

impl Differentiator {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
}

pub struct MovingAverage {
    sum: f64,
    first_val: f64,
    count: usize,
    max_count: usize,
}

impl MovingAverage {
    pub fn new(count: usize) -> Self {
        Self {
            sum: 0.0,
            first_val: 0.0,
            count: 0,
            max_count: count,
        }
    }
}

impl Filter for MovingAverage {
    fn apply(&mut self, sample: f64) -> f64 {
        self.sum += sample;
        if self.count == 0 {
            self.first_val = sample;
            self.count += 1;
        } else if self.count < self.max_count {
            self.count += 1;
        } else {
            self.sum -= self.first_val;
            self.first_val = sample;
        }
        self.sum / (self.count as f64)
    }
}
