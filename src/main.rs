//! # Voxel Engine Application Entry Point
//!
//! This is the main entry point for the native application version of the voxel engine.
//! It simply calls into the library's `run()` function to initialize and start the engine.
//!
//! For web applications, see the `run_web()` function in the library.
//!
//! ## Usage
//!
//! ```bash
//! cargo run --release
//! ```

fn main() {
    #[cfg(not(target_family = "wasm"))]
    voxel_engine::run();
}
