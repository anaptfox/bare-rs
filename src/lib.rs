pub mod bindings;

use std::ffi::NulError;
use std::fmt;
use std::ptr;
use std::mem;
use libc;
use std::sync::Mutex;

use bindings::*;

// Global runtime storage using lazy_static
lazy_static::lazy_static! {
    static ref RUNTIME: Mutex<Option<GlobalRuntime>> = Mutex::new(None);
}

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

pub struct GlobalRuntime {
    pub uv_loop: *mut uv_loop_t,
    pub platform: *mut js_platform_t,
}

// Mark GlobalRuntime as thread safe since we control access through Mutex
unsafe impl Send for GlobalRuntime {}
unsafe impl Sync for GlobalRuntime {}

/// Enhanced exception handler
pub unsafe fn handle_js_exception(env: *mut js_env_t) -> BareResult<()> {
    log::debug!("Checking for exception...");
    let mut has_exception = false;
    let check_result = js_is_exception_pending(env, &mut has_exception);
    log::debug!("Check result: {}, has_exception: {}", check_result, has_exception);

    if check_result != 0 {
        log::error!("Failed to check exception status");
        return Err(BareError::RuntimeError("Failed to check exception status".into()));
    }

    if !has_exception {
        log::debug!("No exception found");
        return Ok(());
    }

    log::debug!("Exception found, getting details...");
    // Get the exception object
    let mut error = ptr::null_mut();
    let clear_result = js_get_and_clear_last_exception(env, &mut error);
    log::debug!("Clear result: {}, error ptr: {:?}", clear_result, error);

    if clear_result != 0 {
        log::error!("Failed to get exception details");
        return Err(BareError::RuntimeError("Failed to get exception details".into()));
    }

    // Extract error details
    log::debug!("Getting error type...");
    let error_type = get_error_type(env, error)?;
    log::debug!("Getting error message...");
    let message = get_error_message(env, error)?;
    log::debug!("Getting error stack...");
    let stack = get_error_stack(env, error)?;

    log::error!("JavaScript error:");
    log::error!("  Type: {}", error_type);
    log::error!("  Message: {}", message);
    log::error!("  Stack: {}", stack);

    Err(BareError::JSError {
        error_type,
        message,
        stack: Some(stack),
    })
}

/// Helper functions for error details extraction
pub unsafe fn get_error_type(env: *mut js_env_t, error: *mut js_value_t) -> BareResult<String> {
    let mut constructor = ptr::null_mut();
    let mut str_len = 0;

    // Convert constructor name to string
    if js_get_value_string_utf8(env, constructor, ptr::null_mut(), 0, &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get constructor string length".into())); 
    }

    let mut buffer = vec![0u8; str_len as usize + 1];
    if js_get_value_string_utf8(env, constructor, buffer.as_mut_ptr() as *mut u8, buffer.len(), &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get constructor string".into()));
    }

    Ok(String::from_utf8_lossy(&buffer[..str_len as usize]).into_owned())
}

pub unsafe fn get_error_message(env: *mut js_env_t, error: *mut js_value_t) -> BareResult<String> {
    let mut message = ptr::null_mut();
    let mut str_len = 0;

    // Get message property
    if js_get_named_property(env, error, "message\0".as_ptr() as *const i8, &mut message) != 0 {
        return Err(BareError::RuntimeError("Failed to get error message".into()));
    }

    // Convert message to string
    if js_get_value_string_utf8(env, message, ptr::null_mut(), 0, &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get message string length".into()));
    }

    let mut buffer = vec![0u8; str_len as usize + 1];
    if js_get_value_string_utf8(env, message, buffer.as_mut_ptr() as *mut u8, buffer.len(), &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get message string".into()));
    }

    Ok(String::from_utf8_lossy(&buffer[..str_len as usize]).into_owned())
}

pub unsafe fn get_error_stack(env: *mut js_env_t, error: *mut js_value_t) -> BareResult<String> {
    let mut stack = ptr::null_mut();
    let mut str_len = 0;

    // Get stack property
    if js_get_named_property(env, error, "stack\0".as_ptr() as *const i8, &mut stack) != 0 {
        return Err(BareError::RuntimeError("Failed to get error stack".into()));
    }

    // Convert stack to string
    if js_get_value_string_utf8(env, stack, ptr::null_mut(), 0, &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get stack string length".into()));
    }

    let mut buffer = vec![0u8; str_len as usize + 1];
    if js_get_value_string_utf8(env, stack, buffer.as_mut_ptr() as *mut u8, buffer.len(), &mut str_len) != 0 {
        return Err(BareError::RuntimeError("Failed to get stack string".into()));
    }

    Ok(String::from_utf8_lossy(&buffer[..str_len as usize]).into_owned())
}

#[cfg(target_os = "macos")]
pub fn set_stack_size() -> BareResult<()> {
    // Only set stack size when running as main executable
    if std::env::args().next().map_or(false, |arg| arg.ends_with("bare-rs")) {
        unsafe {
            let mut attr: libc::pthread_attr_t = std::mem::zeroed();
            if libc::pthread_attr_init(&mut attr) != 0 {
                return Err(BareError::SetupError("Failed to init pthread attr".into()));
            }
            
            // Set stack size to 64MB
            if libc::pthread_attr_setstacksize(&mut attr, 64 * 1024 * 1024) != 0 {
                return Err(BareError::SetupError("Failed to set stack size".into()));
            }
            
            if libc::pthread_attr_destroy(&mut attr) != 0 {
                return Err(BareError::SetupError("Failed to destroy pthread attr".into()));
            }
        }
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn set_stack_size() -> BareResult<()> {
    Ok(())
}

pub unsafe fn init_runtime_once() -> BareResult<()> {
    let mut runtime = RUNTIME.lock().unwrap();
    if runtime.is_none() {
        // Initialize UV loop first
        let uv_loop = uv_loop_new();
        if uv_loop.is_null() {
            return Err(BareError::RuntimeError("Failed to create UV loop".into()));
        }

        // Initialize JS platform
        let mut platform = ptr::null_mut();
        let mut platform_options = js_platform_options_t {
            version: 1,
            expose_garbage_collection: false,
            trace_garbage_collection: false,
            disable_optimizing_compiler: false,
            trace_optimizations: false,
            trace_deoptimizations: false,
            enable_sampling_profiler: false,
            sampling_profiler_interval: 0,
            optimize_for_memory: true,
        };
        
        if js_create_platform(uv_loop, &mut platform_options, &mut platform) != 0 {
            uv_loop_delete(uv_loop);
            return Err(BareError::RuntimeError("Failed to create JS platform".into()));
        }

        *runtime = Some(GlobalRuntime {
            uv_loop,
            platform,
        });
    }
    Ok(())
}

pub unsafe fn get_runtime() -> BareResult<GlobalRuntime> {
    let runtime = RUNTIME.lock().unwrap();
    runtime.as_ref()
        .map(|r| GlobalRuntime { 
            uv_loop: r.uv_loop, 
            platform: r.platform 
        })
        .ok_or_else(|| BareError::RuntimeError("Runtime not initialized".into()))
} 