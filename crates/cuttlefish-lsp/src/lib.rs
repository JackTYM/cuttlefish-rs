#![deny(unsafe_code)]
#![warn(missing_docs)]
//! LSP (Language Server Protocol) client for Cuttlefish code intelligence.
//!
//! This crate provides:
//! - LSP client for communicating with language servers
//! - Registry for managing multiple servers (one per language)
//! - Pre-configured settings for common language servers
//!
//! # Supported Language Servers
//!
//! - **Rust**: rust-analyzer
//! - **TypeScript/JavaScript**: typescript-language-server
//! - **Python**: pyright-langserver
//! - **Go**: gopls
//! - **C/C++**: clangd
//!
//! # Example
//!
//! ```ignore
//! use cuttlefish_lsp::{LspRegistry, LspServerConfig};
//! use std::path::Path;
//!
//! // Create registry and register default configs
//! let registry = LspRegistry::with_root_path("/path/to/project");
//! registry.register_defaults().await;
//!
//! // Get diagnostics for a file
//! let diagnostics = registry.get_diagnostics(Path::new("src/main.rs")).await?;
//!
//! // Get document symbols
//! let symbols = registry.get_symbols(Path::new("src/main.rs")).await?;
//!
//! // Go to definition
//! let definition = registry.goto_definition(Path::new("src/main.rs"), 10, 5).await?;
//! ```

/// LSP client implementation.
pub mod client;
/// Error types.
pub mod error;
/// Server registry.
pub mod registry;

pub use client::LspClient;
pub use error::LspError;
pub use registry::{LspRegistry, LspServerConfig};

// Re-export commonly used lsp-types
pub use lsp_types::{
    Diagnostic, DiagnosticSeverity, DocumentSymbol, DocumentSymbolResponse, GotoDefinitionResponse,
    Hover, Location, Position, Range, SymbolKind, Uri,
};
