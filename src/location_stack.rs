use core::{error::Error, fmt::Write};

use crate::static_location::StaticLocation;

pub trait LocationStack: Error {
    fn location(&self) -> &StaticLocation;
    fn next(&self) -> Option<&dyn LocationStack>;
}

impl<T: LocationStack> LocationStack for &T {
    fn location(&self) -> &StaticLocation {
        (*self).location()
    }

    fn next(&self) -> Option<&dyn LocationStack> {
        (*self).next()
    }
}

/// Walks the location stack, calling the callback for each value. Returns the
/// deepest value in the stack.
fn walk<T: LocationStack + ?Sized, E>(
    stack: &T,
    mut cb: impl FnMut(&dyn LocationStack) -> Result<(), E>,
) -> Result<Option<&dyn LocationStack>, E> {
    let Some(mut cursor) = stack.next() else {
        return Ok(None);
    };
    cb(cursor)?;
    while let Some(next) = cursor.next() {
        cursor = next;
        cb(cursor)?;
    }
    Ok(Some(cursor))
}

pub fn format_location_stack<W: Write, T: LocationStack + ?Sized>(
    f: &mut W,
    stack: &T,
    summary: bool,
) -> core::fmt::Result {
    write!(f, "0: {stack} at {}", stack.location())?;
    let mut height = 1;

    let last = walk(stack, |err| {
        if !summary {
            write!(f, "\n{height}: {err} at {}", err.location())?;
            height += 1;
        }
        Ok(())
    })?;

    if let Some(last) = last {
        if summary {
            write!(f, "\n{height}: {last} at {}", last.location())?;
            height += 1;
        }
        if let Some(source) = last.source() {
            write!(f, "\n{height}: {source}")?;
        }
    } else if let Some(source) = stack.source() {
        write!(f, "\n{height}: {source}")?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use core::{error::Error, fmt::Display};

    use super::*;

    #[derive(Debug)]
    struct TrivialError;

    impl Error for TrivialError {}

    impl Display for TrivialError {
        fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "TrivialError")
        }
    }

    #[derive(Debug)]
    struct Node {
        height: usize,
        loc: StaticLocation,
        next: Option<Box<Node>>,
    }

    impl Error for Node {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            Some(&TrivialError)
        }
    }

    impl Display for Node {
        fn fmt(&self, f: &mut std::fmt::Formatter) -> core::fmt::Result {
            write!(f, "Node({})", self.height)
        }
    }

    impl LocationStack for Node {
        fn location(&self) -> &StaticLocation {
            &self.loc
        }

        fn next(&self) -> Option<&dyn LocationStack> {
            if let Some(next) = &self.next {
                Some(next.as_ref())
            } else {
                None
            }
        }
    }

    #[test]
    fn test_format_location_stack() {
        let loc = StaticLocation::default();
        let node = Node {
            height: 0,
            loc,
            next: Some(Box::new(Node {
                height: 1,
                loc,
                next: Some(Box::new(Node {
                    height: 2,
                    loc,
                    next: None,
                })),
            })),
        };

        let mut output = String::new();
        format_location_stack(&mut output, &node, false).unwrap();
        assert_eq!(
            output,
            format!(
                "0: Node(0) at {loc}\n\
                1: Node(1) at {loc}\n\
                2: Node(2) at {loc}\n\
                3: TrivialError",
            )
        );
    }

    #[test]
    fn test_summarize_location_stack() {
        let loc = StaticLocation::default();
        let node = Node {
            height: 0,
            loc,
            next: Some(Box::new(Node {
                height: 1,
                loc,
                next: Some(Box::new(Node {
                    height: 2,
                    loc,
                    next: None,
                })),
            })),
        };

        let mut output = String::new();
        format_location_stack(&mut output, &node, true).unwrap();
        assert_eq!(
            output,
            format!(
                "0: Node(0) at {loc}\n\
                1: Node(2) at {loc}\n\
                2: TrivialError",
            )
        );
    }
}
