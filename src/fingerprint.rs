use core::fmt::Debug;
use core::fmt::Display;

pub trait Fingerprint: Display + Debug + Send + Sync + 'static {}

impl<T: Display + Debug + Send + Sync + 'static> Fingerprint for T {}
