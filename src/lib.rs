#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]

extern crate alloc;

mod context;
mod culprit;
mod result;
mod src_location;
mod trace;

pub use context::Context;
pub use culprit::{Culprit, CulpritErr};
pub use result::ResultExt;
pub use trace::TracePoint;

pub type Result<T, C> = core::result::Result<T, Culprit<C>>;
