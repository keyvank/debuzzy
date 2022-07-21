mod ampmod;
mod compound;
mod constant;
mod freqmod;
mod gain;
mod impulse;
mod limit;
mod line;
mod oscillator;
mod quadratic;
mod record;
mod shift;
mod window;

pub use ampmod::*;
pub use compound::*;
pub use constant::*;
pub use freqmod::*;
pub use gain::*;
pub use impulse::*;
pub use limit::*;
pub use line::*;
pub use oscillator::*;
pub use quadratic::*;
pub use record::*;
pub use shift::*;
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
