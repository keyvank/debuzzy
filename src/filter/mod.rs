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
