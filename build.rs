use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let bare_build_dir = PathBuf::from(env::current_dir().unwrap()).join("bare/build");
    let bare_build_dir_str = bare_build_dir.to_str().unwrap();
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let target_dir = out_dir.ancestors().find(|p| p.ends_with("target")).unwrap();
    let profile = out_dir.ancestors().find(|p| p.ends_with("debug") || p.ends_with("release")).unwrap();

    // Link directories
    println!("cargo:rustc-link-search={}", bare_build_dir_str);
    
    // Add Homebrew lib path for macOS
    if cfg!(target_os = "macos") {
        // For Apple Silicon Macs
        println!("cargo:rustc-link-search=/opt/homebrew/lib");
        // For Intel Macs
        println!("cargo:rustc-link-search=/usr/local/lib");
        
        // Link libuv
        println!("cargo:rustc-link-lib=uv");
        
        // Use dynamic library instead of static
        println!("cargo:rustc-link-lib=bare");
        
        // Copy libbare.dylib to target directory
        let dylib_src = bare_build_dir.join("libbare.dylib");
        let dylib_dst = profile.join("libbare.dylib");
        fs::copy(&dylib_src, &dylib_dst).expect("Failed to copy libbare.dylib");
        
        // Add rpath for finding dependencies
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path");
        println!("cargo:rustc-link-arg=-Wl,-rpath,@executable_path/.");
    } else if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=static=bare");
        println!("cargo:rustc-link-lib=uv");
        println!("cargo:rustc-link-arg=/WHOLEARCHIVE:bare.lib");
    } else {
        // Linux
        println!("cargo:rustc-link-lib=uv");
        println!("cargo:rustc-link-arg=-Wl,--whole-archive");
        println!("cargo:rustc-link-arg={}/libbare.a", bare_build_dir_str);
        println!("cargo:rustc-link-arg=-Wl,--no-whole-archive");
    }

    // Create bindgen builder
    let mut builder = bindgen::Builder::default()
        .header("bare/include/bare.h")
        .clang_arg("-I./bare/include");

    // Add all deps include directories
    let deps_dir = bare_build_dir.join("_deps");
    if deps_dir.exists() {
        for entry in fs::read_dir(deps_dir).expect("Failed to read _deps directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();
            if path.is_dir() {
                let include_path = path.join("include");
                if include_path.exists() {
                    let include_arg = format!("-I{}", include_path.display());
                    println!("Adding include path: {}", include_arg);
                    builder = builder.clang_arg(include_arg);
                }
            }
        }
    }

    // Generate and write bindings
    let bindings = builder
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");
}