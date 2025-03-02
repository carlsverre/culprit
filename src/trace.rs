use alloc::borrow::Cow;
use alloc::string::ToString;
use alloc::vec::Vec;
use core::error::Error;
use core::fmt::{Display, Formatter};
use smallvec::SmallVec;

use crate::src_location::SrcLocation;

#[derive(Clone)]
pub struct TracePoint {
    location: Option<SrcLocation>,
    note: Cow<'static, str>,
}

impl TracePoint {
    #[track_caller]
    pub fn new<N: Into<Cow<'static, str>>>(note: N) -> Self {
        Self {
            location: Some(SrcLocation::new()),
            note: note.into(),
        }
    }

    pub(crate) fn from_err_source<E: Error>(e: E) -> Self {
        Self {
            location: None,
            note: e.to_string().into(),
        }
    }
}

impl Display for TracePoint {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let note = &self.note;
        match &self.location {
            None => write!(f, "{note}")?,
            Some(loc) => write!(f, "{note}, at {loc}")?,
        }
        Ok(())
    }
}

#[derive(Default, Clone)]
pub struct Trace(SmallVec<[TracePoint; 1]>);

impl Trace {
    pub fn from_ctx(ctx: TracePoint) -> Self {
        Self(SmallVec::from_buf([ctx]))
    }

    #[track_caller]
    pub fn from_err<E: Error>(err: &E) -> Self {
        let ctx = TracePoint::new(err.to_string());
        if err.source().is_none() {
            // fast path if there is no source
            return Self(SmallVec::from_buf([ctx]));
        }

        let mut stack = Vec::new();
        stack.push(ctx);

        // add all error sources to the trace
        let mut source = err.source();
        while let Some(err) = source {
            stack.push(TracePoint::from_err_source(err));
            source = err.source();
        }

        // reverse the stack so the the error is at the top
        stack.reverse();

        Self(SmallVec::from_vec(stack))
    }

    #[inline]
    pub fn push(&mut self, ctx: TracePoint) {
        self.0.push(ctx);
    }

    /// iterate over the trace from the top of the stack to the bottom
    #[inline]
    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &TracePoint> {
        self.0.iter().rev()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for Trace {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        for (i, ctx) in self.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{i}: {ctx}")?;
        }
        Ok(())
    }
}
