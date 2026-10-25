//! Syngard core: deterministic inventory, bundle manifests, diff and
//! verification for AI agent instruction bundles.
//!
//! This crate performs no network access and never executes inspected
//! content. It has no notion of approval authority: a matching baseline
//! detects drift but never authorizes anything.

pub mod diff;
pub mod digest;
pub mod inventory;
pub mod manifest;
pub mod verify;

pub use digest::Sha256Digest;
