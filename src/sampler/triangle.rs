use super::*;

#[derive(Clone)]
pub struct Triangle {
    pub freq: f64,
}

impl Triangle {
    pub fn new(freq: f64) -> DynSampler {
        Box::new(Self { freq })
    }
}

impl Sampler for Triangle {
    fn sample(&self, t: f64) -> f64 {
        let t = t * self.freq;
        2.0 * (2.0 * (t - (t + 0.5).floor())).abs() - 1.0
    }
}
