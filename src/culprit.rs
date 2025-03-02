use alloc::borrow::Cow;
use alloc::string::ToString;
use core::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use crate::{
    context::Context,
    trace::{Trace, TracePoint},
};

#[derive(Clone)]
pub struct Culprit<C: Context> {
    ctx: C,
    stack: Trace,
}

impl<C: Context> Culprit<C> {
    #[inline]
    #[track_caller]
    pub fn new(ctx: C) -> Self {
        let stack = Trace::from_ctx(TracePoint::new(ctx.to_string()));
        Self { ctx, stack }
    }

    #[inline]
    #[track_caller]
    pub fn new_with_note<N: Into<Cow<'static, str>>>(ctx: C, note: N) -> Self {
        let stack = Trace::from_ctx(TracePoint::new(note));
        Self { ctx, stack }
    }

    #[inline]
    pub fn new_with_stack(ctx: impl Into<C>, stack: Trace) -> Self {
        Self {
            ctx: ctx.into(),
            stack,
        }
    }

    #[inline]
    #[track_caller]
    pub fn from_err<E: Error + Into<C>>(err: E) -> Self {
        let stack = Trace::from_err(&err);
        let ctx = err.into();
        Self { ctx, stack }
    }

    #[inline]
    #[track_caller]
    pub fn with_note<I: Into<Cow<'static, str>>>(mut self, note: I) -> Self {
        self.stack.push(TracePoint::new(note));
        self
    }

    #[inline]
    #[track_caller]
    pub fn map_ctx<I, C2, F>(self, map: F) -> Culprit<C2>
    where
        C2: Context + From<I>,
        F: FnOnce(C) -> I,
    {
        Culprit {
            ctx: map(self.ctx).into(),
            stack: self.stack,
        }
    }

    #[inline]
    pub fn ctx(&self) -> &C {
        &self.ctx
    }

    #[inline]
    pub fn trace(&self) -> &Trace {
        &self.stack
    }

    #[inline]
    pub fn into_err(self) -> CulpritErr<C> {
        CulpritErr(self)
    }
}

impl<E: Error, C: Context + From<E>> From<E> for Culprit<C> {
    #[inline]
    #[track_caller]
    fn from(source: E) -> Self {
        Self::from_err(source)
    }
}

impl<C: Context> From<Culprit<C>> for (C, Trace) {
    #[inline]
    fn from(culprit: Culprit<C>) -> Self {
        (culprit.ctx, culprit.stack)
    }
}

impl<C: Context> Debug for Culprit<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}\n{}", self.ctx, self.stack)?;
        Ok(())
    }
}

impl<C: Context> Display for Culprit<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}\n{}", self.ctx, self.stack)?;
        Ok(())
    }
}

pub struct CulpritErr<C: Context>(Culprit<C>);

impl<C: Context> CulpritErr<C> {
    #[inline]
    pub fn into_culprit(self) -> Culprit<C> {
        self.0
    }
}

impl<C: Context> Display for CulpritErr<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<C: Context> Debug for CulpritErr<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<C: Context> Error for CulpritErr<C> {}

#[cfg(test)]
mod tests {
    use core::error::Error;
    use core::fmt::Display;

    use super::Culprit;

    #[derive(Debug, Clone)]
    struct Ctx;

    impl Display for Ctx {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Ctx")
        }
    }

    impl Error for Ctx {}

    #[test]
    fn test_clone() {
        let culprit = Culprit::new(Ctx);
        let _ = culprit.clone();
    }
}
