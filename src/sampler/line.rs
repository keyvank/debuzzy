use super::*;

#[derive(Clone)]
pub struct Line {
    a0: f64,
    a1: f64,
}

impl Line {
    pub fn new(a0: f64, a1: f64) -> DynSampler {
        Box::new(Self { a0, a1 })
    }
    pub fn interpolate(a: (f64, f64), b: (f64, f64)) -> DynSampler {
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
