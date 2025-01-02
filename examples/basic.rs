use std::{error::Error, fmt::Display};

use culprit::{Culprit, ResultExt};

#[derive(Debug)]
struct SimpleError;
impl Error for SimpleError {}
impl Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SimpleError")
    }
}

#[derive(Debug)]
enum Fingerprint {
    A,
}

impl Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Fingerprint::A")
    }
}

impl From<SimpleError> for Fingerprint {
    fn from(_: SimpleError) -> Self {
        Fingerprint::A
    }
}

#[derive(Debug)]
enum Fingerprint2 {
    Wrapped(Fingerprint),
}

impl Display for Fingerprint2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fingerprint2::Wrapped(x) => write!(f, "Fingerprint2::Wrapped({x})"),
        }
    }
}

impl From<Fingerprint> for Fingerprint2 {
    fn from(_: Fingerprint) -> Self {
        Fingerprint2::Wrapped(Fingerprint::A)
    }
}

fn raise_simple_error() -> Result<(), SimpleError> {
    Err(SimpleError)
}

fn wrap_in_culprit() -> Result<(), Culprit<Fingerprint>> {
    raise_simple_error()?;
    Ok(())
}

fn map_fingerprint() -> Result<(), Culprit<Fingerprint2>> {
    wrap_in_culprit().fingerprint()?;
    Ok(())
}

fn add_note() -> Result<(), Culprit<Fingerprint2>> {
    map_fingerprint().note("This is a note")?;
    Ok(())
}

fn map_fingerprint_without_changing() -> Result<(), Culprit<Fingerprint2>> {
    add_note().fingerprint()?;
    Ok(())
}

pub fn main() {
    let culprit = map_fingerprint_without_changing().unwrap_err();
    println!("{:?}", culprit);

    assert!(matches!(
        culprit.fingerprint(),
        &Fingerprint2::Wrapped(Fingerprint::A)
    ));
    let context = culprit.context();
    assert_eq!(context.len(), 4);
}
