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

**First**, define some context types which must implement Debug and Display. A Context represents the unique error cases that can occur in your app. Context types may wrap context from other abstraction layers such as modules or crates. Context types should be small and represent the most relevant context of a given error state.

```rust
#[derive(Debug)]
enum StorageCtx {
    Io(std::io::ErrorKind),
    Corrupt,
}
impl Display for StorageCtx { ... }

#[derive(Debug)]
enum CalcCtx {
    NotANumber,
    Storage(StorageCtx
  ),
}
impl Display for CalcCtx { ... }
```

**Next**, Implement `From` conversions to build `Contexts` from `Errors` and other `Contexts`:

```rust
impl From<std::io::Error> for StorageCtx {
    fn from(e: std::io::Error) -> Self {
        StorageCtx::Io(e.kind())
    }
}

impl From<StorageCtx> for CalcCtx {
    fn from(f: StorageCtx) -> Self {
        CalcCtx::Storage(f)
    }
}
```

**Next**, use `Culprit<Ctx>` as the error type in a Result:

```rust
fn read_file(file: &str) -> Result<String, Culprit<StorageCtx>> {
   Ok(std::fs::read_to_string(file)?)
}

fn sum_file(file: &str) -> Result<f64, Culprit<CalcCtx>> {
   // calling `or_ctx` or `or_into_ctx` is required when the result already contains a Culprit but you want to change the context.
   let data = read_file(&req.file).or_into_ctx()?;
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

1. New Type: A new type per abstraction boundary (often an enum) often wraps the source error from lower layers directly, providing additional context via the type's `Display/Debug impl` and associated fields. Example: [thiserror]

2. Accumulator: A single type that consumes an error and then allows additional context to be attached to it as it flows up the stack. Example: [anyhow]

3. Hybrid: An Accumulator type which is generic, allowing it to be specialized with a New Type (often an enum). Provides context via the New Type as well as dynamic attachments. Example: [culprit]

Each of these patterns are able to capture errors as well as the **logical trace** of an erroneous program state.

<a id='logical-trace'></a>
> [!TIP]  
> I define **logical trace** as the path the error takes through a program. The path is made up of steps, which primarily correlate with an error passing between modules, crates, threads, or async tasks. The developer may add context to points on the path to further illuminate the error's path. It's important to note that while they share similar properties, a logical trace is not the same as a backtrace. The former captures the error's flow through the codebase while the latter captures the state of the stack at a particular point in time.

Here is a table comparing common Rust error crates to Culprit. _Please file an issue if I made a mistake or am missing a commonly used crate._

<a id='comparison-table'></a>
| crate           | pattern     | uses unsafe | logical trace                   | captured context                                                |
| --------------- | ----------- | ----------- | ------------------------------- | --------------------------------------------------------------- |
| [anyhow]        | Accumulator | yes         | context                         | strings, Backtrace³, custom types¹, source error¹               |
| [error-stack]   | Hybrid      | yes         | context, enriched, [SpanTrace]³ | strings, Backtrace³, [SpanTrace]³, custom types¹, source error¹ |
| [eyre]          | Accumulator | yes         | context, [SpanTrace]³           | strings, Backtrace³, [SpanTrace]³, custom types¹, source error¹ |
| [thiserror]     | New Type    | no          | context                         | Backtrace²⁺³, custom types, source error                        |
| [tracing_error] | Hybrid      | yes         | [SpanTrace]                     | only [SpanTrace]                                                |
| [culprit]       | Hybrid      | no          | context, enriched               | strings, custom types, source error                             |

¹ Runtime retrieval requires `TypeId` lookup <br />
² Requires Rust nightly <br />
³ Optional feature

The first thing you may notice is that most of the error handling crates use unsafe. They do this for varying reasons, but the most common use case I found is to support dynamic extraction of nested types at runtime. Examples: [anyhow:downcast_ref] and [error-stack:downcast_ref]. This is a perfectly fine decision, however I believe it comes with some important tradeoffs. The first is crate complexity. The machinery required to store and retrieve values by type involves a lot of very tricky unsafe usage and some clever typesystem shenanigans to keep the Rust compiler happy. The second tradeoff is that it obfuscates the set of possible erroneous states by hiding that information from the typesystem.

> [!NOTE]  
> When [error_generic_member_access] finally stabilizes, these crates may choose to eliminate some or all of their unsafe usage by switching to `Error::provide`. However it's not clear if or when this feature will land as the error working group seems to be somewhat abandoned as of January 2025.

Returning to the [table], the next interesting feature is how the crate captures the error's [logical-trace]. I've summarized this with three labels: **context**, **enriched**, and **[SpanTrace]**.

* **context**: The logical-trace is captured as additional context is accumulated by the error type.
* **enriched**: The logical-trace is automatically enriched with file names and line numbers as context is accumulated.
* **[SpanTrace]**: The current [tracing]::Span is captured and can be used later to query or print out the tree of Spans that led to the instantiation of an error. See [tracing_error] for more details.

Finally, the [table] outlines the various ways context can be captured:

* **strings**: Strings may be attached as context as the error propagates.
* **Backtrace**: A [Backtrace] is generated when the error is captured.
* **custom types**: A user defined type is attached as context. In Accumulator style crates the type is erased and may only be retrieved through runtime reflection.
* **source error**: An unenriched error is captured as the "source". For example an underlying `io::Error` is stored as context as the root cause.
* **[SpanTrace]**: The current [tracing]::Span is stored as context when the error is captured.

With these details in mind, we can finally discuss what makes Culprit unique. Per the table, Culprit is a Hybrid between the New Type and Accumulator model. It's generic over a `Context` type which is provided by the developer. Internally, Culprit builds a [logical-trace] of physical source code locations, context changes, and notes. The combination of a dynamic [logical-trace] with a custom `Context` type provides a best-of-both-worlds experience.

When using Culprit, the highest level `Context` types should represent all the erroneous states a program can be in. These higher level types will wrap lower level `Context` types all the way down to the root cause of each error state. Because Culprit automatically captures human-readable details in the [logical-trace], the `Context` types are free to only capture what the program needs to handle error states. This allows `Context` types to be closer to a pure representation of the various error states a program can be in.

One of Culprit's goals is to make it easier for the developer to map erroneous states to error codes. This is useful when writing developer facing binaries or APIs. Culprit suggests that by keeping the `Context` as simple as possible, error codes can be defined as a mapping between paths through the `Context` type and a set of error codes. I accept that you can also achieve this statically with [thiserror] or dynamically via reflection in the other error libraries. However, I believe Culprit is the first crate to make this an explicit design goal and I hope to develop methods to make managing this error code mapping easier.

# Outstanding Work

- [x] prefix result extension fns with `or_` to be more idiomatic
- [x] context fns should return `Into<Context>`
- [ ] implement `#[derive(CulpritContext)]`
  - [ ] provides `Display` attr and impl
  - [ ] provides `From` impls for deriving Contexts from Errors or other Contexts. Would be nice to support mapping/extraction.
  - [ ] provides `Into<Result<T,Culprit>>` impl for raising new errors
- [x] rename `Fingerprint` to `Context`
- [ ] add [SpanTrace] support behind a featureflag
- [x] add `type Result<T, C> = Result<T, Culprit<C>>`
- [ ] `bail!` or similar macro for easy error generation
- [ ] document all methods and modules

[table]: #comparison-table
[logical-trace]: #logical-trace

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
[Backtrace]: https://doc.rust-lang.org/std/backtrace/index.html