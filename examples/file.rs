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
    pub enum Fingerprint {
        Io(std::io::ErrorKind),
        Corrupt,
    }

    impl Display for Fingerprint {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            match self {
                Fingerprint::Io(kind) => write!(f, "I/O error: {}", kind),
                Fingerprint::Corrupt => write!(f, "corrupt data"),
            }
        }
    }

    impl From<std::io::Error> for Fingerprint {
        fn from(err: std::io::Error) -> Self {
            Fingerprint::Io(err.kind())
        }
    }

    pub fn read_file<P: AsRef<Path>>(
        path: P,
    ) -> Result<Vec<serde_json::Value>, Culprit<Fingerprint>> {
        let file = std::fs::File::open(&path)?;
        let reader = std::io::BufReader::new(file);
        let data = serde_json::from_reader(reader).fingerprint_with(|_| Fingerprint::Corrupt)?;
        Ok(data)
    }
}

mod calc {
    use crate::file;
    use culprit::{Culprit, ResultExt};
    use std::path::Path;

    #[derive(Debug)]
    pub enum Fingerprint {
        NotANumber,
        File(file::Fingerprint),
    }

    impl std::fmt::Display for Fingerprint {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                Fingerprint::NotANumber => write!(f, "not a number"),
                Fingerprint::File(fp) => write!(f, "{}", fp),
            }
        }
    }

    impl From<file::Fingerprint> for Fingerprint {
        fn from(fp: file::Fingerprint) -> Self {
            Fingerprint::File(fp)
        }
    }

    pub fn sum_file<P: AsRef<Path>>(path: P) -> Result<f64, Culprit<Fingerprint>> {
        let data = file::read_file(&path).fingerprint()?;
        let mut sum = 0f64;
        for value in data {
            match value {
                serde_json::Value::Number(n) => {
                    sum += n
                        .as_f64()
                        .ok_or_else(|| Culprit::new(Fingerprint::NotANumber))?
                }
                other => {
                    return Err(Culprit::new_with_note(
                        Fingerprint::NotANumber,
                        format!("expected number; got {:?}", other),
                    ))
                }
            }
        }
        Ok(sum)
    }
}

#[derive(Debug)]
pub enum Fingerprint {
    Calc(calc::Fingerprint),
    Usage,
}

impl std::fmt::Display for Fingerprint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Fingerprint::Calc(fp) => write!(f, "{}", fp),
            Fingerprint::Usage => write!(f, "usage error; expected one argument"),
        }
    }
}

impl From<calc::Fingerprint> for Fingerprint {
    fn from(fp: calc::Fingerprint) -> Self {
        Fingerprint::Calc(fp)
    }
}

pub fn main() -> Result<(), Culprit<Fingerprint>> {
    let path = std::env::args()
        .nth(1)
        .ok_or_else(|| Culprit::new(Fingerprint::Usage))?;
    let sum = calc::sum_file(&path).fingerprint()?;
    println!("Sum: {}", sum);
    Ok(())
}
