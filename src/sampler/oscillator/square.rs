use super::*;

#[derive(Clone)]
pub struct Square {
    pub freq: f64,
}

impl Square {
    pub fn new(freq: f64) -> DynSampler {
        Box::new(Square { freq })
    }
}

impl Sampler for Square {
    fn sample(&self, t: f64) -> f64 {
        if (t * self.freq).floor() as i64 % 2 == 0 {
            1.0
        } else {
            -1.0
        }
    }
}
