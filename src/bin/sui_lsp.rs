//! Sui Language Server Protocol (LSP) implementation
//!
//! Provides IDE features for Sui language:
//! - Diagnostics (syntax errors)
//! - Hover information
//! - Document symbols

use std::collections::HashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use sui_lang::interpreter::Parser;

/// Sui Language Server
struct SuiLanguageServer {
    client: Client,
    documents: tokio::sync::RwLock<HashMap<Url, String>>,
}

impl SuiLanguageServer {
    fn new(client: Client) -> Self {
        Self {
            client,
            documents: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Validate document and return diagnostics
    async fn validate_document(&self, _uri: &Url, text: &str) -> Vec<Diagnostic> {
        use sui_lang::interpreter::ParseError;

        let errors = Parser::validate(text);
        let mut diagnostics = Vec::new();

        for error in errors {
            let line_num = match &error {
                ParseError::InvalidInstruction(_, line) => *line,
                ParseError::MissingArguments(_, line, _, _) => *line,
                ParseError::InvalidFunctionDef(line) => *line,
                ParseError::UnmatchedBrace(line) => *line,
                ParseError::General(line, _) => *line,
            };

            let line = line_num.saturating_sub(1) as u32;
            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position { line, character: 0 },
                    end: Position { line, character: 100 },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("sui".to_string()),
                message: error.to_string(),
                ..Default::default()
            });
        }

        diagnostics
    }

    /// Get hover information for a position
    fn get_hover_info(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        let line_idx = position.line as usize;

        if line_idx >= lines.len() {
            return None;
        }

        let line = lines[line_idx].trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with(';') {
            return None;
        }

        let first_char = line.chars().next()?;

        Some(match first_char {
            '=' => "**Assignment**\n\n`= var value`\n\nAssigns a value to a variable.".to_string(),
            '+' => "**Addition**\n\n`+ result a b`\n\nAdds two values and stores in result.".to_string(),
            '-' => "**Subtraction**\n\n`- result a b`\n\nSubtracts b from a and stores in result.".to_string(),
            '*' => "**Multiplication**\n\n`* result a b`\n\nMultiplies two values and stores in result.".to_string(),
            '/' => "**Division**\n\n`/ result a b`\n\nDivides a by b and stores in result.".to_string(),
            '%' => "**Modulo**\n\n`% result a b`\n\nComputes a mod b and stores in result.".to_string(),
            '<' => "**Less Than**\n\n`< result a b`\n\nReturns 1 if a < b, else 0.".to_string(),
            '>' => "**Greater Than**\n\n`> result a b`\n\nReturns 1 if a > b, else 0.".to_string(),
            '~' => "**Equality**\n\n`~ result a b`\n\nReturns 1 if a == b, else 0.".to_string(),
            '!' => "**Logical NOT**\n\n`! result a`\n\nReturns 1 if a is 0, else 0.".to_string(),
            '&' => "**Logical AND**\n\n`& result a b`\n\nReturns 1 if both are non-zero.".to_string(),
            '|' => "**Logical OR**\n\n`| result a b`\n\nReturns 1 if either is non-zero.".to_string(),
            '?' => "**Conditional Jump**\n\n`? cond label`\n\nJumps to label if cond is non-zero.".to_string(),
            '@' => "**Unconditional Jump**\n\n`@ label`\n\nJumps to the specified label.".to_string(),
            ':' => "**Label Definition**\n\n`: label`\n\nDefines a jump target.".to_string(),
            '#' => "**Function Definition**\n\n`# id argc {`\n\nDefines a function with given id and argument count.".to_string(),
            '}' => "**Function End**\n\n`}`\n\nEnds a function definition.".to_string(),
            '$' => "**Function Call**\n\n`$ result func args...`\n\nCalls function and stores result.".to_string(),
            '^' => "**Return**\n\n`^ value`\n\nReturns a value from function.".to_string(),
            '[' => "**Array Create**\n\n`[ var size`\n\nCreates an array of given size.".to_string(),
            ']' => "**Array Read**\n\n`] result arr idx`\n\nReads value from array at index.".to_string(),
            '{' => "**Array Write**\n\n`{ arr idx value`\n\nWrites value to array at index.".to_string(),
            '.' => "**Output**\n\n`. value`\n\nPrints the value to output.".to_string(),
            ',' => "**Input**\n\n`, var`\n\nReads input into variable.".to_string(),
            'R' | 'P' => "**FFI Call**\n\n`R result \"func\" args...`\n\nCalls a builtin function.\n\nAvailable: math.sqrt, pow, sin, cos, len, abs, max, min, round, int, float, str, random.randint".to_string(),
            _ => return None,
        })
    }

    /// Get document symbols (functions and labels)
    fn get_symbols(&self, text: &str, _uri: &Url) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            let trimmed = line.trim();

            // Function definition
            if trimmed.starts_with('#') {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 3 {
                    let func_id = parts[1];
                    let argc = parts[2];
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: format!("function {}", func_id),
                        detail: Some(format!("{} args", argc)),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        children: None,
                    });
                }
            }

            // Label definition
            if trimmed.starts_with(':') {
                let parts: Vec<&str> = trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let label = parts[1];
                    #[allow(deprecated)]
                    symbols.push(DocumentSymbol {
                        name: format!("label {}", label),
                        detail: None,
                        kind: SymbolKind::KEY,
                        tags: None,
                        deprecated: None,
                        range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        selection_range: Range {
                            start: Position { line: line_idx as u32, character: 0 },
                            end: Position { line: line_idx as u32, character: line.len() as u32 },
                        },
                        children: None,
                    });
                }
            }
        }

        symbols
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SuiLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "sui-lsp".to_string(),
                version: Some(sui_lang::VERSION.to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Sui language server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.write().await.insert(uri.clone(), text.clone());

        let diagnostics = self.validate_document(&uri, &text).await;
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        if let Some(change) = params.content_changes.into_iter().next() {
            let text = change.text;
            self.documents.write().await.insert(uri.clone(), text.clone());

            let diagnostics = self.validate_document(&uri, &text).await;
            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.write().await.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let documents = self.documents.read().await;
        if let Some(text) = documents.get(uri) {
            if let Some(info) = self.get_hover_info(text, position) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = &params.text_document.uri;

        let documents = self.documents.read().await;
        if let Some(text) = documents.get(uri) {
            let symbols = self.get_symbols(text, uri);
            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| SuiLanguageServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
