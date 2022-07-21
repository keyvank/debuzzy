use super::*;

#[derive(Clone)]
pub struct Impulse;
impl Sampler for Impulse {
    fn sample(&self, t: f64) -> f64 {
        if t == 0.0 {
            1.0
        } else {
            0.0
        }
    }
}
