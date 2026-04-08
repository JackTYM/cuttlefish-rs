//! LSP client for communicating with language servers.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};

use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, DocumentSymbolParams, DocumentSymbolResponse,
    GotoDefinitionParams, GotoDefinitionResponse, HoverParams, HoverProviderCapability,
    InitializeParams, InitializeResult, InitializedParams, Location, Position,
    PublishDiagnosticsParams, ServerCapabilities, TextDocumentIdentifier, TextDocumentItem,
    TextDocumentPositionParams, Uri, WorkspaceFolder,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex, mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::error::LspError;

/// Convert a file path to an LSP Uri.
fn path_to_uri(path: &Path) -> Result<Uri, ()> {
    url::Url::from_file_path(path)
        .map(|u| u.as_str().parse::<Uri>().map_err(|_| ()))
        .and_then(|r| r)
}

/// JSON-RPC request ID.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum RequestId {
    /// Integer ID.
    Int(i64),
    /// String ID.
    String(String),
}

impl From<i64> for RequestId {
    fn from(id: i64) -> Self {
        RequestId::Int(id)
    }
}

/// JSON-RPC request.
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: &'static str,
    id: RequestId,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// JSON-RPC notification (no response expected).
#[derive(Debug, Serialize)]
struct JsonRpcNotification {
    jsonrpc: &'static str,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<Value>,
}

/// JSON-RPC response.
#[derive(Debug, Deserialize)]
struct JsonRpcResponse {
    #[allow(dead_code)]
    jsonrpc: String,
    id: Option<RequestId>,
    result: Option<Value>,
    error: Option<JsonRpcError>,
}

/// JSON-RPC error.
#[derive(Debug, Deserialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

/// LSP client for a single language server.
pub struct LspClient {
    /// Child process handle.
    child: Arc<Mutex<Option<Child>>>,
    /// Stdin writer channel.
    stdin_tx: mpsc::Sender<String>,
    /// Pending requests.
    pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>>,
    /// Request ID counter.
    next_id: AtomicI64,
    /// Server capabilities.
    capabilities: Arc<Mutex<Option<ServerCapabilities>>>,
    /// Whether the client is initialized.
    initialized: Arc<Mutex<bool>>,
    /// Root URI for the workspace.
    root_uri: Option<Uri>,
    /// Diagnostics callback channel (for future push-based diagnostics).
    #[allow(dead_code)]
    diagnostics_tx: Option<mpsc::Sender<PublishDiagnosticsParams>>,
}

impl LspClient {
    /// Spawn a language server process.
    ///
    /// # Arguments
    /// * `command` - The command to run (e.g., "rust-analyzer", "typescript-language-server")
    /// * `args` - Command arguments
    /// * `root_path` - Root path for the workspace
    pub async fn spawn(
        command: &str,
        args: &[&str],
        root_path: Option<&Path>,
    ) -> Result<Self, LspError> {
        Self::spawn_with_env(command, args, &[], root_path).await
    }

    /// Spawn a language server with environment variables.
    pub async fn spawn_with_env(
        command: &str,
        args: &[&str],
        env: &[(&str, &str)],
        root_path: Option<&Path>,
    ) -> Result<Self, LspError> {
        info!("Spawning LSP server: {} {:?}", command, args);

        let mut cmd = Command::new(command);
        cmd.args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        for (key, value) in env {
            cmd.env(key, value);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| LspError::SpawnFailed(e.to_string()))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| LspError::SpawnFailed("Failed to get stdin".to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| LspError::SpawnFailed("Failed to get stdout".to_string()))?;

        // Channel for sending to stdin
        let (stdin_tx, mut stdin_rx) = mpsc::channel::<String>(100);

        // Pending requests
        let pending: Arc<Mutex<HashMap<RequestId, oneshot::Sender<JsonRpcResponse>>>> =
            Arc::new(Mutex::new(HashMap::new()));

        // Spawn stdin writer
        let mut stdin_writer = stdin;
        tokio::spawn(async move {
            while let Some(msg) = stdin_rx.recv().await {
                // LSP uses Content-Length header
                let header = format!("Content-Length: {}\r\n\r\n", msg.len());
                if stdin_writer.write_all(header.as_bytes()).await.is_err() {
                    break;
                }
                if stdin_writer.write_all(msg.as_bytes()).await.is_err() {
                    break;
                }
                if stdin_writer.flush().await.is_err() {
                    break;
                }
            }
        });

        // Spawn stdout reader
        let pending_clone = pending.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            let mut header_line = String::new();

            loop {
                header_line.clear();

                // Read headers until empty line
                let mut content_length: Option<usize> = None;
                loop {
                    header_line.clear();
                    match reader.read_line(&mut header_line).await {
                        Ok(0) => return, // EOF
                        Ok(_) => {
                            let line = header_line.trim();
                            if line.is_empty() {
                                break; // End of headers
                            }
                            if let Some(len_str) = line.strip_prefix("Content-Length: ") {
                                content_length = len_str.parse().ok();
                            }
                        }
                        Err(e) => {
                            error!("LSP read error: {}", e);
                            return;
                        }
                    }
                }

                // Read content
                let Some(len) = content_length else {
                    warn!("Missing Content-Length header");
                    continue;
                };

                let mut content = vec![0u8; len];
                if let Err(e) = tokio::io::AsyncReadExt::read_exact(&mut reader, &mut content).await
                {
                    error!("Failed to read LSP content: {}", e);
                    return;
                }

                let content_str = match String::from_utf8(content) {
                    Ok(s) => s,
                    Err(e) => {
                        warn!("Invalid UTF-8 in LSP response: {}", e);
                        continue;
                    }
                };

                // Parse response
                match serde_json::from_str::<JsonRpcResponse>(&content_str) {
                    Ok(response) => {
                        if let Some(id) = &response.id {
                            let mut pending = pending_clone.lock().await;
                            if let Some(tx) = pending.remove(id) {
                                let _ = tx.send(response);
                            }
                        }
                        // Notifications (no id) are handled separately
                    }
                    Err(e) => {
                        debug!("Failed to parse LSP response: {} - {}", e, content_str);
                    }
                }
            }
        });

        let root_uri = root_path.and_then(|p| path_to_uri(p).ok());

        Ok(Self {
            child: Arc::new(Mutex::new(Some(child))),
            stdin_tx,
            pending,
            next_id: AtomicI64::new(1),
            capabilities: Arc::new(Mutex::new(None)),
            initialized: Arc::new(Mutex::new(false)),
            root_uri,
            diagnostics_tx: None,
        })
    }

    /// Initialize the language server.
    pub async fn initialize(&self) -> Result<InitializeResult, LspError> {
        let root_uri = self.root_uri.clone();

        #[allow(deprecated)]
        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_path: None,
            root_uri: root_uri.clone(),
            initialization_options: None,
            capabilities: ClientCapabilities::default(),
            trace: None,
            workspace_folders: root_uri.map(|uri| {
                vec![WorkspaceFolder {
                    uri,
                    name: "workspace".to_string(),
                }]
            }),
            client_info: Some(lsp_types::ClientInfo {
                name: "cuttlefish".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            locale: None,
            work_done_progress_params: Default::default(),
        };

        let result: InitializeResult = self.request("initialize", Some(params)).await?;

        // Store capabilities
        {
            let mut caps = self.capabilities.lock().await;
            *caps = Some(result.capabilities.clone());
        }

        // Send initialized notification
        self.notify("initialized", Some(InitializedParams {}))
            .await?;

        {
            let mut init = self.initialized.lock().await;
            *init = true;
        }

        info!("LSP server initialized");
        Ok(result)
    }

    /// Check if initialized.
    pub async fn is_initialized(&self) -> bool {
        *self.initialized.lock().await
    }

    /// Get server capabilities.
    pub async fn capabilities(&self) -> Option<ServerCapabilities> {
        self.capabilities.lock().await.clone()
    }

    /// Open a text document.
    pub async fn did_open(&self, uri: &Uri, language_id: &str, text: &str) -> Result<(), LspError> {
        self.ensure_initialized().await?;

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri: uri.clone(),
                language_id: language_id.to_string(),
                version: 1,
                text: text.to_string(),
            },
        };

        self.notify("textDocument/didOpen", Some(params)).await
    }

    /// Get diagnostics by opening a file and waiting for publishDiagnostics.
    /// Note: This is a simplified version. In practice, diagnostics are pushed via notifications.
    pub async fn get_diagnostics(
        &self,
        path: &Path,
    ) -> Result<Vec<lsp_types::Diagnostic>, LspError> {
        self.ensure_initialized().await?;

        let uri =
            path_to_uri(path).map_err(|_| LspError::InvalidUri(path.display().to_string()))?;

        let text = tokio::fs::read_to_string(path)
            .await
            .map_err(|_| LspError::FileNotFound(path.display().to_string()))?;

        let language_id = detect_language(path);
        self.did_open(&uri, &language_id, &text).await?;

        // In a real implementation, we'd wait for publishDiagnostics notification
        // For now, return empty - diagnostics come asynchronously
        Ok(Vec::new())
    }

    /// Get document symbols.
    pub async fn document_symbols(
        &self,
        path: &Path,
    ) -> Result<Option<DocumentSymbolResponse>, LspError> {
        self.ensure_initialized().await?;

        let uri =
            path_to_uri(path).map_err(|_| LspError::InvalidUri(path.display().to_string()))?;

        let params = DocumentSymbolParams {
            text_document: TextDocumentIdentifier { uri },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.request("textDocument/documentSymbol", Some(params))
            .await
    }

    /// Go to definition.
    pub async fn goto_definition(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<GotoDefinitionResponse>, LspError> {
        self.ensure_initialized().await?;

        let uri =
            path_to_uri(path).map_err(|_| LspError::InvalidUri(path.display().to_string()))?;

        let params = GotoDefinitionParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        self.request("textDocument/definition", Some(params)).await
    }

    /// Get hover information.
    pub async fn hover(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<lsp_types::Hover>, LspError> {
        self.ensure_initialized().await?;

        // Check if hover is supported
        let caps = self.capabilities.lock().await;
        if let Some(ref caps) = *caps {
            match &caps.hover_provider {
                Some(HoverProviderCapability::Simple(true)) => {}
                Some(HoverProviderCapability::Options(_)) => {}
                _ => return Ok(None),
            }
        }
        drop(caps);

        let uri =
            path_to_uri(path).map_err(|_| LspError::InvalidUri(path.display().to_string()))?;

        let params = HoverParams {
            text_document_position_params: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
        };

        self.request("textDocument/hover", Some(params)).await
    }

    /// Find references to a symbol.
    pub async fn find_references(
        &self,
        path: &Path,
        line: u32,
        character: u32,
    ) -> Result<Option<Vec<Location>>, LspError> {
        self.ensure_initialized().await?;

        let uri =
            path_to_uri(path).map_err(|_| LspError::InvalidUri(path.display().to_string()))?;

        let params = lsp_types::ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
            context: lsp_types::ReferenceContext {
                include_declaration: true,
            },
        };

        self.request("textDocument/references", Some(params)).await
    }

    /// Shutdown the language server.
    pub async fn shutdown(&self) -> Result<(), LspError> {
        // Send shutdown request
        let _: Option<()> = self.request("shutdown", None::<()>).await?;

        // Send exit notification
        self.notify("exit", None::<()>).await?;

        // Kill process if still running
        let mut child = self.child.lock().await;
        if let Some(mut c) = child.take() {
            let _ = c.kill().await;
        }

        Ok(())
    }

    /// Send a request and wait for response.
    async fn request<P: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        method: &str,
        params: Option<P>,
    ) -> Result<R, LspError> {
        let id = RequestId::Int(self.next_id.fetch_add(1, Ordering::SeqCst));

        let request = JsonRpcRequest {
            jsonrpc: "2.0",
            id: id.clone(),
            method: method.to_string(),
            params: params.map(|p| serde_json::to_value(p).expect("serialize params")),
        };

        let json = serde_json::to_string(&request)?;

        // Register pending request
        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending.lock().await;
            pending.insert(id.clone(), tx);
        }

        // Send request
        self.stdin_tx
            .send(json)
            .await
            .map_err(|_| LspError::ServerShutdown)?;

        // Wait for response with timeout
        let response = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| LspError::Timeout)?
            .map_err(|_| LspError::ServerShutdown)?;

        // Check for error
        if let Some(err) = response.error {
            return Err(LspError::ServerError {
                code: err.code,
                message: err.message,
            });
        }

        // Parse result
        let result = response.result.unwrap_or(Value::Null);
        serde_json::from_value(result).map_err(|e| LspError::InvalidResponse(e.to_string()))
    }

    /// Send a notification (no response expected).
    async fn notify<P: Serialize>(&self, method: &str, params: Option<P>) -> Result<(), LspError> {
        let notification = JsonRpcNotification {
            jsonrpc: "2.0",
            method: method.to_string(),
            params: params.map(|p| serde_json::to_value(p).expect("serialize params")),
        };

        let json = serde_json::to_string(&notification)?;

        self.stdin_tx
            .send(json)
            .await
            .map_err(|_| LspError::ServerShutdown)?;

        Ok(())
    }

    /// Ensure the server is initialized.
    async fn ensure_initialized(&self) -> Result<(), LspError> {
        if !*self.initialized.lock().await {
            return Err(LspError::NotInitialized);
        }
        Ok(())
    }
}

/// Detect language ID from file extension.
fn detect_language(path: &Path) -> String {
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

    match ext {
        "rs" => "rust",
        "py" => "python",
        "js" => "javascript",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "jsx" => "javascriptreact",
        "go" => "go",
        "java" => "java",
        "c" => "c",
        "cpp" | "cc" | "cxx" => "cpp",
        "h" | "hpp" => "cpp",
        "rb" => "ruby",
        "php" => "php",
        "cs" => "csharp",
        "swift" => "swift",
        "kt" | "kts" => "kotlin",
        "scala" => "scala",
        "lua" => "lua",
        "sh" | "bash" => "shellscript",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" => "markdown",
        "html" => "html",
        "css" => "css",
        "scss" => "scss",
        "sql" => "sql",
        _ => "plaintext",
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(&PathBuf::from("main.rs")), "rust");
        assert_eq!(detect_language(&PathBuf::from("app.py")), "python");
        assert_eq!(detect_language(&PathBuf::from("index.ts")), "typescript");
        assert_eq!(detect_language(&PathBuf::from("unknown.xyz")), "plaintext");
    }

    #[test]
    fn test_request_id() {
        let id: RequestId = 42i64.into();
        assert_eq!(id, RequestId::Int(42));
    }
}
