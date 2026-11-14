//! megamap's engine, built on its own (no game) for `cargo test` and the `bench` example.

#[path = "../../src/engine/mod.rs"]
pub mod engine;

pub mod fake;
pub mod golden;

#[cfg(test)]
mod tests;
