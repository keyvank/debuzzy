use super::*;

#[derive(Clone)]
pub struct Const {
    a0: f64,
}

impl Const {
    pub fn new(a0: f64) -> DynSampler {
        Box::new(Const { a0 })
    }
}

impl Sampler for Const {
    fn sample(&self, _t: f64) -> f64 {
        self.a0
    }
    fn integral(&self) -> DynSampler {
        Line::new(0.0, self.a0)
    }
}
