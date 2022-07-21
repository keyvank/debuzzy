use super::*;

#[derive(Clone)]
pub struct Response {
    input: DynSampler,
    system: DynSampler,
}

impl Response {
    pub fn new(input: DynSampler, system: DynSampler) -> DynSampler {
        Box::new(Response { system, input })
    }
}

impl Sampler for Response {
    fn sample(&self, t: f64) -> f64 {
        self.system.sample(self.input.sample(t))
    }
}
