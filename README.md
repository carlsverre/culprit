<h1 align="center">Culprit</h1>
<p align="center">
  <a href="https://docs.rs/culprit">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/culprit">
  </a>
  <a href="https://crates.io/crates/culprit">
    <img alt="crates.io" src="https://img.shields.io/crates/v/culprit.svg">
  </a>
  <a href="https://github.com/carlsverre/culprit/actions">
    <img alt="Build Status" src="https://github.com/carlsverre/culprit/actions/workflows/rust.yml/badge.svg">
  </a>
</p>

A Rust error crate with the goal of identifying precisely where and in which context an error occurs.

**Goals:**
1. Context both in the logical control flow as well as physical space in files
2. Unique public facing errors
3. Minimal error sets per function/module
4. Aligning errors to error codes for external handling (i.e. outside of rust)

> [!WARNING]  
> Culprit is extremely-alpha and may dramatically change at any point. It's currently undergoing design and testing within some projects. Ideas and discussion is welcome during this process.

**Table of Contents**:
- [Getting started](#getting-started)
- [Concept and comparison to other crates](#concept-and-comparison-to-other-crates)
- [Outstanding Work](#outstanding-work)

# Getting started

**First**, define some fingerprint types which must implement Debug and Display. A Fingerprint represents the unique error cases that can occur in your app. Fingerprints may wrap fingerprints from other abstraction layers such as modules or crates. Fingerprints should be small and represent a unique "fingerprint" of a given error state.

```rust
#[derive(Debug)]
enum StorageFingerprint {
    Io(std::io::ErrorKind),
    Corrupt,
}
impl Display for StorageFingerprint { ... }

#[derive(Debug)]
enum CalcFingerprint {
    NotANumber,
    Storage(StorageFingerprint),
}
impl Display for CalcFingerprint { ... }
```

**Next**, Implement `From` conversions to build `Fingerprints` from `Error`s and other `Fingerprint`s:

```rust
impl From<std::io::Error> for StorageFingerprint {
    fn from(e: std::io::Error) -> Self {
        StorageFingerprint::Io(e.kind())
    }
}

impl From<StorageFingerprint> for CalcFingerprint {
    fn from(f: StorageFingerprint) -> Self {
        CalcFingerprint::Storage(f)
    }
}
```

**Next**, use `Culprit<Fingerprint>` as the error type in a Result:

```rust
fn read_file(file: &str) -> Result<String, Culprit<StorageFingerprint>> {
   Ok(std::fs::read_to_string(file)?)
}

fn sum_file(file: &str) -> Result<f64, Culprit<CalcFingerprint>> {
   // calling fingerprint is required when the fingerprint type changes
   let data = read_file(&req.file).fingerprint()?;
   ...
}
```

**Finally**, check out your fancy new Culprit error message when you debug or display a Culprit:

```
Error: Calc(NotANumber)
0: not a number, at examples/file.rs:136:37
1: expected number; got String("hello"), at examples/file.rs:100:32
```

> [!NOTE]  
> For the full example please visit [file.rs](./examples/file.rs)

# Concept and comparison to other crates

Culprit came around while I was exploring various error handling patterns and crates in Rust. I roughly categorize them in the following way:

1. New Type: A new type per abstraction boundary (usually an enum) often wraps the source error from lower layers directly, providing additional context through the type's `Display/Debug impl`. Examples: vanilla error handling, [thiserror]

2. Accumulator: A single type that consumes an error and then allows additional context to be attached to it as it flows up the stack. Example: [anyhow]

3. Hybrid: A single type that is generic over a new type per abstraction boundary (usually and enum). Provides context via the new type and may support attaching additional context. Example: [culprit]

Each of these patterns maps to a distinct way of structuring errors in a Rust program as well as how a **logical trace** may be captured.

<a id='logical-trace'></a>
> [!TIP]  
> I define **logical trace** as the path the error takes through a program. The path is made up of steps, which primarily correlate with an error passing between modules, crates, threads, or async tasks. The developer may add context to points on the path to further illuminate the error's path.

Here is a table comparing common Rust error crates to Culprit. Please file an issue if I made a mistake or am missing a commonly used crate.

| crate           | pattern     | uses unsafe | logical trace         | captured context                                               |
| --------------- | ----------- | ----------- | --------------------- | -------------------------------------------------------------- |
| [anyhow]        | Accumulator | yes         | context               | strings, Backtrace, custom types¹, source error¹               |
| [error-stack]   | Hybrid      | yes         | context, [SpanTrace]³ | strings, Backtrace, [SpanTrace]³, custom types¹, source error¹ |
| [eyre]          | Accumulator | yes         | context, [SpanTrace]³ | strings, Backtrace, [SpanTrace]³, custom types¹, source error¹ |
| [thiserror]     | New Type    | no          | context               | Backtrace², custom types, source error                         |
| [tracing_error] | Hybrid      | yes         | [SpanTrace]           | only [SpanTrace]                                               |
| [culprit]       | Hybrid      | no          | context, enriched     | strings, custom types, source error                            |

¹ Runtime retrieval requires `TypeId` lookup <br />
² Requires Rust nightly
³ Optional feature


The first thing you may notice is that most of the error handling crates use unsafe. They do this for varying reasons, but the most common use case I found is to support dynamic extraction of nested types at runtime. Examples: [anyhow:downcast_ref] and [error-stack:downcast_ref]. This is a perfectly fine decision, however I believe it comes with some important tradeoffs. The first is crate complexity. The machinery required to store and retrieve values by type involves a lot of very tricky unsafe usage and some clever types to keep the Rust compiler happy. The second tradeoff is that it obfuscates which errors can be raised by which functions by hiding that information from the compiler and thus upstream developers.

Note, when [error_generic_member_access] finally is stabilized, these crates may choose to eliminate some of their unsafe usage by switching to `Error::provide`. However it's not clear if or when this feature will land as the error working group seems to be somewhat abandoned as of January 2025.

Returning to the table, the next interesting feature is how the crate captures the error's <a href="#logical-trace">logical-trace</a>. I've summarized this into three labels: **context**, **enriched**, and **[SpanTrace]**.

* **context**: The logical-trace is captured as additional context is accumulated by the error type.
* **enriched**: The logical-trace is automatically enriched with file names and line numbers as context is accumulated.
* **[SpanTrace]**: The current [tracing]::Span is captured and can be used later to query or print out the tree of Spans that led to the instantiation of an error. See [tracing_error] for more details.

> [!WARNING]  
> This section is still under construction.

# Outstanding Work

- [ ] prefix result extension fns with `or_` to be more idiomatic
- [ ] fingerprint builder fns should return `Into<Fingerprint>`
- [ ] support gradual refinement via making the `Fingerprint` type optional
- [ ] implement `#[derive(Fingerprint)]`
  - [ ] provides `Display` attr and impl
  - [ ] provides `From` impls for deriving Fingerprints from Errors or other Fingerprints. Would be nice to support mapping/extraction.
  - [ ] provides `Into<Culprit>` impl for raising new errors
- [ ] rename `Fingerprint` to `Context` (or something similar)
- [ ] add [SpanTrace] support behind a featureflag
- [ ] add `type Result<T, C> = Result<T, Culprit<C>>`
- [ ] `bail!` or similar macro for easy error generation
- [ ] document all methods and modules

[anyhow]:https://docs.rs/anyhow/latest/anyhow/
[error-stack]: https://docs.rs/error-stack/0.5.0/error_stack/
[eyre]: https://docs.rs/eyre/latest/eyre/
[thiserror]:https://docs.rs/thiserror/latest/thiserror/
[tracing_error]: https://docs.rs/tracing-error/latest/tracing_error/
[culprit]: https://docs.rs/culprit/latest/culprit/
[SpanTrace]: https://docs.rs/tracing-error/latest/tracing_error/struct.SpanTrace.html
[anyhow:downcast_ref]: https://docs.rs/anyhow/1.0.95/anyhow/struct.Error.html#method.downcast_ref
[error-stack:downcast_ref]: https://docs.rs/error-stack/0.5.0/error_stack/struct.Report.html
[error_generic_member_access]: https://github.com/rust-lang/rust/issues/99301
[tracing]: https://docs.rs/tracing/latest/tracing/index.html