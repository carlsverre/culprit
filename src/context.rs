use core::fmt::Debug;
use core::fmt::Display;

pub trait Context: Display + Debug + Send + Sync + 'static {}

impl<T: Display + Debug + Send + Sync + 'static> Context for T {}
