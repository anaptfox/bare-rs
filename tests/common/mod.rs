use std::ffi::CString;
use std::ptr;
use std::sync::Mutex;
use bare_rs::{BareResult, BareError, init_runtime_once, get_runtime};
use bare_rs::bindings::*;

// Global mutex for test synchronization
lazy_static::lazy_static! {
    static ref TEST_MUTEX: Mutex<()> = Mutex::new(());
}

pub struct TestInstance {
    pub bare: *mut bare_t,
    pub env: *mut js_env_t,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl TestInstance {
    pub fn new() -> BareResult<Self> {
        // Acquire mutex to prevent parallel test execution
        let guard = TEST_MUTEX.lock().unwrap();

        unsafe {
            // Initialize global runtime if needed
            init_runtime_once()?;
            
            // Get runtime reference
            let runtime = get_runtime()?;

            // Initialize bare runtime
            let options = bare_options_t {
                version: 0,
                memory_limit: 1024 * 1024 * 1024,
            };

            let mut bare = ptr::null_mut();
            let mut env = ptr::null_mut();
            let args = vec![CString::new("test").unwrap()];
            let mut c_args: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();

            let setup_result = bare_setup(
                runtime.uv_loop,
                runtime.platform,
                &mut env,
                c_args.len() as i32,
                c_args.as_mut_ptr(),
                &options,
                &mut bare,
            );

            if setup_result != 0 {
                return Err(BareError::SetupError("Bare setup failed".into()));
            }

            Ok(TestInstance {
                bare,
                env,
                _guard: guard,
            })
        }
    }

    // Helper to run JavaScript code and expect success
    pub unsafe fn run_script(&self, code: &str) -> BareResult<()> {
        let script = CString::new(code).unwrap();
        let len = script.as_bytes().len();
        let source = uv_buf_t {
            base: script.as_ptr() as *mut i8,
            len,
        };
        let filename = CString::new("test.js").unwrap();
        let mut result = ptr::null_mut();

        // Load the script
        let load_result = bare_load(self.bare, filename.as_ptr(), &source, &mut result);
        if load_result != 0 {
            return Err(BareError::RuntimeError("Failed to load script".into()));
        }

        // Run the script
        let run_result = bare_run(self.bare);
        if run_result != 0 {
            return Err(BareError::RuntimeError("Failed to run script".into()));
        }

        // Check for exceptions
        bare_rs::handle_js_exception(self.env)
    }

    // Helper to run JavaScript code and expect an error
    pub unsafe fn run_script_expect_error(&self, code: &str, expected_error: &str) -> BareResult<()> {
        let script = CString::new(code).unwrap();
        let len = script.as_bytes().len();
        let source = uv_buf_t {
            base: script.as_ptr() as *mut i8,
            len,
        };
        let filename = CString::new("test.js").unwrap();
        let mut result = ptr::null_mut();

        // Load and run the script
        let load_result = bare_load(self.bare, filename.as_ptr(), &source, &mut result);
        if load_result != 0 {
            return Err(BareError::RuntimeError("Failed to load script".into()));
        }

        let run_result = bare_run(self.bare);
        if run_result == 0 {
            return Err(BareError::RuntimeError("Expected script to fail".into()));
        }

        // Check for the expected error
        match bare_rs::handle_js_exception(self.env) {
            Ok(_) => Err(BareError::RuntimeError("Expected error but got success".into())),
            Err(BareError::JSError { error_type, message, .. }) => {
                let error_text = format!("{}: {}", error_type, message);
                if error_text.contains(expected_error) {
                    Ok(())
                } else {
                    Err(BareError::RuntimeError(format!(
                        "Expected error '{}' but got '{}'",
                        expected_error, error_text
                    )))
                }
            }
            Err(e) => Err(e),
        }
    }
}

impl Drop for TestInstance {
    fn drop(&mut self) {
        unsafe {
            let mut exit_code = 0;
            bare_teardown(self.bare, &mut exit_code);
        }
    }
} 