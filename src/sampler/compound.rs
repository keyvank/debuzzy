use super::*;

#[derive(Clone)]
pub struct Compound {
    pub samplers: Vec<(f64, DynSampler)>,
}

impl Compound {
    pub fn new(samplers: Vec<(f64, DynSampler)>) -> DynSampler {
        Box::new(Compound { samplers })
    }
    pub fn adsr(
        attack_length: f64,
        decay_length: f64,
        sustain_length: f64,
        release_length: f64,
        sustain_level: f64,
    ) -> DynSampler {
        Compound::new(vec![
            (
                1.0,
                Limit::new(
                    Line::interpolate((0.0, 0.0), (attack_length, 1.0)),
                    0.0,
                    attack_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Line::interpolate(
                        (attack_length, 1.0),
                        (attack_length + decay_length, sustain_level),
                    ),
                    attack_length,
                    attack_length + decay_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Const::new(sustain_level),
                    attack_length + decay_length,
                    attack_length + decay_length + sustain_length,
                ),
            ),
            (
                1.0,
                Limit::new(
                    Line::interpolate(
                        (attack_length + decay_length + sustain_length, sustain_level),
                        (
                            attack_length + decay_length + sustain_length + release_length,
                            0.0,
                        ),
                    ),
                    attack_length + decay_length + sustain_length,
                    attack_length + decay_length + sustain_length + release_length,
                ),
            ),
        ])
    }

    pub fn unison<F>(pitch: f64, count: usize, creator: F) -> DynSampler
    where
        F: Fn(f64) -> DynSampler,
    {
        if count % 2 == 0 {
            panic!("Not supported!");
        }
        let pows = -(count as isize / 2)..(count as isize / 2 + 1);
        Compound::new(
            pows.into_iter()
                .map(|p| pitch * 2f64.powf(p as f64))
                .map(|f| (1.0, creator(f)))
                .collect(),
        )
    }
    pub fn play(events: Vec<(f64, DynSampler)>) -> DynSampler {
        Compound::new(
            events
                .into_iter()
                .map(|(d, s)| -> (f64, DynSampler) { (1.0, Shift::new(s, -d)) })
                .collect(),
        )
    }
}

impl Sampler for Compound {
    fn sample(&self, t: f64) -> f64 {
        let mut s = 0f64;
        for (vol, sampler) in self.samplers.iter() {
            s += sampler.sample(t) * vol;
        }
        s
    }
    fn integral(&self) -> DynSampler {
        Compound::new(
            self.samplers
                .iter()
                .map(|(c, s)| (*c, s.integral()))
                .collect(),
        )
    }
}
