use super::*;

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
