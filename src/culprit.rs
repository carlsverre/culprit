use alloc::borrow::Cow;
use alloc::string::ToString;
use core::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use crate::{
    context::{Context, ContextStack},
    fingerprint::Fingerprint,
};

pub struct Culprit<F: Fingerprint> {
    fingerprint: F,
    stack: ContextStack,
}

impl<F: Fingerprint> Culprit<F> {
    #[inline]
    #[track_caller]
    pub fn new(fingerprint: F) -> Self {
        let stack = ContextStack::from_ctx(Context::new(fingerprint.to_string()));
        Self { fingerprint, stack }
    }

    #[inline]
    #[track_caller]
    pub fn new_with_note<N: Into<Cow<'static, str>>>(fingerprint: F, note: N) -> Self {
        let stack = ContextStack::from_ctx(Context::new(note));
        Self { fingerprint, stack }
    }

    #[inline]
    pub fn new_with_stack(fingerprint: F, stack: ContextStack) -> Self {
        Self { fingerprint, stack }
    }

    #[inline]
    #[track_caller]
    pub fn from_err<E: Error + Into<F>>(err: E) -> Self {
        let stack = ContextStack::from_err(&err);
        let fingerprint = err.into();
        Self { fingerprint, stack }
    }

    #[inline]
    #[track_caller]
    pub fn with_note<I: Into<Cow<'static, str>>>(mut self, note: I) -> Self {
        self.stack.push(Context::new(note));
        self
    }

    #[inline]
    #[track_caller]
    pub fn with_fingerprint<F2, B>(self, fingerprinter: B) -> Culprit<F2>
    where
        F2: Fingerprint,
        B: FnOnce(F) -> F2,
    {
        Culprit {
            fingerprint: fingerprinter(self.fingerprint),
            stack: self.stack,
        }
    }

    #[inline]
    pub fn fingerprint(&self) -> &F {
        &self.fingerprint
    }

    #[inline]
    pub fn context(&self) -> &ContextStack {
        &self.stack
    }

    #[inline]
    pub fn into_err(self) -> CulpritErr<F> {
        CulpritErr(self)
    }
}

impl<E: Error, F: Fingerprint + From<E>> From<E> for Culprit<F> {
    #[inline]
    #[track_caller]
    fn from(source: E) -> Self {
        Self::from_err(source)
    }
}

impl<F: Fingerprint> From<Culprit<F>> for (F, ContextStack) {
    #[inline]
    fn from(culprit: Culprit<F>) -> Self {
        (culprit.fingerprint, culprit.stack)
    }
}

impl<F: Fingerprint> Debug for Culprit<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}\n{}", self.fingerprint, self.stack)?;
        Ok(())
    }
}

impl<F: Fingerprint> Display for Culprit<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}\n{}", self.fingerprint, self.stack)?;
        Ok(())
    }
}

pub struct CulpritErr<F: Fingerprint>(Culprit<F>);

impl<F: Fingerprint> CulpritErr<F> {
    #[inline]
    pub fn into_culprit(self) -> Culprit<F> {
        self.0
    }
}

impl<F: Fingerprint> Display for CulpritErr<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<F: Fingerprint> Debug for CulpritErr<F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<F: Fingerprint> Error for CulpritErr<F> {}
