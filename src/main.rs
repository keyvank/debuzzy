use debuzzy::filter::*;
use debuzzy::instrument::*;
use debuzzy::mml;
use debuzzy::sampler::*;
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
            let val = (sampler.sample(t) * 32767.0) as i16;
            std::io::stdout().write(&val.to_le_bytes()).unwrap();
            t += sample_rate_step;
        }
    }
}

fn main() {
    StdoutPlayer::play(mml::play::<LegitInstrument>(mml::MARIO), SAMPLE_RATE, 100.0);
}
