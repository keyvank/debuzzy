use super::*;

#[derive(Clone)]
pub struct Const {
    pub val: f64,
}

impl Const {
    pub fn new(val: f64) -> DynSampler {
        Box::new(Const { val })
    }
}

impl Sampler for Const {
    fn sample(&self, _t: f64) -> f64 {
        self.val
    }
    fn integral(&self) -> DynSampler {
        Line::new(0.0, self.val)
    }
}
