use std::ffi::NulError;
use std::fmt;

/// Custom error type for bare-rs
#[derive(Debug)]
pub enum BareError {
    // System level errors
    RuntimeError(String),
    SetupError(String),
    
    // JavaScript errors
    JSError {
        error_type: String,
        message: String,
        stack: Option<String>,
    },
    
    // Resource errors
    MemoryError(String),
    ResourceExhausted(String),
}

impl fmt::Display for BareError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BareError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            BareError::SetupError(msg) => write!(f, "Setup error: {}", msg),
            BareError::JSError { error_type, message, stack } => {
                if let Some(stack_trace) = stack {
                    write!(f, "{}: {}\nStack trace:\n{}", error_type, message, stack_trace)
                } else {
                    write!(f, "{}: {}", error_type, message)
                }
            },
            BareError::MemoryError(msg) => write!(f, "Memory error: {}", msg),
            BareError::ResourceExhausted(msg) => write!(f, "Resource exhausted: {}", msg),
        }
    }
}

impl std::error::Error for BareError {}

// Add conversion from NulError to BareError
impl From<NulError> for BareError {
    fn from(error: NulError) -> Self {
        BareError::RuntimeError(format!("String contains null byte: {}", error))
    }
}

pub type BareResult<T> = Result<T, BareError>; 