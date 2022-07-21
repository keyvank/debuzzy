use super::*;

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
