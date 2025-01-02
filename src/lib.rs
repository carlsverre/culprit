#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod context;
mod culprit;
mod fingerprint;
mod result;
mod src_location;

pub use context::Context;
pub use culprit::{Culprit, CulpritErr};
pub use fingerprint::Fingerprint;
pub use result::ResultExt;
