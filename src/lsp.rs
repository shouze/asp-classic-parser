#[allow(deprecated)]
use dashmap::DashMap;
use log;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::parser;

/// Structure representing a parser error with additional LSP-compatible information
#[derive(Debug)]
pub struct ParseError {
    /// The error message
    pub message: String,
    /// The line number (1-based)
    pub line: Option<usize>,
    /// The column number (1-based)
    pub column: Option<usize>,
    /// The end column number (1-based)
    pub column_end: Option<usize>,
    /// The error type (error, warning, info, hint)
    pub error_type: String,
}

/// Parses an ASP Classic file and returns any errors
///
/// # Arguments
///
/// * `file_path` - The path to the file (used for error reporting)
/// * `content` - The content of the file to parse
///
/// # Returns
///
/// * `Ok(())` if parsing was successful, or
/// * `Err(Vec<ParseError>)` containing the parser errors
pub fn parse_asp_file(_file_path: &str, content: &str) -> std::result::Result<(), Vec<ParseError>> {
    match parser::parse(content, false) {
        Ok(_) => Ok(()),
        Err(err) => {
            if let Some(asp_err) = err.downcast_ref::<parser::AspParseError>() {
                let error_type = if asp_err.is_no_asp_tags_error() {
                    "warning"
                } else {
                    "error"
                };

                // Create a ParseError from the AspParseError
                // Extract line and column from the error message since we can't access private fields directly
                let message = asp_err.to_string();
                let mut line = None;
                let mut column = None;

                // Parse the error message to extract line and column
                // Format is typically: "Parse error at line X, column Y: message"
                if let Some(line_start) = message.find("line ") {
                    let line_part = &message[line_start + 5..];
                    if let Some(line_end) =
                        line_part.find(|c: char| !c.is_ascii_digit() && c != ',' && c != ' ')
                    {
                        if let Ok(line_num) = line_part[..line_end]
                            .trim_end_matches(',')
                            .trim()
                            .parse::<usize>()
                        {
                            line = Some(line_num);
                        }
                    }

                    if let Some(col_start) = line_part.find("column ") {
                        let col_part = &line_part[col_start + 7..];
                        if let Some(col_end) =
                            col_part.find(|c: char| !c.is_ascii_digit() && c != ':' && c != ' ')
                        {
                            if let Ok(col_num) = col_part[..col_end]
                                .trim_end_matches(':')
                                .trim()
                                .parse::<usize>()
                            {
                                column = Some(col_num);
                            }
                        }
                    }
                }

                let parse_error = ParseError {
                    message: asp_err.to_string(),
                    line,
                    column,
                    column_end: None, // We don't have this information from the parser yet
                    error_type: error_type.to_string(),
                };

                Err(vec![parse_error])
            } else {
                // For other error types, create a generic error
                let parse_error = ParseError {
                    message: err.to_string(),
                    line: None,
                    column: None,
                    column_end: None,
                    error_type: "error".to_string(),
                };

                Err(vec![parse_error])
            }
        }
    }
}

/// Structure representing an entry in the diagnostics cache
#[derive(Debug, Clone)]
struct DiagnosticCacheEntry {
    /// Content of the file
    content: String,
    /// Diagnostics for the file
    diagnostics: Vec<Diagnostic>,
    /// Timestamp of when the diagnostics were calculated
    timestamp: Instant,
}

/// The ASP Classic Language Server
#[derive(Debug)]
pub struct AspLspServer {
    /// The client connection
    client: Client,
    /// Document store for currently open documents
    documents: DashMap<Url, String>,
    /// Cache of the last diagnostics results to avoid re-parsing unchanged files
    diagnostics_cache: Arc<Mutex<HashMap<PathBuf, DiagnosticCacheEntry>>>,
}

impl AspLspServer {
    /// Create a new ASP Classic Language Server
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            diagnostics_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Convert a VS Code file URI to a file path
    fn uri_to_path(&self, uri: &Url) -> Option<PathBuf> {
        uri.to_file_path().ok()
    }

    /// Get the document content from an open document or file
    async fn get_document_content(&self, uri: &Url) -> Option<String> {
        if let Some(content) = self.documents.get(uri) {
            // Document is open, return the in-memory content
            return Some(content.clone());
        }

        // Document not open, try to read from file
        if let Some(path) = self.uri_to_path(uri) {
            match tokio::fs::read_to_string(path).await {
                Ok(content) => Some(content),
                Err(err) => {
                    log::error!("Failed to read file {}: {}", uri, err);
                    None
                }
            }
        } else {
            log::error!("Failed to convert URI to path: {}", uri);
            None
        }
    }

    /// Get the file extension from a URI
    fn get_file_extension(&self, uri: &Url) -> Option<String> {
        self.uri_to_path(uri).and_then(|path| {
            path.extension()
                .map(|ext| ext.to_string_lossy().to_string())
        })
    }

    /// Check if a file should be parsed based on its extension
    fn should_parse_file(&self, uri: &Url) -> bool {
        if let Some(ext) = self.get_file_extension(uri) {
            // Parse ASP and VBS files
            ext.eq_ignore_ascii_case("asp") || ext.eq_ignore_ascii_case("vbs")
        } else {
            false
        }
    }

    /// Convert a parse error to a LSP diagnostic
    fn parse_error_to_diagnostic(&self, error: ParseError, file_content: &str) -> Diagnostic {
        // Get line number (1-based) from the error
        let line_number = error.line.unwrap_or(1);

        // Convert to 0-based line number for LSP
        let line = line_number.saturating_sub(1);

        // Get the line content to determine the column range
        let file_lines: Vec<&str> = file_content.lines().collect();
        let line_content = if line < file_lines.len() {
            file_lines[line]
        } else {
            ""
        };

        // Set column range (if not available, highlight the whole line)
        let (start_char, end_char) = match (error.column, error.column_end) {
            (Some(col), Some(col_end)) => (col.saturating_sub(1), col_end),
            (Some(col), None) => (col.saturating_sub(1), line_content.len()),
            _ => (0, line_content.len()),
        };

        // Create the diagnostic range
        let range = Range {
            start: Position {
                line: line as u32,
                character: start_char as u32,
            },
            end: Position {
                line: line as u32,
                character: end_char as u32,
            },
        };

        // Determine the diagnostic severity based on error type
        let severity = match error.error_type.as_str() {
            "error" => Some(DiagnosticSeverity::ERROR),
            "warning" => Some(DiagnosticSeverity::WARNING),
            "hint" => Some(DiagnosticSeverity::HINT),
            "info" => Some(DiagnosticSeverity::INFORMATION),
            _ => Some(DiagnosticSeverity::ERROR), // Default to error
        };

        // Create the diagnostic
        Diagnostic {
            range,
            severity,
            code: None,
            code_description: None,
            source: Some("asp-classic-parser".to_string()),
            message: error.message,
            related_information: None,
            tags: None,
            data: None,
        }
    }

    /// Parse a document and return diagnostics
    async fn parse_document(&self, uri: &Url) -> Vec<Diagnostic> {
        // Check if this is a file we should parse
        if !self.should_parse_file(uri) {
            return Vec::new();
        }

        // Get the file path
        let file_path = match self.uri_to_path(uri) {
            Some(path) => path,
            None => return Vec::new(),
        };

        // Get document content
        let content = match self.get_document_content(uri).await {
            Some(content) => content,
            None => return Vec::new(),
        };

        // Check the cache - if the content hasn't changed, return cached diagnostics
        {
            let cache = self.diagnostics_cache.lock().await;
            if let Some(cached_entry) = cache.get(&file_path) {
                if cached_entry.content == content {
                    log::debug!("Using cached diagnostics for {}", uri);
                    return cached_entry.diagnostics.clone();
                }
            }
        }

        // Convert the URI to a string path for parsing
        let path_str = file_path.to_string_lossy();

        // Parse the document
        let parse_result = match parse_asp_file(&path_str, &content) {
            Ok(_) => Vec::new(), // No errors
            Err(errors) => errors
                .into_iter()
                .map(|err| self.parse_error_to_diagnostic(err, &content))
                .collect(),
        };

        // Update the cache
        {
            let mut cache = self.diagnostics_cache.lock().await;
            cache.insert(
                file_path,
                DiagnosticCacheEntry {
                    content,
                    diagnostics: parse_result.clone(),
                    timestamp: Instant::now(),
                },
            );
        }

        parse_result
    }

    /// Validate a document and publish diagnostics
    async fn validate_document(&self, uri: Url) {
        // Parse the document to get diagnostics
        let diagnostics = self.parse_document(&uri).await;

        // Publish the diagnostics
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    /// Cleanup the diagnostics cache periodically
    async fn cleanup_diagnostics_cache(&self) {
        // Get current time
        let now = Instant::now();

        // Maximum age for cache entries (1 hour)
        let max_age = std::time::Duration::from_secs(3600);

        // Lock the cache and remove old entries
        let mut cache = self.diagnostics_cache.lock().await;
        let old_paths: Vec<PathBuf> = cache
            .iter()
            .filter_map(|entry| {
                let path = entry.0;
                let timestamp = entry.1.timestamp;
                if now.duration_since(timestamp) > max_age {
                    Some(path.clone())
                } else {
                    None
                }
            })
            .collect();

        // Remove old entries
        for path in old_paths {
            cache.remove(&path);
        }
    }

    /// Convert a position (line, character) to an offset in the text
    fn position_to_offset(&self, text: &str, position: Position) -> Option<usize> {
        let mut lines = text.split('\n');
        let mut offset = 0;

        // Find the offset for the line
        for _ in 0..position.line {
            if let Some(line) = lines.next() {
                offset += line.len() + 1; // +1 for the newline
            } else {
                return None;
            }
        }

        // Add the character offset
        if let Some(line) = lines.next() {
            if position.character as usize <= line.len() {
                offset += position.character as usize;
                Some(offset)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get the word at a given position
    fn get_word_at_position(&self, text: &str, position: Position) -> Option<String> {
        // Get the line at the position
        let lines: Vec<&str> = text.lines().collect();
        let line = position.line as usize;

        if line >= lines.len() {
            return None;
        }

        let line_text = lines[line];
        let character = position.character as usize;

        if character >= line_text.len() {
            return None;
        }

        // Find the word boundaries
        let mut start = character;
        let mut end = character;

        // Move to the start of the word
        while start > 0 {
            let prev_char = line_text.chars().nth(start - 1).unwrap_or(' ');
            if !prev_char.is_alphanumeric() && prev_char != '_' {
                break;
            }
            start -= 1;
        }

        // Move to the end of the word
        while end < line_text.len() {
            let next_char = line_text.chars().nth(end).unwrap_or(' ');
            if !next_char.is_alphanumeric() && next_char != '_' {
                break;
            }
            end += 1;
        }

        // Extract the word
        if start < end {
            Some(line_text[start..end].to_string())
        } else {
            None
        }
    }

    /// Check if a position is inside ASP tags
    fn is_position_in_asp_tag(&self, text: &str, position: Position) -> bool {
        // Get the line at the position
        let lines: Vec<&str> = text.lines().collect();
        let line = position.line as usize;

        if line >= lines.len() {
            return false;
        }

        // Try to find the nearest ASP tags
        let mut in_asp_tag = false;
        let mut open_pos = 0;

        for (i, current_line) in lines.iter().enumerate().take(line + 1) {
            // Check for ASP open tags
            let mut line_offset = 0;
            while let Some(pos) = current_line[line_offset..].find("<%") {
                let absolute_pos = line_offset + pos;
                open_pos = absolute_pos;
                in_asp_tag = true;
                line_offset = absolute_pos + 2;
            }

            // Check for ASP close tags
            let mut line_offset = 0;
            while let Some(pos) = current_line[line_offset..].find("%>") {
                let absolute_pos = line_offset + pos;
                if i == line && absolute_pos < position.character as usize && in_asp_tag {
                    in_asp_tag = false;
                }
                line_offset = absolute_pos + 2;
            }

            // If we're on the current line, check position
            if i == line {
                if in_asp_tag {
                    return position.character as usize > open_pos;
                }
                break;
            }
        }

        in_asp_tag
    }

    /// Generate code completions based on context
    fn generate_completions(&self, text: &str, position: Position) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        // Get the line at the position
        let lines: Vec<&str> = text.lines().collect();
        let line = position.line as usize;

        if line >= lines.len() {
            return completions;
        }

        let line_text = lines[line];
        let prefix = &line_text[..position.character as usize];

        // Add VBScript keywords
        if !prefix.trim().is_empty() {
            let keywords = [
                "Dim",
                "Set",
                "If",
                "Then",
                "Else",
                "ElseIf",
                "End",
                "For",
                "Next",
                "Do",
                "Loop",
                "While",
                "Wend",
                "Select",
                "Case",
                "Function",
                "Sub",
                "Class",
                "With",
                "Option",
                "Explicit",
                "Call",
                "Exit",
                "On",
                "Error",
                "Resume",
                "Next",
                "Response",
                "Request",
                "Session",
                "Application",
                "Server",
                "ASPError",
            ];

            for keyword in &keywords {
                if keyword
                    .to_lowercase()
                    .starts_with(&prefix.trim().to_lowercase())
                {
                    completions.push(CompletionItem {
                        label: keyword.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(format!("VBScript keyword: {}", keyword)),
                        insert_text: Some(keyword.to_string()),
                        ..CompletionItem::default()
                    });
                }
            }
        }

        // Add ASP built-in objects
        if prefix.trim_end().ends_with(".") {
            let object_prefix = prefix.trim_end().trim_end_matches(".");

            // Response object methods
            if object_prefix.eq_ignore_ascii_case("Response") {
                let methods = [
                    "Write",
                    "End",
                    "Flush",
                    "Clear",
                    "Redirect",
                    "Buffer",
                    "ContentType",
                ];

                for method in &methods {
                    completions.push(CompletionItem {
                        label: method.to_string(),
                        kind: Some(CompletionItemKind::METHOD),
                        detail: Some(format!("Response.{} - ASP Response object method", method)),
                        insert_text: Some(method.to_string()),
                        ..CompletionItem::default()
                    });
                }
            }

            // Request object methods
            if object_prefix.eq_ignore_ascii_case("Request") {
                let methods = [
                    "QueryString",
                    "Form",
                    "Cookies",
                    "ServerVariables",
                    "TotalBytes",
                ];

                for method in &methods {
                    completions.push(CompletionItem {
                        label: method.to_string(),
                        kind: Some(CompletionItemKind::METHOD),
                        detail: Some(format!("Request.{} - ASP Request object method", method)),
                        insert_text: Some(method.to_string()),
                        ..CompletionItem::default()
                    });
                }
            }
        }

        // Add VBScript built-in functions
        let functions = [
            "Abs",
            "Array",
            "Asc",
            "AscB",
            "AscW",
            "Chr",
            "ChrB",
            "ChrW",
            "CBool",
            "CByte",
            "CCur",
            "CDate",
            "CDbl",
            "CInt",
            "CLng",
            "CSng",
            "CStr",
            "DateAdd",
            "DateDiff",
            "DatePart",
            "DateSerial",
            "DateValue",
            "Date",
            "Day",
            "FormatCurrency",
            "FormatDateTime",
            "FormatNumber",
            "FormatPercent",
            "Hour",
            "InStr",
            "InStrB",
            "InStrRev",
            "Join",
            "LBound",
            "LCase",
            "Left",
            "LeftB",
            "Len",
            "LenB",
            "Mid",
            "MidB",
            "Minute",
            "Month",
            "MonthName",
            "Now",
            "Replace",
            "Right",
            "RightB",
            "Round",
            "Second",
            "Split",
            "Sqr",
            "StrComp",
            "String",
            "StrReverse",
            "Time",
            "Timer",
            "TimeSerial",
            "TimeValue",
            "Trim",
            "TypeName",
            "UBound",
            "UCase",
            "VarType",
            "Weekday",
            "WeekdayName",
            "Year",
        ];

        for function in &functions {
            if function
                .to_lowercase()
                .starts_with(&prefix.trim().to_lowercase())
            {
                completions.push(CompletionItem {
                    label: function.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail: Some(format!("{} - VBScript built-in function", function)),
                    insert_text: Some(format!("{}()", function)),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    ..CompletionItem::default()
                });
            }
        }

        completions
    }

    /// Provide hover content for common ASP/VBScript elements
    fn get_hover_content(&self, word: &str) -> Option<String> {
        // Match common ASP/VBScript keywords and objects
        match word.to_lowercase().as_str() {
            "response" => Some("**Response** Object\n\nThe ASP Response object is used to send output to the client.\n\nCommon methods:\n- Response.Write(string) - Writes content to the page\n- Response.End() - Ends the response\n- Response.Redirect(url) - Redirects to another URL".to_string()),            
            "request" => Some("**Request** Object\n\nThe ASP Request object is used to get information from the client.\n\nCommon properties:\n- Request.QueryString(name) - Gets query string values\n- Request.Form(name) - Gets form values\n- Request.Cookies(name) - Gets cookie values\n- Request.ServerVariables(name) - Gets server environment variables".to_string()),            
            "session" => Some("**Session** Object\n\nThe ASP Session object is used to store information for a user session.\n\nCommon methods and properties:\n- Session(name) - Gets or sets a session variable\n- Session.Timeout - Gets or sets the timeout period\n- Session.Abandon - Destroys a session".to_string()),            
            "application" => Some("**Application** Object\n\nThe ASP Application object is used to store information for the entire application.\n\nCommon methods and properties:\n- Application(name) - Gets or sets an application variable\n- Application.Lock - Locks application variables for writing\n- Application.Unlock - Unlocks application variables".to_string()),            
            "server" => Some("**Server** Object\n\nThe ASP Server object is used to access server properties and methods.\n\nCommon methods:\n- Server.CreateObject(progID) - Creates an instance of a COM object\n- Server.MapPath(path) - Maps a virtual path to a physical path\n- Server.HTMLEncode(string) - Encodes HTML special characters\n- Server.URLEncode(string) - Encodes URL special characters".to_string()),            
            "dim" => Some("**Dim** Statement\n\nUsed to declare variables.\n\nExample:\n```vb\nDim name, age, isActive\nDim users(10)  ' Array with 11 elements (0-10)\n```".to_string()),            
            "if" => Some("**If...Then...Else** Statement\n\nConditional execution structure.\n\nExample:\n```vb\nIf condition Then\n   ' Code to execute when condition is true\nElseIf anotherCondition Then\n   ' Code to execute when anotherCondition is true\nElse\n   ' Code to execute when all conditions are false\nEnd If\n```".to_string()),            
            "for" => Some("**For...Next** Loop\n\nRepeating code a specific number of times.\n\nExample:\n```vb\nFor i = 1 To 10\n   ' Code to execute\nNext\n```".to_string()),            
            "function" => Some("**Function** Statement\n\nDeclares a function that returns a value.\n\nExample:\n```vb\nFunction CalculateTotal(price, quantity)\n   CalculateTotal = price * quantity\nEnd Function\n```".to_string()),            
            "sub" => Some("**Sub** Statement\n\nDeclares a subroutine that doesn't return a value.\n\nExample:\n```vb\nSub DisplayMessage(message)\n   Response.Write message\nEnd Sub\n```".to_string()),
            "class" => Some("**Class** Statement\n\nDeclares a class definition.\n\nExample:\n```vb\nClass Person\n   Private m_name\n   \n   Public Property Get Name\n       Name = m_name\n   End Property\n   \n   Public Property Let Name(value)\n       m_name = value\n   End Property\nEnd Class\n```".to_string()),            
            "option" => Some("**Option Explicit** Statement\n\nForces explicit declaration of all variables in a script.\n\nExample:\n```vb\nOption Explicit\n\n' Now all variables must be declared with Dim\nDim name\nname = \"John\"  ' Correct\n' age = 30  ' This would cause an error\n```".to_string()),            
            _ => None,
        }
    }

    /// Extract document symbols from content
    fn extract_document_symbols(&self, content: &str) -> Vec<DocumentSymbol> {
        let mut symbols = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Track function-level constructs
        let mut function_stack: Vec<(DocumentSymbol, usize)> = Vec::new();

        // Regular expressions for matching syntax structures
        let function_regex = regex::Regex::new(r"(?i)^\s*(function|sub)\s+([a-z0-9_]+)").unwrap();
        let end_function_regex = regex::Regex::new(r"(?i)^\s*end\s+(function|sub)").unwrap();
        let class_regex = regex::Regex::new(r"(?i)^\s*class\s+([a-z0-9_]+)").unwrap();
        let end_class_regex = regex::Regex::new(r"(?i)^\s*end\s+class").unwrap();
        let dim_regex = regex::Regex::new(r"(?i)^\s*dim\s+([a-z0-9_,\s]+)").unwrap();

        for (i, line) in lines.iter().enumerate() {
            let line_trimmed = line.trim();

            // Skip lines outside ASP tags
            if !self.is_line_in_asp_tag(content, i) {
                continue;
            }

            // Check for Function/Sub declarations
            if let Some(caps) = function_regex.captures(line_trimmed) {
                let kind = caps.get(1).unwrap().as_str();
                let name = caps.get(2).unwrap().as_str();

                let symbol = DocumentSymbol {
                    name: name.to_string(),
                    detail: Some(format!("{} {}", kind, name)),
                    kind: if kind.eq_ignore_ascii_case("sub") {
                        SymbolKind::FUNCTION
                    } else {
                        SymbolKind::METHOD
                    },
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    children: Some(Vec::new()),
                    tags: None,
                    deprecated: None,
                };

                function_stack.push((symbol, i));
            }

            // Check for End Function/Sub
            if end_function_regex.captures(line_trimmed).is_some() {
                if let Some((mut symbol, _)) = function_stack.pop() {
                    // Update the end range
                    symbol.range.end = Position {
                        line: i as u32,
                        character: line.len() as u32,
                    };

                    // Add to the parent or directly to the symbols list
                    if let Some((parent, _)) = function_stack.last_mut() {
                        if let Some(children) = &mut parent.children {
                            children.push(symbol);
                        }
                    } else {
                        symbols.push(symbol);
                    }
                }
            }

            // Check for Class declarations
            if let Some(caps) = class_regex.captures(line_trimmed) {
                let name = caps.get(1).unwrap().as_str();

                let symbol = DocumentSymbol {
                    name: name.to_string(),
                    detail: Some(format!("Class {}", name)),
                    kind: SymbolKind::CLASS,
                    range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    selection_range: Range {
                        start: Position {
                            line: i as u32,
                            character: 0,
                        },
                        end: Position {
                            line: i as u32,
                            character: line.len() as u32,
                        },
                    },
                    children: Some(Vec::new()),
                    tags: None,
                    deprecated: None,
                };

                function_stack.push((symbol, i));
            }

            // Check for End Class
            if end_class_regex.captures(line_trimmed).is_some() {
                if let Some((mut symbol, _)) = function_stack.pop() {
                    // Update the end range
                    symbol.range.end = Position {
                        line: i as u32,
                        character: line.len() as u32,
                    };

                    // Add to the parent or directly to the symbols list
                    if let Some((parent, _)) = function_stack.last_mut() {
                        if let Some(children) = &mut parent.children {
                            children.push(symbol);
                        }
                    } else {
                        symbols.push(symbol);
                    }
                }
            }

            // Check for variable declarations (Dim statements)
            if let Some(caps) = dim_regex.captures(line_trimmed) {
                let vars = caps.get(1).unwrap().as_str();

                // Split variables (they may be comma-separated)
                for var in vars.split(',') {
                    let var_name = var.trim();
                    if !var_name.is_empty() {
                        let var_symbol = DocumentSymbol {
                            name: var_name.to_string(),
                            detail: Some(format!("Variable {}", var_name)),
                            kind: SymbolKind::VARIABLE,
                            range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: line.len() as u32,
                                },
                            },
                            selection_range: Range {
                                start: Position {
                                    line: i as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: i as u32,
                                    character: line.len() as u32,
                                },
                            },
                            children: None,
                            tags: None,
                            deprecated: None,
                        };

                        // Add to the current function/class or directly to the symbols list
                        if let Some((parent, _)) = function_stack.last_mut() {
                            if let Some(children) = &mut parent.children {
                                children.push(var_symbol);
                            }
                        } else {
                            symbols.push(var_symbol);
                        }
                    }
                }
            }
        }

        symbols
    }

    /// Check if a line is within ASP tags
    fn is_line_in_asp_tag(&self, content: &str, line_index: usize) -> bool {
        let lines: Vec<&str> = content.lines().collect();

        // Search for an open ASP tag before or on this line
        let mut nearest_open = None;
        let mut nearest_close = None;

        for (i, line) in lines.iter().enumerate().take(line_index + 1) {
            if line.contains("<%") {
                nearest_open = Some(i);
            }
            if line.contains("%>") {
                nearest_close = Some(i);
            }
        }

        // If we found an open tag, check if there's a close tag between it and the current line
        if let Some(open_idx) = nearest_open {
            if let Some(close_idx) = nearest_close {
                // If close tag comes after open tag, check if our line is in between
                if close_idx > open_idx {
                    return line_index >= open_idx && line_index <= close_idx;
                }
            }
            // If no close tag found or it's before the open tag, the line is inside ASP tags
            return line_index >= open_idx;
        }

        false
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for AspLspServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        log::info!("ASP Classic Language Server initialized");

        // Set up the server capabilities
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string(), "<".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                document_symbol_provider: Some(OneOf::Left(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: "ASP Classic Language Server".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        log::info!("ASP Classic Language Server is now fully initialized");

        // Start a background task to periodically clean up the diagnostics cache
        let server = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(1800)); // 30 minutes
            loop {
                interval.tick().await;
                server.cleanup_diagnostics_cache().await;
            }
        });
    }

    async fn shutdown(&self) -> Result<()> {
        log::info!("ASP Classic Language Server shutting down");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        log::info!("Document opened: {}", uri);

        // Store the document
        self.documents.insert(uri.clone(), text);

        // Validate the document
        self.validate_document(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;

        // Get the current document
        if let Some(mut document) = self.documents.get_mut(&uri) {
            // Apply the changes to the document
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Convert the range to string indices
                    let start_offset = self.position_to_offset(&document, range.start);
                    let end_offset = self.position_to_offset(&document, range.end);

                    if let (Some(start), Some(end)) = (start_offset, end_offset) {
                        // Replace the text in the range
                        let mut new_text = document[..start].to_string();
                        new_text.push_str(&change.text);
                        new_text.push_str(&document[end..]);
                        *document = new_text;
                    }
                } else {
                    // Full document update
                    *document = change.text;
                }
            }
        } else {
            log::warn!("Document not found in memory: {}", uri);
            return;
        }

        // Validate the document with a small delay to avoid excessive parsing during typing
        let server_uri = uri.clone();
        let server = self.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            server.validate_document(server_uri).await;
        });
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;

        log::info!("Document saved: {}", uri);

        // If text is provided, update the document
        if let Some(text) = params.text {
            self.documents.insert(uri.clone(), text);
        }

        // Validate the document immediately on save
        self.validate_document(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;

        log::info!("Document closed: {}", uri);

        // Remove the document from memory
        self.documents.remove(&uri);

        // Clear diagnostics for closed document
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        // Get document content
        if let Some(content) = self.get_document_content(&uri).await {
            // Find the word at the position
            if let Some(word) = self.get_word_at_position(&content, position) {
                // Provide hover information based on the word
                if let Some(hover_content) = self.get_hover_content(&word) {
                    return Ok(Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: hover_content,
                        }),
                        range: None,
                    }));
                }
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        // Get document content
        if let Some(content) = self.get_document_content(&uri).await {
            // Check if we're inside ASP tags
            if self.is_position_in_asp_tag(&content, position) {
                // Generate completions based on context
                let items = self.generate_completions(&content, position);
                if !items.is_empty() {
                    return Ok(Some(CompletionResponse::Array(items)));
                }
            }
        }

        Ok(None)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;

        // Get document content
        if let Some(content) = self.get_document_content(&uri).await {
            // Parse the document to extract symbols
            let symbols = self.extract_document_symbols(&content);
            if !symbols.is_empty() {
                return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
            }
        }

        Ok(None)
    }
}

impl Clone for AspLspServer {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            documents: self.documents.clone(),
            diagnostics_cache: self.diagnostics_cache.clone(),
        }
    }
}
