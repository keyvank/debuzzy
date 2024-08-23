use debuzzy::filter::*;
use debuzzy::instrument::*;
use debuzzy::mml;
use debuzzy::sampler::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io::Write;

const SAMPLE_RATE: f64 = 44100.0;

trait Player {
    fn play(sampler: DynSampler, sample_rate: f64, duration: f64);
}

struct StdoutPlayer;

impl Player for StdoutPlayer {
    fn play(sampler: DynSampler, sample_rate: f64, duration: f64) {
        let sample_rate_step = 1f64 / sample_rate;
        let mut t = 0f64;
        while t < duration {
            // Sample one second
            let samples = (0..SAMPLE_RATE as usize)
                .into_par_iter()
                .map(|i| (sampler.sample(t + sample_rate_step * (i as f64)) * 32767.0) as i16)
                .collect::<Vec<_>>();
            for val in samples {
                std::io::stdout().write(&val.to_le_bytes()).unwrap();
            }
            t += sample_rate_step * (SAMPLE_RATE as f64);
        }
    }
}

fn main() {
    StdoutPlayer::play(
        mml::play::<LegitInstrument>(mml::SMOKE_ON_THE_WATER),
        SAMPLE_RATE,
        100.0,
    );
}
