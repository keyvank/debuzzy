use super::*;
use crate::fft::*;
use crate::filter::Filter;
use rayon::prelude::*;
use rustfft::num_complex::Complex;

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
    pub fn apply_filter<F: Filter>(&mut self, mut filter: F) {
        let mut output = Vec::new();
        for sample in self.samples.iter() {
            output.push(filter.apply(*sample));
        }
        self.samples = output;
    }
    pub fn convolve(&mut self, filter_fft: &Vec<Complex<f64>>) {
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
