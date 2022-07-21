use super::*;

#[derive(Clone)]
pub struct AmplitudeModulator {
    amplitude: DynSampler,
    sampler: DynSampler,
}

impl AmplitudeModulator {
    pub fn new(sampler: DynSampler, amplitude: DynSampler) -> DynSampler {
        Box::new(AmplitudeModulator { sampler, amplitude })
    }
}

impl Sampler for AmplitudeModulator {
    fn sample(&self, t: f64) -> f64 {
        self.sampler.sample(t) * self.amplitude.sample(t)
    }
}
