# culprit
A Rust error crate with the goal of identifying precisely where and in which context an error occurs.

**Goals:**
1. Context both in the logical control flow as well as physical space in files
2. Unique public facing errors
3. Minimal error sets per function/module
4. Aligning errors to error codes for external handling (i.e. outside of rust)

**NOTES**

What about a visitor pattern? Each error type can define a visitor trait which permits recursively walking down a tree. This is very similar to Serde.