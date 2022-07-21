use crate::sampler::*;

pub trait Instrument {
    fn play(note: f64, length: f64, volume: f64) -> DynSampler;
}

pub struct Drum;

impl Instrument for Drum {
    fn play(note: f64, length: f64, volume: f64) -> DynSampler {
        let snd = AmplitudeModulator::new(
            Sawtooth::new(note / 32.0),
            Compound::adsr(0.1, 0.1, 0.0, 0.1, 0.1),
        );
        Gain::new(
            FrequencyModulator::new(snd, Compound::adsr(0.05, 1.0, 0.05, 0.05, 0.1).integral()),
            0.2,
        )
    }
}

pub struct DummyInstrument;

impl Instrument for DummyInstrument {
    fn play(note: f64, length: f64, volume: f64) -> DynSampler {
        let snd = AmplitudeModulator::new(
            Sawtooth::new(note),
            Compound::adsr(0.1, length, 0.0, 0.1, 0.1),
        );
        Gain::new(
            FrequencyModulator::new(snd, Window::new(Sine::new(5.0), 1.05, 1.10).integral()),
            0.1,
        )
    }
}

pub struct LegitInstrument;

impl Instrument for LegitInstrument {
    fn play(note: f64, length: f64, volume: f64) -> DynSampler {
        Gain::new(
            AmplitudeModulator::new(
                AmplitudeModulator::new(
                    Compound::new(vec![(0.1, Compound::unison(note, 7, |f| Sine::new(f)))]),
                    Window::new(Sine::new(4.0), 0.3, 1.0),
                ),
                Compound::adsr(0.1, length, 0.0, 0.1, 0.1),
            ),
            volume,
        )
    }
}
