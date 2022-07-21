mod fft;
mod filter;
mod instrument;
mod mml;
mod notes;
mod sampler;

use instrument::*;
use notes::*;
use sampler::*;
use std::io::Write;

fn out(sample: f64) -> Result<(), std::io::Error> {
    let val = (sample * 32767.0) as i16;
    std::io::stdout().write(&val.to_le_bytes())?;
    Ok(())
}

fn main() -> Result<(), std::io::Error> {
    const SAMPLE_RATE: usize = 44100;

    let music = mml::play::<DummyInstrument>(mml::AIR_ON_G_STRING);
    let mut music = Record::record(music, SAMPLE_RATE as f64, 20.0);
    //music.apply_filter(&CONCERT_HALL_FILTER_FFTS);

    const SAMPLE_RATE_STEP: f64 = 1f64 / (SAMPLE_RATE as f64);
    let mut t = 0f64;
    loop {
        out(music.sample(t))?;
        t += SAMPLE_RATE_STEP;
    }
}
