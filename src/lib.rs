//! # nlib — Numerical Algorithms in Rust
//!
//! Provable-contracts-first Rust port of Di Pierro's
//! "Annotated Algorithms in Python" (3rd Ed., 2023).
//!
//! Every module is specified by a YAML contract in `contracts/`
//! before implementation. Contracts define equations, preconditions,
//! postconditions, proof obligations, and falsification tests.
//!
//! ## Modules (mapped from book chapters)
//!
//! - [`matrix`] — Dense matrix algebra (Ch. 4.4)
//! - [`solve`] — Nonlinear equation solvers (Ch. 4.6)
//! - [`optimize`] — Optimization methods (Ch. 4.7-4.8)
//! - [`integrate`] — Numerical integration (Ch. 4.10)
//! - [`fourier`] — DFT/FFT (Ch. 4.11)
//! - [`random`] — PRNGs and distributions (Ch. 6)
//! - [`monte_carlo`] — Monte Carlo simulation (Ch. 7)
//! - [`graph`] — Graph algorithms (Ch. 3.7)
//! - [`sort`] — Sorting algorithms (Ch. 3.5)
//! - [`stats`] — Statistics and probability (Ch. 5)

pub mod fourier;
pub mod graph;
pub mod integrate;
pub mod matrix;
pub mod monte_carlo;
pub mod optimize;
pub mod random;
pub mod solve;
pub mod sort;
pub mod stats;
