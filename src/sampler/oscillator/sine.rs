use super::*;

#[derive(Clone)]
pub struct Sine {
    freq: f64,
    phase: f64,
}

impl Sine {
    pub fn new(freq: f64, phase: f64) -> DynSampler {
        Box::new(Self { freq, phase })
    }
    pub fn sin(freq: f64) -> DynSampler {
        Self::new(freq, 0f64)
    }
    pub fn cos(freq: f64) -> DynSampler {
        Self::new(freq, std::f64::consts::PI / 2.0)
    }
}

impl Sampler for Sine {
    fn sample(&self, t: f64) -> f64 {
        (t * (2.0 * std::f64::consts::PI * self.freq + self.phase)).sin()
    }
    fn integral(&self) -> DynSampler {
        Gain::new(
            Sine::cos(self.freq),
            -1.0 / (self.freq * 2.0 * std::f64::consts::PI),
        )
    }
}
