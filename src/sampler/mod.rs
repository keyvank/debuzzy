mod compound;
mod limit;
mod linear;
mod modulator;
mod oscillator;
mod record;
mod signal;
mod window;

pub use compound::*;
pub use limit::*;
pub use linear::*;
pub use modulator::*;
pub use oscillator::*;
pub use record::*;
pub use signal::*;
pub use window::*;

use dyn_clone::DynClone;

pub trait Sampler: DynClone + Send + Sync {
    fn sample(&self, t: f64) -> f64;
    fn integral(&self) -> Box<dyn Sampler> {
        unimplemented!();
    }
}

pub type DynSampler = Box<dyn Sampler>;

dyn_clone::clone_trait_object!(Sampler);
