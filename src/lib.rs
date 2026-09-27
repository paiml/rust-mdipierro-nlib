#![allow(unused_macros)]
//! # nlib — Numerical Algorithms in Rust
//!
//! Provable-contracts-first Rust port of Di Pierro's
//! "Annotated Algorithms in Python" (Experts4Solutions, 2013;
//! ISBN 978-0991160402; revised by the author for Python 3.8).
//!
//! Every module is specified by a YAML contract in `contracts/`
//! before implementation. Contracts define equations, preconditions,
//! postconditions, proof obligations, and falsification tests.
//!
//! ## Modules (mapped from book chapters)
//!
//! - [`matrix`] — Dense matrix algebra (§4.4)
//! - [`solve`] — Nonlinear equation solvers (§4.6)
//! - [`optimize`] — Optimization methods (§4.7–4.8)
//! - [`integrate`] — Numerical integration (§4.10)
//! - [`fourier`] — DFT/FFT (§4.11)
//! - [`random`] — PRNGs and distributions (§6.4)
//! - [`monte_carlo`] — Monte Carlo simulation (Ch. 7)
//! - [`graph`] — Graph algorithms (§3.7)
//! - [`sort`] — Sorting algorithms (§3.5.5, §3.6.1)
//! - [`stats`] — Statistics and probability (Ch. 5)

include!(concat!(env!("OUT_DIR"), "/generated_contracts.rs"));

pub mod fourier;
pub mod graph;
pub mod integrate;
pub mod matrix;
pub mod monte_carlo;
pub mod optimize;
pub mod random;
pub mod receipt;
pub mod solve;
pub mod sort;
pub mod stats;

#[cfg(kani)]
mod kani_harnesses;
