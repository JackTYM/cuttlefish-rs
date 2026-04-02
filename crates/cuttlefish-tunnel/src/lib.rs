//! Cuttlefish tunnel — WebSocket-based reverse tunnel for remote access.
//!
//! This crate provides client and server components for establishing
//! secure tunnels from self-hosted Cuttlefish instances to cuttlefish.ai.

#![deny(unsafe_code)]
#![deny(clippy::unwrap_used)]
#![warn(missing_docs)]

pub mod auth;
pub mod client;
pub mod error;
pub mod protocol;
pub mod server;

pub use client::{
    ReconnectPolicy, ReconnectingTunnelClient, TunnelClient, TunnelClientConfig, TunnelEvent,
};
pub use error::TunnelError;
pub use protocol::{ClientMessage, ServerMessage};
pub use server::{TunnelServer, TunnelServerConfig};
