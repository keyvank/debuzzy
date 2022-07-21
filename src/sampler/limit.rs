use super::*;

#[derive(Clone)]
pub struct Limit {
    pub sampler: DynSampler,
    pub start: f64,
    pub end: f64,
}

impl Limit {
    pub fn new(sampler: DynSampler, start: f64, end: f64) -> DynSampler {
        Box::new(Limit {
            sampler,
            start,
            end,
        })
    }
}

impl Sampler for Limit {
    fn sample(&self, t: f64) -> f64 {
        if t >= self.start && t <= self.end {
            self.sampler.sample(t)
        } else {
            0.0
        }
    }
    fn integral(&self) -> DynSampler {
        let int = self.sampler.integral();
        let s0 = int.sample(self.start);
        Limit::new(
            Compound::new(vec![(1.0, int), (-1.0, Const::new(s0))]),
            self.start,
            self.end,
        )
    }
}
