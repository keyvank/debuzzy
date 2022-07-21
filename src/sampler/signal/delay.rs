use super::*;

#[derive(Clone)]
pub struct Delay {
    pub delay: f64,
    pub sampler: DynSampler,
}

impl Delay {
    pub fn new(sampler: DynSampler, delay: f64) -> DynSampler {
        Box::new(Self { sampler, delay })
    }
}

impl Sampler for Delay {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t - self.delay)
    }
}
