use bare_rs::{BareResult, set_stack_size};
use bare_rs::bindings::{bare_t, bare_on_before_exit, bare_on_exit, bare_on_idle};
use super::common::TestInstance;
use log::debug;

// Test callbacks
unsafe extern "C" fn test_before_exit_cb(_bare: *mut bare_t) {
    debug!("Test: beforeExit event fired");
}

unsafe extern "C" fn test_exit_cb(_bare: *mut bare_t) {
    debug!("Test: exit event fired");
}

unsafe extern "C" fn test_idle_cb(_bare: *mut bare_t) {
    debug!("Test: idle event fired");
}

#[test]
fn test_bare_runtime_basic() -> BareResult<()> {
    // Set larger stack size first
    set_stack_size()?;

    let instance = TestInstance::new()?;

    unsafe {
        // Register event handlers
        bare_on_before_exit(instance.bare, Some(test_before_exit_cb));
        bare_on_exit(instance.bare, Some(test_exit_cb));
        bare_on_idle(instance.bare, Some(test_idle_cb));

        // Test basic arithmetic and console output
        instance.run_script(r#"
            console.log('Running basic arithmetic test...');
            let x = 1 + 1;
            if(x !== 2) throw new Error('Math is broken!');
            console.log('Basic arithmetic test passed');
            Bare.exit(0);
        "#)?;
    }

    Ok(())
}

#[test]
fn test_bare_runtime_json() -> BareResult<()> {
    let instance = TestInstance::new()?;

    unsafe {
        // Test JSON handling
        instance.run_script(r#"
            const data = { test: 'value', number: 42 };
            const str = JSON.stringify(data);
            const parsed = JSON.parse(str);
            if (parsed.test !== 'value' || parsed.number !== 42) {
                throw new Error('JSON handling failed');
            }
            Bare.exit(0);
        "#)?;
    }

    Ok(())
}

#[test]
fn test_bare_runtime_error_handling() -> BareResult<()> {
    let instance = TestInstance::new()?;

    unsafe {
        // Test error handling
        instance.run_script_expect_error(
            r#"throw new Error('Expected test error');"#,
            "Expected test error"
        )?;

        // Test try-catch handling
        instance.run_script(r#"
            try {
                throw new Error('Caught error');
            } catch (err) {
                if (err.message !== 'Caught error') {
                    throw new Error('Error message mismatch');
                }
            }
            Bare.exit(0);
        "#)?;
    }

    Ok(())
}

#[test]
fn test_bare_runtime_events() -> BareResult<()> {
    let instance = TestInstance::new()?;

    unsafe {
        // Test Bare events
        instance.run_script(r#"
            let eventsFired = {
                beforeExit: false,
                exit: false,
                idle: false
            };

            Bare.on('beforeExit', () => {
                console.log('Bare beforeExit event received');
                eventsFired.beforeExit = true;
            });

            Bare.on('exit', () => {
                console.log('Bare exit event received');
                eventsFired.exit = true;
            });

            Bare.on('idle', () => {
                console.log('Bare idle event received');
                eventsFired.idle = true;
            });

            // Small delay to allow events to fire
            setTimeout(() => {
                if (!eventsFired.idle) {
                    throw new Error('Idle event not fired');
                }
                console.log('Events test completed');
                Bare.exit(0);
            }, 100);
        "#)?;
    }

    Ok(())
}

#[test]
fn test_bare_runtime_async() -> BareResult<()> {
    let instance = TestInstance::new()?;

    unsafe {
        // Test async operations
        instance.run_script(r#"
            let counter = 0;
            const promise = new Promise((resolve) => {
                const timer = setInterval(() => {
                    counter++;
                    console.log('Timer tick:', counter);
                    if (counter >= 2) {
                        clearInterval(timer);
                        resolve(counter);
                    }
                }, 100);
            });

            promise.then((finalCount) => {
                if (finalCount !== 2) {
                    throw new Error('Async counter mismatch');
                }
                console.log('Async test completed');
                Bare.exit(0);
            });
        "#)?;
    }

    Ok(())
}

#[test]
fn test_bare_runtime_memory() -> BareResult<()> {
    let instance = TestInstance::new()?;

    unsafe {
        // Test memory operations
        instance.run_script(r#"
            // Allocate and manipulate a large array
            const array = new Array(1000).fill(0);
            array.forEach((_, i) => array[i] = i);
            
            // Test array sum
            const sum = array.reduce((a, b) => a + b, 0);
            const expected = (999 * 1000) / 2;  // Sum of 0 to 999
            
            if (sum !== expected) {
                throw new Error(`Memory test failed: sum ${sum} !== expected ${expected}`);
            }
            console.log('Memory test passed');
            Bare.exit(0);
        "#)?;
    }

    Ok(())
} 