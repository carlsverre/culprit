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
enum Context {
    A,
}

impl Display for Context {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Context::A")
    }
}

impl From<SimpleError> for Context {
    fn from(_: SimpleError) -> Self {
        Context::A
    }
}

#[derive(Debug)]
enum Context2 {
    Wrapped(Context),
}

impl Display for Context2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Context2::Wrapped(x) => write!(f, "Context2::Wrapped({x})"),
        }
    }
}

impl From<Context> for Context2 {
    fn from(_: Context) -> Self {
        Context2::Wrapped(Context::A)
    }
}

fn raise_simple_error() -> Result<(), SimpleError> {
    Err(SimpleError)
}

fn wrap_in_culprit() -> Result<(), Culprit<Context>> {
    raise_simple_error()?;
    Ok(())
}

fn map_context() -> Result<(), Culprit<Context2>> {
    wrap_in_culprit().or_into_ctx()?;
    Ok(())
}

fn add_note() -> Result<(), Culprit<Context2>> {
    map_context().or_into_culprit("This is a note")?;
    Ok(())
}

fn map_context_without_changing() -> Result<(), Culprit<Context2>> {
    add_note().or_into_ctx()?;
    Ok(())
}

pub fn main() {
    let culprit = map_context_without_changing().unwrap_err();
    println!("{:?}", culprit);

    assert!(matches!(culprit.ctx(), &Context2::Wrapped(Context::A)));
    let context = culprit.trace();
    assert_eq!(context.len(), 4);
}
