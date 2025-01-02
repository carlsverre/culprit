use alloc::borrow::Cow;
use alloc::string::ToString;
use core::error::Error;

use crate::{context::ContextStack, culprit::Culprit, fingerprint::Fingerprint};

pub trait ResultExt {
    type Ok;
    type Residual;

    #[track_caller]
    fn fingerprint_with<F, B>(self, fingerprinter: B) -> Result<Self::Ok, Culprit<F>>
    where
        B: FnOnce(Self::Residual) -> F,
        F: Fingerprint;

    #[inline]
    #[track_caller]
    fn fingerprint<F>(self) -> Result<Self::Ok, Culprit<F>>
    where
        Self: Sized,
        F: Fingerprint + From<Self::Residual>,
    {
        self.fingerprint_with(F::from)
    }

    #[inline]
    #[track_caller]
    fn note_with<F, N, B>(self, note: N, fingerprinter: B) -> Result<Self::Ok, Culprit<F>>
    where
        N: Into<Cow<'static, str>>,
        B: FnOnce(Self::Residual) -> F,
        F: Fingerprint,
        Self: Sized,
    {
        self.fingerprint_with(fingerprinter)
            .map_err(|culprit| culprit.with_note(note))
    }

    #[inline]
    #[track_caller]
    fn note<F, N>(self, note: N) -> Result<Self::Ok, Culprit<F>>
    where
        Self: Sized,
        F: Fingerprint + From<Self::Residual>,
        N: Into<Cow<'static, str>>,
    {
        self.note_with(note, F::from)
    }
}

impl<Ok, Err: Error> ResultExt for core::result::Result<Ok, Err> {
    type Ok = Ok;
    type Residual = Err;

    #[track_caller]
    fn fingerprint_with<F, B>(self, fingerprinter: B) -> Result<Ok, Culprit<F>>
    where
        B: FnOnce(Err) -> F,
        F: Fingerprint,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => {
                let stack = ContextStack::from_err(&e);
                let fingerprint = fingerprinter(e);
                Err(Culprit::new_with_stack(fingerprint, stack))
            }
        }
    }
}

impl<Ok, F1: Fingerprint> ResultExt for core::result::Result<Ok, Culprit<F1>> {
    type Ok = Ok;
    type Residual = F1;

    #[track_caller]
    fn fingerprint_with<F2, B>(self, fingerprinter: B) -> Result<Ok, Culprit<F2>>
    where
        B: FnOnce(F1) -> F2,
        F2: Fingerprint,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => {
                let note = culprit.fingerprint().to_string();
                Err(culprit.with_fingerprint(fingerprinter).with_note(note))
            }
        }
    }

    fn note_with<F2, N, B>(self, note: N, fingerprinter: B) -> Result<Ok, Culprit<F2>>
    where
        N: Into<Cow<'static, str>>,
        B: FnOnce(F1) -> F2,
        Self: Sized,
        F2: Fingerprint,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => Err(culprit.with_fingerprint(fingerprinter).with_note(note)),
        }
    }

    fn note<F2, N>(self, note: N) -> Result<Ok, Culprit<F2>>
    where
        Self: Sized,
        F2: Fingerprint + From<F1>,
        N: Into<Cow<'static, str>>,
    {
        match self {
            Ok(t) => Ok(t),
            Err(culprit) => Err(culprit.with_fingerprint(F2::from).with_note(note)),
        }
    }
}
