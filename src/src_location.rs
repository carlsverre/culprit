use core::{
    fmt::{Debug, Display, Formatter, Result},
    panic::Location,
};

#[derive(Copy, Clone, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SrcLocation(&'static Location<'static>);

impl SrcLocation {
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

impl From<SrcLocation> for &'static Location<'static> {
    #[inline]
    fn from(val: SrcLocation) -> Self {
        val.0
    }
}

impl Default for SrcLocation {
    #[inline]
    #[track_caller]
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for SrcLocation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("StaticLocation")
            .field("file", &self.0.file())
            .field("line", &self.0.line())
            .field("column", &self.0.column())
            .finish()
    }
}

impl Display for SrcLocation {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        Display::fmt(&self.0, f)
    }
}
