use super::*;
#[derive(Clone)]
pub struct Sine {
    pub freq: f64,
}

impl Sine {
    pub fn new(freq: f64) -> DynSampler {
        Box::new(Self { freq })
    }
}

impl Sampler for Sine {
    fn sample(&self, t: f64) -> f64 {
        (t * 2.0 * std::f64::consts::PI * self.freq).sin()
    }
    fn integral(&self) -> DynSampler {
        Gain::new(
            Box::new(self.clone()),
            1.0 / (self.freq * 2.0 * std::f64::consts::PI),
        )
    }
}
