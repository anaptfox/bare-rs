use bare_rs::{BareResult, init_runtime_once, get_runtime, set_stack_size, handle_js_exception};
use bare_rs::bindings::*;
use std::ffi::CString;
use std::ptr;
use log::{info, debug, error};
use env_logger::Env;

// Callback for before exit event
unsafe extern "C" fn before_exit_cb(bare: *mut bare_t) {
    info!("Bare is about to exit...");
}

// Callback for exit event
unsafe extern "C" fn exit_cb(bare: *mut bare_t) {
    info!("Bare is exiting...");
}

// Callback for teardown event
unsafe extern "C" fn teardown_cb(bare: *mut bare_t) {
    info!("Bare is tearing down...");
}

// Callback for idle event
unsafe extern "C" fn idle_cb(bare: *mut bare_t) {
    debug!("Bare is idle...");
}

// Callback for suspend event
unsafe extern "C" fn suspend_cb(bare: *mut bare_t, linger: ::std::os::raw::c_int) {
    debug!("Bare is suspending with linger: {}", linger);
}

// Callback for resume event
unsafe extern "C" fn resume_cb(bare: *mut bare_t) {
    debug!("Bare is resuming...");
}

fn main() -> BareResult<()> {
    // Initialize logger with debug level to see all events
    env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
        .init();
    
    info!("Starting bare-rs example...");
    
    // Set stack size and initialize runtime
    set_stack_size()?;
    
    unsafe {
        // Initialize the runtime
        init_runtime_once()?;
        let runtime = get_runtime()?;
        
        // Setup bare runtime options
        let options = bare_options_t {
            version: 0,
            memory_limit: 512 * 1024 * 1024, // 512MB memory limit
        };

        // Initialize bare runtime
        let mut bare = ptr::null_mut();
        let mut env = ptr::null_mut();
        
        // Setup arguments
        let args = vec![
            CString::new("bare-rs-example").unwrap(),
            CString::new("--example").unwrap(),
            CString::new("basic").unwrap(),
        ];
        let mut c_args: Vec<_> = args.iter().map(|s| s.as_ptr()).collect();
        
        debug!("Setting up Bare runtime...");
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
            return Err(bare_rs::BareError::SetupError("Failed to setup Bare runtime".into()));
        }

        // Register all available event handlers
        debug!("Registering event handlers...");
        bare_on_before_exit(bare, Some(before_exit_cb));
        bare_on_exit(bare, Some(exit_cb));
        bare_on_teardown(bare, Some(teardown_cb));
        bare_on_idle(bare, Some(idle_cb));
        bare_on_suspend(bare, Some(suspend_cb));
        bare_on_resume(bare, Some(resume_cb));
        
        // Example JavaScript code that demonstrates various features
        let script = CString::new(r#"
            // Basic console output
            console.log('Hello from bare-rs example!');
            
            // Demonstrate JSON handling
            const data = { message: 'Hello', count: 42 };
            console.log('JSON data:', JSON.stringify(data));
            
            // Basic arithmetic
            const result = 10 + 32;
            console.log('Math result:', result);
            
            // Error handling example
            Bare.on('uncaughtException', (err) => {
                console.error('Uncaught exception:', err);
                Bare.exit(1);
            });

            // Test error handling
            try {
                // Intentionally cause an error
                throw new Error('Test error handling');
            } catch (err) {
                console.log('Caught error:', err.message);
            }

            // Demonstrate Bare events
            Bare.on('beforeExit', () => {
                console.log('Bare: beforeExit event fired');
            });

            Bare.on('exit', (code) => {
                console.log('Bare: exit event fired with code:', code);
            });

            Bare.on('idle', () => {
                console.log('Bare: idle event fired');
            });

            // Using setTimeout for async operations
            console.log('Starting async operations...');
            
            let counter = 0;
            const timer = setInterval(() => {
                console.log('Timer tick:', counter);
                counter++;
                
                if (counter >= 3) {
                    clearInterval(timer);
                    console.log('Timer complete, exiting...');
                    Bare.exit(0);
                }
            }, 500);
        "#).unwrap();
        
        let source = uv_buf_t {
            base: script.as_ptr() as *mut i8,
            len: script.as_bytes().len(),
        };
        
        let filename = CString::new("example.js").unwrap();
        let mut result = ptr::null_mut();
        
        // Load and run the script
        debug!("Loading script...");
        let load_result = bare_load(bare, filename.as_ptr(), &source, &mut result);
        if load_result != 0 {
            error!("Failed to load script");
            let _ = bare_teardown(bare, &mut 1);  // Attempt cleanup
            return Err(bare_rs::BareError::RuntimeError("Failed to load script".into()));
        }
        
        // Run the script and event loop
        debug!("Running script and event loop...");
        let run_result = bare_run(bare);
        if run_result != 0 {
            // Check for any JavaScript exceptions
            if let Err(e) = handle_js_exception(env) {
                error!("JavaScript error occurred: {}", e);
                let _ = bare_teardown(bare, &mut 1);  // Attempt cleanup
                return Err(e);
            }
            let _ = bare_teardown(bare, &mut 1);  // Attempt cleanup
            return Err(bare_rs::BareError::RuntimeError("Failed to run script".into()));
        }
        
        // Cleanup
        debug!("Cleaning up...");
        let mut exit_code = 0;
        let teardown_result = bare_teardown(bare, &mut exit_code);
        if teardown_result != 0 {
            error!("Failed to teardown Bare runtime");
            return Err(bare_rs::BareError::RuntimeError("Failed to teardown Bare runtime".into()));
        }
        
        info!("Example completed successfully with exit code: {}", exit_code);
        Ok(())
    }
} 