use super::*;

#[derive(Clone)]
pub struct FrequencyModulator {
    frequency_integral: DynSampler,
    sampler: DynSampler,
}

impl FrequencyModulator {
    pub fn new(sampler: DynSampler, frequency: DynSampler) -> DynSampler {
        Box::new(FrequencyModulator {
            sampler,
            frequency_integral: frequency.integral(),
        })
    }
}

impl Sampler for FrequencyModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(self.frequency_integral.sample(t))
    }
}
