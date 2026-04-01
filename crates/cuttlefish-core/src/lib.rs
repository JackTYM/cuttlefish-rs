#![deny(unsafe_code)]
#![warn(missing_docs)]

//! Core infrastructure for the Cuttlefish platform: errors, config, traits, and tracing.

pub mod config;
pub mod context;
pub mod error;
pub mod traits;
pub mod tracing;

pub use config::CuttlefishConfig;
pub use context::{ContextConfig, ContextManager};
pub use error::CuttlefishError;
