//! Integration tests for the LSP server.
//! These tests ensure the Language Server Protocol implementation functions correctly.

use asp_classic_parser::lsp::parse_asp_file;

#[test]
fn test_parse_asp_file_valid() {
    // Test parsing a valid ASP file
    let content = r#"<%
    ' Valid ASP syntax
    Dim name
    name = "John"
    Response.Write("Hello, " & name)
    %>"#;

    let result = parse_asp_file("test.asp", content);
    assert!(result.is_ok(), "Should parse without errors");
}

#[test]
fn test_parse_asp_file_invalid() {
    // Test parsing an invalid ASP file with unclosed tag
    let content = r#"<%
    ' Invalid ASP syntax - missing closing tag
    Dim name
    name = "John"
    Response.Write("Hello, " & name)
    "#;

    let result = parse_asp_file("test.asp", content);
    assert!(result.is_err(), "Should detect syntax error");

    if let Err(errors) = result {
        assert!(!errors.is_empty(), "Should have at least one error");
    }
}

#[test]
fn test_parse_asp_file_no_asp_tags() {
    // Test parsing a file with no ASP tags
    let content = "<html><body>Hello, world!</body></html>";

    let result = parse_asp_file("test.asp", content);
    assert!(result.is_err(), "Should detect no ASP tags");

    if let Err(errors) = result {
        assert!(!errors.is_empty(), "Should have at least one error");
        assert!(
            errors[0].error_type == "warning",
            "Should be a warning for no ASP tags"
        );
    }
}

#[test]
fn test_parse_asp_file_empty() {
    // Test parsing an empty file
    let content = "";

    let result = parse_asp_file("test.asp", content);
    assert!(result.is_err(), "Should detect empty file");
}
