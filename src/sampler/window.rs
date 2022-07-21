use super::*;

#[derive(Clone)]
pub struct Window {
    pub sampler: DynSampler,
    pub low: f64,
    pub high: f64,
}

impl Window {
    pub fn new(sampler: DynSampler, low: f64, high: f64) -> DynSampler {
        Box::new(Window { sampler, low, high })
    }
}

impl Sampler for Window {
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
