use alloc::borrow::Cow;
use alloc::string::ToString;
use core::error::Error;

use crate::{context::Context, culprit::Culprit, trace::Trace};

pub trait ResultExt {
    type Ok;
    type Residual;

    #[track_caller]
    fn or_ctx<I, C, F>(self, op: F) -> Result<Self::Ok, Culprit<C>>
    where
        F: FnOnce(Self::Residual) -> I,
        C: Context + From<I>;

    #[inline]
    #[track_caller]
    fn or_into_ctx<C>(self) -> Result<Self::Ok, Culprit<C>>
    where
        Self: Sized,
        C: Context + From<Self::Residual>,
    {
        self.or_ctx(C::from)
    }

    #[inline]
    #[track_caller]
    fn or_culprit<I, C, N, F>(self, note: N, op: F) -> Result<Self::Ok, Culprit<C>>
    where
        N: Into<Cow<'static, str>>,
        F: FnOnce(Self::Residual) -> I,
        C: Context + From<I>,
        Self: Sized,
    {
        self.or_ctx(op).map_err(|culprit| culprit.with_note(note))
    }

    #[inline]
    #[track_caller]
    fn or_into_culprit<C, N>(self, note: N) -> Result<Self::Ok, Culprit<C>>
    where
        Self: Sized,
        C: Context + From<Self::Residual>,
        N: Into<Cow<'static, str>>,
    {
        self.or_culprit(note, C::from)
    }
}

impl<Ok, Err: Error> ResultExt for core::result::Result<Ok, Err> {
    type Ok = Ok;
    type Residual = Err;

    #[track_caller]
    fn or_ctx<I, C, F>(self, op: F) -> Result<Ok, Culprit<C>>
    where
        F: FnOnce(Err) -> I,
        C: Context + From<I>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                let stack = Trace::from_err(&e);
                Err(Culprit::new_with_stack(op(e), stack))
            }
        }
    }
}

impl<Ok, C1: Context> ResultExt for core::result::Result<Ok, Culprit<C1>> {
    type Ok = Ok;
    type Residual = C1;

    #[track_caller]
    fn or_ctx<I, C2, F>(self, op: F) -> Result<Ok, Culprit<C2>>
    where
        F: FnOnce(C1) -> I,
        C2: Context + From<I>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => {
                let note = culprit.ctx().to_string();
                Err(culprit.map_ctx(op).with_note(note))
            }
        }
    }

    fn or_culprit<I, C2, N, F>(self, note: N, op: F) -> Result<Ok, Culprit<C2>>
    where
        N: Into<Cow<'static, str>>,
        F: FnOnce(C1) -> I,
        Self: Sized,
        C2: Context + From<I>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => Err(culprit.map_ctx(op).with_note(note)),
        }
    }

    fn or_into_culprit<C2, N>(self, note: N) -> Result<Ok, Culprit<C2>>
    where
        Self: Sized,
        C2: Context + From<C1>,
        N: Into<Cow<'static, str>>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => Err(culprit.map_ctx(C2::from).with_note(note)),
        }
    }
}
