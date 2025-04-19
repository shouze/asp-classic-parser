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

    // Parse the content
    let result = parser::parse(&content);

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

    // Parse the content
    let result = parser::parse(asp_code);

    // Check if parsing completes without errors
    assert!(
        result.is_ok(),
        "Parsing failed with error: {:?}",
        result.err()
    );
}

#[test]
fn test_invalid_syntax_parsing() {
    // Path to our invalid test fixture
    let fixture_path = Path::new("fixtures/failing/invalid_syntax.asp");

    // Read the test file
    let content = fs::read_to_string(fixture_path).expect("Failed to read invalid test fixture file");

    // Parse the content
    let result = parser::parse(&content);

    // Check that parsing fails for invalid syntax
    assert!(
        result.is_err(),
        "Invalid syntax was parsed successfully, but should have failed"
    );
}
