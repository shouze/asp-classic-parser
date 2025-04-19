/// ASP Classic Parser Module
///
/// This module provides functionality for parsing ASP Classic (VBScript) syntax using
/// the Pest parsing library. It handles the basic syntax elements of ASP Classic
/// including ASP tags, comments, statements, and expressions.
use pest::Parser;
use pest_derive::Parser;
use std::error::Error;
use std::fmt;

/// Custom error type for ASP parsing errors
#[derive(Debug)]
pub struct AspParseError {
    message: String,
    line: Option<usize>,
    column: Option<usize>,
}

impl fmt::Display for AspParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.line, self.column) {
            (Some(line), Some(column)) => write!(f, "Parse error at line {}, column {}: {}", line, column, self.message),
            (Some(line), None) => write!(f, "Parse error at line {}: {}", line, self.message),
            _ => write!(f, "Parse error: {}", self.message),
        }
    }
}

impl Error for AspParseError {}

/// The main parser for ASP Classic files
///
/// This struct implements the Parser trait from the Pest library,
/// utilizing the grammar defined in the grammar.pest file.
#[derive(Parser)]
#[grammar = "parser/grammar.pest"]
pub struct AspParser;

/// Parses an ASP Classic file and returns the result
///
/// # Arguments
///
/// * `input` - A string slice containing the ASP Classic code to parse
///
/// # Returns
///
/// * `Result<(), Box<dyn Error>>` - Ok(()) if parsing was successful, or an error
///   if parsing failed
///
/// # Examples
///
/// ```
/// use asp_classic_parser::parser;
///
/// let asp_code = "<%\nResponse.Write \"Hello, World!\"\n%>";
/// match parser::parse(asp_code) {
///     Ok(_) => println!("ASP code parsed successfully!"),
///     Err(e) => eprintln!("Error parsing ASP code: {}", e),
/// }
/// ```
pub fn parse(input: &str) -> Result<(), Box<dyn Error>> {
    // Parse the input with the file rule
    match AspParser::parse(Rule::file, input) {
        Ok(pairs) => {
            // Do some basic validation on the parse result
            let mut tag_count = 0;
            for pair in pairs {
                // Currently just logging for debugging purposes
                // In future iterations, this will build an AST
                println!("Successfully parsed ASP file structure");
                println!("Rule: {:?}", pair.as_rule());
                
                // Count ASP tags to ensure we have balanced tags
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::asp_script_block | Rule::asp_expression_block => {
                            tag_count += 1;
                        }
                        _ => {}
                    }
                }
            }
            
            // For validation purposes, ensure we have at least one ASP tag
            // This helps catch some types of invalid syntax
            if tag_count == 0 {
                return Err(Box::new(AspParseError {
                    message: "No valid ASP tags found in the file".to_string(),
                    line: None,
                    column: None,
                }));
            }
            
            Ok(())
        }
        Err(e) => {
            // Convert Pest error into our custom error with location info
            let message = format!("{}", e);
            
            // Extract line and column from the error message or use None
            // Message format is typically: "--> line:column"
            let (line, column) = extract_position_from_error(&message);
            
            Err(Box::new(AspParseError {
                message,
                line,
                column,
            }))
        }
    }
}

/// Helper function to extract position information from a Pest error message
fn extract_position_from_error(error_msg: &str) -> (Option<usize>, Option<usize>) {
    // Look for patterns like "--> 1:5" in the error message
    if let Some(pos_index) = error_msg.find("-->") {
        if let Some(line_col) = error_msg[pos_index + 3..].split_whitespace().next() {
            if let Some((line_str, col_str)) = line_col.split_once(':') {
                if let (Ok(line), Ok(column)) = (line_str.parse::<usize>(), col_str.parse::<usize>()) {
                    return (Some(line), Some(column));
                }
            }
        }
    }
    
    // Unable to extract position info
    (None, None)
}
