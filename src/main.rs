use bare_rs::{BareResult, BareError, init_runtime_once, get_runtime, set_stack_size, handle_js_exception};
use bare_rs::bindings::*;
use std::ffi::CString;
use std::ptr;
use log::{info, debug, error};
use env_logger::Env;
use std::env;

fn main() -> BareResult<()> {
    // Initialize logger with INFO level by default, can be overridden with RUST_LOG env var
    env_logger::Builder::from_env(Env::default().default_filter_or("error"))
        .init();
    
    info!("Starting Bare-rs...");
    
    // Set larger stack size
    debug!("Setting stack size...");
    set_stack_size()?;
    debug!("Stack size set successfully");
    
    unsafe {
        // Initialize global runtime
        debug!("Initializing runtime...");
        init_runtime_once()?;
        let runtime = get_runtime()?;
        debug!("Runtime initialized successfully");

        // Initialize bare options with sane defaults
        debug!("Initializing Bare options...");
        let options = bare_options_t {
            version: 0, // Current version
            memory_limit: 1024 * 1024 * 1024, // 1GB memory limit
        };
        debug!("Bare options initialized with version {} and memory_limit {} MB", 
            options.version, options.memory_limit / (1024 * 1024));

        // Setup bare runtime with defaults
        debug!("Setting up Bare runtime...");
        let mut bare = ptr::null_mut();
        let mut env = ptr::null_mut();
        
        // Default empty args
        let args = vec![CString::new("bare-rs").unwrap()];
        let mut c_args: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();
        
        debug!("Calling bare_setup...");
        let setup_result = bare_setup(
            runtime.uv_loop,
            runtime.platform,
            &mut env,
            c_args.len() as i32,
            c_args.as_mut_ptr(),
            &options,
            &mut bare,
        );
        debug!("bare_setup returned: {}", setup_result);
        
        if setup_result != 0 {
            return Err(BareError::SetupError("Failed to setup Bare runtime".into()));
        }
        debug!("Bare runtime setup successfully");

        // Get command line args
        let args: Vec<String> = env::args().collect();
        
        if args.len() <= 1 {
            return Err(BareError::RuntimeError("No script file provided. Usage: bare-rs <script_path>".into()));
        }

        // Load script from file
        debug!("Loading script from file: {}", args[1]);
        let file_script = std::fs::read_to_string(&args[1])
            .map_err(|e| BareError::RuntimeError(format!("Failed to read script file: {}", e)))?;
        let script = CString::new(file_script)?;
        let filename = CString::new(args[1].clone())?;

        let source = uv_buf_t {
            base: script.as_ptr() as *mut i8,
            len: script.as_bytes().len(),
        };

        debug!("Loading script...");
        let mut result = ptr::null_mut();
        let load_result = bare_load(bare, filename.as_ptr(), &source, &mut result);
        debug!("bare_load returned: {}", load_result);
        
        if load_result != 0 {
            return Err(BareError::RuntimeError("Failed to load script".into()));
        }
        debug!("Script loaded successfully");

        debug!("Running script...");
        let run_result = bare_run(bare);
        debug!("bare_run() result: {}", run_result);
        
        // Check for exceptions
        if let Err(e) = handle_js_exception(env) {
            error!("JavaScript error: {}", e);
            
            // Cleanup before returning error
            let mut exit_code = 1;
            let _ = bare_teardown(bare, &mut exit_code);
            debug!("Teardown after error completed with exit code {}", exit_code);
            
            return Err(e);
        }

        // Cleanup
        debug!("Starting cleanup...");
        let mut exit_code = 0;
        
        debug!("Tearing down Bare runtime...");
        let teardown_result = bare_teardown(bare, &mut exit_code);
        debug!("bare_teardown returned: {} with exit_code: {}", teardown_result, exit_code);
        
        if teardown_result != 0 {
            return Err(BareError::RuntimeError("Failed to teardown Bare runtime".into()));
        }
        debug!("Bare runtime torn down successfully");

        info!("Bare-rs completed successfully");
        Ok(())
    }
}