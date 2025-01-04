# Bare-rs

Rust bindings for [bare](https://github.com/holepunchto/bare) - a small and modular JavaScript runtime for desktop and mobile. This project provides safe Rust bindings and a testing framework for the Bare runtime.

## Features

- Safe Rust bindings for the Bare JavaScript runtime
- Comprehensive error handling and type conversions
- Thread-safe runtime management
- Testing framework for JavaScript execution
- Support for async operations and events
- Memory management and garbage collection controls

## Prerequisites

Before building, ensure you have:

- Rust toolchain installed (via [rustup](https://rustup.rs/))
- Node.js and npm installed
- C compiler (gcc, clang, or MSVC)
- On Windows: Visual Studio Build Tools
- bare-make installed globally:
```sh
npm install -g bare-make
```

## Building

1. Clone the repository with submodules:
```sh 
git clone --recursive https://github.com/yourusername/bare-rs.git
cd bare-rs
```

If you've already cloned the repository:
```sh
git submodule update --init --recursive
```

2. Build the project:
```sh
cargo build
```

## Usage

### Basic Example

```rust
use bare_rs::{BareResult, init_runtime_once, get_runtime, set_stack_size};
use std::ffi::CString;

fn main() -> BareResult<()> {
    // Initialize runtime
    unsafe {
        set_stack_size()?;
        init_runtime_once()?;
        let runtime = get_runtime()?;

        // Create a new Bare instance
        let instance = TestInstance::new()?;

        // Run JavaScript code
        instance.run_script(r#"
            console.log('Hello from Bare-rs!');
            
            // Use async operations
            setTimeout(() => {
                console.log('Async operation complete');
                Bare.exit(0);
            }, 1000);
        "#)?;
    }
    Ok(())
}
```

### Error Handling

The library provides comprehensive error handling:

```rust
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
```

### Testing

The project includes a testing framework for JavaScript code:

```rust
#[test]
fn test_bare_runtime() -> BareResult<()> {
    let instance = TestInstance::new()?;
    
    unsafe {
        // Test JavaScript execution
        instance.run_script(r#"
            let x = 1 + 1;
            if(x !== 2) throw new Error('Math is broken!');
        "#)?;
    }
    
    Ok(())
}
```

## Project Structure

```
bare-rs/
├── .cargo/
│   └── config.toml      # Cargo configuration
├── bare/                # bare submodule
├── src/
│   ├── lib.rs          # Core library implementation
│   ├── bindings.rs     # Generated Bare bindings
│   └── main.rs         # CLI entry point
├── tests/
│   ├── mod.rs          # Test organization
│   ├── runtime/        # Runtime tests
│   ├── errors/         # Error handling tests
│   └── common/         # Shared test utilities
├── build.rs            # Build configuration
├── Cargo.toml          # Rust dependencies and project config
└── README.md          
```

## Advanced Features

### Platform Configuration

The runtime can be configured with various options:

```rust
let platform_options = js_platform_options_t {
    version: 1,
    expose_garbage_collection: false,
    optimize_for_memory: true,
    // ... other options
};
```

### Event Handling

Support for Bare's event system:

```rust
// Register event handlers
bare_on_before_exit(bare, Some(before_exit_cb));
bare_on_exit(bare, Some(exit_cb));
bare_on_idle(bare, Some(idle_cb));
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [bare](https://github.com/holepunchto/bare) - The original JavaScript runtime
- The Rust and JavaScript communities for their invaluable resources and tools 