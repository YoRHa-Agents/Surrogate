#![warn(missing_docs)]

//! Shared types, traits, and contracts for the Surrogate proxy kernel.

/// Configuration loading, validation, and normalization.
pub mod config;
/// Core domain types: proxy units, profiles, policies, and state machines.
pub mod domain;
/// Error enums shared across crate boundaries.
pub mod error;
/// Observability events and sinks for structured telemetry.
pub mod events;
/// Health-check probe abstraction.
pub mod health;
/// Plugin capability model and async intercept trait.
pub mod plugin;
/// Rule predicate AST, compiled rule sets, and conflict descriptors.
pub mod rules;
/// Pluggable transport abstraction for outbound connections.
pub mod transport;
