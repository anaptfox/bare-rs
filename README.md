# Bare-rs

Rust bindings for [bare](https://github.com/holepunchto/bare) - a small and modular JavaScript runtime for desktop and mobile. This project provides safe Rust bindings and a testing framework for the Bare runtime.

## Prerequisites

Before building, ensure you have:

- Rust toolchain installed (via [rustup](https://rustup.rs/))
- Node.js and npm installed
- C compiler (gcc, clang, or MSVC)
- bare-make installed globally:```sh
npm install -g bare-make```

## Building

1. Clone the repository with submodules:```sh 
git clone --recursive https://github.com/yourusername/bare-rs.git
cd bare-rs```

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

See [examples/basic.rs](examples/basic.rs) for a basic example.

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

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Acknowledgments

- [bare](https://github.com/holepunchto/bare) - The original JavaScript runtime
- The Rust and JavaScript communities for their invaluable resources and tools 



