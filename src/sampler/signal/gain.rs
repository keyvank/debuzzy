use super::*;

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
