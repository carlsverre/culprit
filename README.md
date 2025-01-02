# culprit
A Rust error crate with the goal of identifying precisely where and in which context an error occurs.

**Goals:**
1. Context both in the logical control flow as well as physical space in files
2. Unique public facing errors
3. Minimal error sets per function/module
4. Aligning errors to error codes for external handling (i.e. outside of rust)

> [!WARNING]  
> Culprit is extremely-alpha and may dramatically change at any point. It's currently undergoing design and testing within some projects. Ideas and discussion is welcome during this process.

## Getting started

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