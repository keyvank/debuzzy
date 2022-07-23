use super::*;

#[derive(Clone)]
pub struct Square {
    freq: f64,
    pulse_width: f64,
}

impl Square {
    pub fn new(freq: f64, pulse_width: f64) -> DynSampler {
        Box::new(Square { freq, pulse_width })
    }
}

impl Sampler for Square {
    fn sample(&self, t: f64) -> f64 {
        let t = t * self.freq;
        if (t - t.floor()) < self.pulse_width {
            1.0
        } else {
            -1.0
        }
    }
}
