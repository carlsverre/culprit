use core::{
    fmt::{Debug, Display, Formatter, Result},
    panic::Location,
};

#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StaticLocation(&'static Location<'static>);

impl StaticLocation {
    #[inline]
    #[track_caller]
    pub fn new() -> Self {
        Self(Location::caller())
    }

    #[inline]
    pub fn file(&self) -> &'static str {
        self.0.file()
    }

    #[inline]
    pub fn line(&self) -> u32 {
        self.0.line()
    }

    #[inline]
    pub fn column(&self) -> u32 {
        self.0.column()
    }
}

impl Default for StaticLocation {
    #[inline]
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for StaticLocation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("StaticLocation")
            .field("file", &self.0.file())
            .field("line", &self.0.line())
            .field("column", &self.0.column())
            .finish()
    }
}

impl Display for StaticLocation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}
