use core::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::static_location::StaticLocation;

pub type Result<T, Ctx> = core::result::Result<T, Culprit<Ctx>>;

pub struct Culprit<Ctx> {
    location: StaticLocation,
    context: Ctx,
}

impl<Ctx> Culprit<Ctx> {
    #[inline]
    #[track_caller]
    pub fn new(context: Ctx) -> Self {
        Self {
            location: StaticLocation::new(),
            context,
        }
    }
}

impl<Ctx: CulpritContext> Culprit<Ctx> {
    pub fn sources(&self) -> impl Iterator<Item = &dyn CulpritContext> {
        let mut cursor = Some(&self.context as &dyn CulpritContext);
        core::iter::from_fn(move || {
            let next = cursor.and_then(CulpritContext::source);
            core::mem::replace(&mut cursor, next)
        })
    }
}

impl<Ctx> Display for Culprit<Ctx>
where
    Ctx: CulpritContext,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.context)?;
        for source in self.sources() {
            write!(f, "\nCaused by: {}", source)?;
        }
        Ok(())
    }
}

impl<Ctx> Debug for Culprit<Ctx>
where
    Ctx: CulpritContext,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{} at {}", self.context, self.location)?;
        for source in self.sources() {
            if let Some(loc) = source.location() {
                write!(f, "\nCaused by: {:?} at {}", source, loc)?;
            } else {
                write!(f, "\nCaused by: {:?}", source)?;
            }
        }
        Ok(())
    }
}

pub trait ResultExt {
    type Ok;
    type Err;

    #[track_caller]
    fn with_context<Ctx: CulpritContext>(
        self,
        builder: impl FnOnce(Self::Err) -> Ctx,
    ) -> Result<Self::Ok, Ctx>;
}

impl<Ok, Err: Error> ResultExt for core::result::Result<Ok, Err> {
    type Ok = Ok;
    type Err = Err;

    #[inline]
    #[track_caller]
    fn with_context<Ctx: CulpritContext>(
        self,
        builder: impl FnOnce(Err) -> Ctx,
    ) -> Result<Ok, Ctx> {
        self.map_err(|s| Culprit::new(builder(s)))
    }
}

impl<E: Error, C> From<E> for Culprit<C>
where
    C: CulpritContext + From<E>,
{
    #[inline]
    #[track_caller]
    fn from(source: E) -> Self {
        Culprit::new(C::from(source))
    }
}

pub trait CulpritContext: Display + Debug {
    fn source(&self) -> Option<&dyn CulpritContext> {
        None
    }
    fn location(&self) -> Option<StaticLocation> {
        None
    }
}

impl<Ctx: CulpritContext> CulpritContext for Culprit<Ctx> {
    fn source(&self) -> Option<&dyn CulpritContext> {
        self.context.source()
    }

    fn location(&self) -> Option<StaticLocation> {
        Some(self.location)
    }
}

#[cfg(test)]
mod tests {
    use core::fmt::{Debug, Formatter};

    use super::*;

    #[derive(Debug)]
    struct TrivialError;
    impl Error for TrivialError {}
    impl Display for TrivialError {
        fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
            write!(f, "TrivialError")
        }
    }

    #[derive(Debug)]
    struct TrivialContext {
        source: TrivialError,
    }
    impl Display for TrivialContext {
        fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
            Debug::fmt(self, f)
        }
    }
    impl CulpritContext for TrivialContext {}
    impl From<TrivialError> for TrivialContext {
        fn from(err: TrivialError) -> Self {
            TrivialContext { source: err }
        }
    }

    #[derive(Debug)]
    struct TrivialContext2 {
        source: Culprit<TrivialContext>,
    }
    impl Display for TrivialContext2 {
        fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
            Debug::fmt(self, f)
        }
    }
    impl CulpritContext for TrivialContext2 {}
    impl From<Culprit<TrivialContext>> for TrivialContext2 {
        fn from(err: Culprit<TrivialContext>) -> Self {
            TrivialContext2 { source: err }
        }
    }

    fn wat() -> Result<(), TrivialContext> {
        Err(TrivialError.into())
    }

    fn wat2() -> Result<(), TrivialContext2> {
        wat()?;
        Ok(())
    }

    #[test]
    fn test_culprit_sanity() {
        let err = wat().unwrap_err();
        println!("{:?}\n", err);
        let err = wat2().unwrap_err();
        println!("{:?}", err);
    }
}
