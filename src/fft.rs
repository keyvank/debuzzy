use realfft::RealFftPlanner;
use rustfft::{num_complex::Complex, FftPlanner};

pub fn fft(vals: &[f64]) -> Vec<Complex<f64>> {
    let mut real_planner = RealFftPlanner::<f64>::new();

    let r2c = real_planner.plan_fft_forward(vals.len());
    let mut indata = r2c.make_input_vec();

    let mut spectrum = r2c.make_output_vec();

    assert_eq!(indata.len(), vals.len());
    assert_eq!(spectrum.len(), vals.len() / 2 + 1);

    indata.copy_from_slice(vals);
    r2c.process(&mut indata, &mut spectrum).unwrap();
    spectrum
}

pub fn ifft(vals: &Vec<Complex<f64>>) -> Vec<f64> {
    let mut vals = vals.clone();
    let length = (vals.len() - 1) * 2;
    let mut real_planner = RealFftPlanner::<f64>::new();
    let c2r = real_planner.plan_fft_inverse(length);
    let mut outdata = c2r.make_output_vec();
    assert_eq!(outdata.len(), length);

    c2r.process(&mut vals, &mut outdata).unwrap();

    let norm = 1.0 / (length as f64);
    outdata.into_iter().map(|v| v * norm).collect()
}
