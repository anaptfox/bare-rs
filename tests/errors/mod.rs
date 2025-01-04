use bare_rs::BareResult;
use super::common::TestInstance;
use log::debug;

#[test]
fn test_bare_runtime_syntax_error() -> BareResult<()> {
    let instance = TestInstance::new()?;
    debug!("=== Starting syntax error test ===");
    
    unsafe {
        // Test syntax error
        let result = instance.run_script_expect_error(
            "this is not valid javascript;",
            "SyntaxError"
        );
        assert!(result.is_ok(), "Expected SyntaxError but got: {:?}", result);
        Ok(())
    }
}

#[test]
fn test_bare_runtime_runtime_error() -> BareResult<()> {
    let instance = TestInstance::new()?;
    debug!("=== Starting runtime error test ===");

    unsafe {
        // Test runtime error
        let result = instance.run_script_expect_error(
            "throw new Error('Expected error');",
            "Error: Expected error"
        );
        assert!(result.is_ok(), "Expected Error but got: {:?}", result);
        Ok(())
    }
}

#[test]
fn test_bare_runtime_reference_error() -> BareResult<()> {
    let instance = TestInstance::new()?;
    debug!("=== Starting reference error test ===");

    unsafe {
        // Test reference error
        let result = instance.run_script_expect_error(
            "nonexistentFunction();",
            "ReferenceError"
        );
        assert!(result.is_ok(), "Expected ReferenceError but got: {:?}", result);
        Ok(())
    }
} 