use crate::fft::fft;
use rustfft::num_complex::Complex;

lazy_static::lazy_static! {
     static ref CONCERT_HALL_FILTER_FFT: Vec<Complex<f64>> ={
         let v:Vec<f64> = std::fs::read("hall.raw")
         .unwrap()
         .chunks(2)
         .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
         .map(|i| (i as f64) / 32767.0)
         .collect();
         fft(&v)
     };
}
