# Contributing Guide

Thank you for your interest in improving **GPUI Learning**! This project pins the Rust
compiler using [`rust-toolchain.toml`](./rust-toolchain.toml); install the listed nightly
toolchain and components (`clippy` and `rustfmt`) before contributing.

## Linting and Documentation Expectations

* All warnings are treated as errors. The workspace enforces `clippy::all` and
  `clippy::pedantic` by default through `.cargo/config.toml`.
* Public APIs must include rustdoc comments. Crates expose their README content as crate
  documentation and opt into the `missing_docs` and `unreachable_pub` lints, so new items
  need thorough docs or should remain private.
* If a pedantic lint must be allowed, document the rationale directly in
  [`clippy.toml`](./clippy.toml) so future contributors understand the trade-off.

## Running Checks Locally

Run the following commands from the repository root before submitting a pull request:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace
```

These are the same commands executed in continuous integration.
