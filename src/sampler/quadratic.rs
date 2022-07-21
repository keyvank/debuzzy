use super::*;

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
