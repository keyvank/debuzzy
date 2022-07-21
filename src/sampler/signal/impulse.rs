use super::*;

#[derive(Clone)]
pub struct Impulse {
    half_width: f64,
    height: f64,
}

impl Impulse {
    pub fn new(sample_rate: f64) -> DynSampler {
        Box::new(Self {
            half_width: 0.5 / sample_rate,
            height: sample_rate,
        })
    }
}

impl Sampler for Impulse {
    fn sample(&self, t: f64) -> f64 {
        if t < self.half_width && t > -self.half_width {
            self.height
        } else {
            0.0
        }
    }
    fn integral(&self) -> DynSampler {
        Const::new(1.0)
    }
}
