use std::fs;
use std::path::Path;

// Import our parser module
use asp_classic_parser::parser;

#[test]
fn test_basic_syntax_parsing() {
    // Path to our test fixture
    let fixture_path = Path::new("fixtures/passing/basic_syntax.asp");

    // Read the test file
    let content = fs::read_to_string(fixture_path).expect("Failed to read test fixture file");

    // Parse the content (with verbose=false)
    let result = parser::parse(&content, false);

    // Check if parsing completes without errors
    assert!(
        result.is_ok(),
        "Parsing failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_response_write_parsing() {
    // Simple ASP code with Response.Write
    let asp_code = "<%\nResponse.Write \"Hello, World!\"\n%>";

    // Parse the content (with verbose=false)
    let result = parser::parse(asp_code, false);

    // Check if parsing completes without errors
    assert!(
        result.is_ok(),
        "Parsing failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_html_asp_content() {
    // ASP code mixed with HTML, similar to examples/basic.asp
    let mixed_content = r#"<% var country ="fr"; %>
<html>
    <script language="JavaScript">
        <% Response.Write('document.country = "'+country+'"'); %>
    </script>
    <body>
    </body>
</html>"#;

    // Parse the content (with verbose=false)
    let result = parser::parse(mixed_content, false);

    // Check if parsing completes without errors
    assert!(
        result.is_ok(),
        "Parsing mixed HTML/ASP content failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_mixed_content_fixture_parsing() {
    // Path to our mixed content test fixture
    let fixture_path = Path::new("fixtures/passing/mixed_content.asp");

    // Read the test file
    let content =
        fs::read_to_string(fixture_path).expect("Failed to read mixed content fixture file");

    // Parse the content (with verbose=false)
    let result = parser::parse(&content, false);

    // Check if parsing completes without errors
    assert!(
        result.is_ok(),
        "Parsing mixed content fixture failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_invalid_syntax_parsing() {
    // Path to our invalid test fixture
    let fixture_path = Path::new("fixtures/failing/invalid_syntax.asp");

    // Read the test file
    let content =
        fs::read_to_string(fixture_path).expect("Failed to read invalid test fixture file");

    // Parse the content (with verbose=false)
    let result = parser::parse(&content, false);

    // Check that parsing fails for invalid syntax
    assert!(
        result.is_err(),
        "Invalid syntax was parsed successfully, but should have failed"
    );
}
