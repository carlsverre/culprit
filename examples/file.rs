//! A simple example program which parses a file as a JSON array and then sums
//! the values. All errors are handled by Culprit.
//!
//! This program is overengineered on purpose to demonstrate multiple
//! abstraction layers and showcase more of Culprit's functionality.
//!
//! Here are some example invocations:
//!
//! ```sh
//! # missing an argument
//! cargo run --example file
//!
//! # missing file
//! cargo run --example file missing.json
//!
//! # bad JSON array contents
//! cargo run --example file examples/json/bad.json
//!
//! # good JSON array contents
//! cargo run --example file examples/json/good.json
//! ```

use culprit::{Culprit, ResultExt};

mod file {
    use culprit::{Culprit, ResultExt};
    use std::{
        fmt::{Display, Formatter},
        path::Path,
    };

    #[derive(Debug)]
    pub enum FileCtx {
        Io(std::io::ErrorKind),
        Corrupt,
    }

    impl Display for FileCtx {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                FileCtx::Io(kind) => write!(f, "I/O error: {}", kind),
                FileCtx::Corrupt => write!(f, "corrupt data"),
            }
        }
    }

    impl From<std::io::Error> for FileCtx {
        fn from(err: std::io::Error) -> Self {
            FileCtx::Io(err.kind())
        }
    }

    pub fn read_file<P: AsRef<Path>>(path: P) -> Result<Vec<serde_json::Value>, Culprit<FileCtx>> {
        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let data = serde_json::from_reader(reader).or_ctx(|_| FileCtx::Corrupt)?;
        Ok(data)
    }
}

mod calc {
    use crate::file;
    use culprit::{Culprit, ResultExt};
    use std::path::Path;

    #[derive(Debug)]
    pub enum CalcCtx {
        NotANumber,
        File(file::FileCtx),
    }

    impl std::fmt::Display for CalcCtx {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                CalcCtx::NotANumber => write!(f, "not a number"),
                CalcCtx::File(fp) => write!(f, "{}", fp),
            }
        }
    }

    impl From<file::FileCtx> for CalcCtx {
        fn from(fp: file::FileCtx) -> Self {
            CalcCtx::File(fp)
        }
    }

    pub fn sum_file<P: AsRef<Path>>(path: P) -> Result<f64, Culprit<CalcCtx>> {
        let data = file::read_file(&path).or_into_ctx()?;
        let mut sum = 0f64;
        for value in data {
            match value {
                serde_json::Value::Number(n) => {
                    sum += n
                        .as_f64()
                        .ok_or_else(|| Culprit::new(CalcCtx::NotANumber))?
                }
                other => {
                    return Err(Culprit::new_with_note(
                        CalcCtx::NotANumber,
                        format!("expected number; got {:?}", other),
                    ))
                }
            }
        }
        Ok(sum)
    }
}

#[derive(Debug)]
pub enum Ctx {
    Calc(calc::CalcCtx),
    Usage,
}

impl std::fmt::Display for Ctx {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ctx::Calc(fp) => write!(f, "{}", fp),
            Ctx::Usage => write!(f, "usage error; expected one argument"),
        }
    }
}

impl From<calc::CalcCtx> for Ctx {
    fn from(fp: calc::CalcCtx) -> Self {
        Ctx::Calc(fp)
    }
}

pub fn main() -> Result<(), Culprit<Ctx>> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| Culprit::new(Ctx::Usage))?;
    let sum = calc::sum_file(&path).or_into_ctx()?;
    println!("Sum: {}", sum);
    Ok(())
}
