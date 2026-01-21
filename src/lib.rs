#![feature(downcast_unchecked)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(rustdoc::invalid_rust_codeblocks)]

//! # Voxel Engine
//!
//! A high-performance voxel rendering and simulation engine built with Rust and WGPU.
//!
//! This crate provides a complete framework for creating voxel-based games and applications,
//! with a focus on performance, modularity, and cross-platform compatibility (including WebAssembly).
//!
//! ## Key Modules
//!
//! * `application_state` - Manages the application lifecycle and window management
//! * `core` - Core utilities and data structures used throughout the engine
//! * `engine_state` - The main engine components including rendering, voxels, and task management
//!
//! ## Architecture
//!
//! The engine follows a modular architecture with clear separation between:
//! * Platform abstraction (supporting native and web targets)
//! * Rendering system (based on WGPU)
//! * Voxel data management and meshing
//! * Task scheduling and execution
//!
//! ## Usage
//!
//! ```rust
//! // Native application initialization
//! fn main() {
//!     voxel_engine::run();
//! }
//! ```
//!
//! For web applications:
//!
//! ```rust
//! // Called from JavaScript
//! #[wasm_bindgen]
//! pub fn start() {
//!     voxel_engine::run_web();
//! }
//! ```
//!
//! ## Performance Considerations
//!
//! The engine is designed with performance as a primary concern:
//! * Chunk-based voxel storage for efficient memory usage
//! * Optimized meshing algorithms for fast geometry generation
//! * Multi-threaded task execution for CPU-intensive operations
//! * Efficient GPU resource management

use application_state::{
    graphics_resources_builder::{GraphicsBuilder, MaybeGraphics},
    ApplicationState,
};
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::wasm_bindgen;

use winit::event_loop::EventLoop;

// use std::io::Write;

#[cfg(not(target_family = "wasm"))]
use log::info;

mod application_state;
mod core;
mod engine_state;

#[cfg(target_family = "wasm")]
const CANVAS_ID: &str = "wgpu-canvas";

pub const APPLICATION_INITIALIZATION_STOPWATCH: &str = "Application Initialization";

#[cfg(not(target_family = "wasm"))]
pub fn run() {
    let mut log_builder = env_logger::Builder::new();
    log_builder
        .target(env_logger::Target::Stdout)
        .parse_env("RUST_LOG")
        .init();

    // use std::fs::File;

    // use log::LevelFilter;

    // let target = Box::new(File::create("log.txt").expect("Can't create file"));

    // env_logger::Builder::new()
    //     .target(env_logger::Target::Pipe(target))
    //     .filter(None, LevelFilter::Error)
    //     .format(|buf, record| {
    //         writeln!(
    //             buf,
    //             "[{} {}:{}] {}",
    //             record.level(),
    //             record.file().unwrap_or("unknown"),
    //             record.line().unwrap_or(0),
    //             record.args()
    //         )
    //     })
    //     .init();

    info!("Logger initialized");
    let event_loop = EventLoop::with_user_event().build().unwrap();

    let mut state: ApplicationState = ApplicationState {
        graphics: MaybeGraphics::Builder(GraphicsBuilder::new(event_loop.create_proxy())),
        state: None,
        web_window_size: None,
    };

    let _ = event_loop.run_app(&mut state);
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub fn run_web() {
    use winit::platform::web::EventLoopExtWebSys;

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");

    let event_loop = EventLoop::with_user_event().build().unwrap();

    let state: ApplicationState = ApplicationState {
        graphics: MaybeGraphics::Builder(GraphicsBuilder::new(event_loop.create_proxy())),
        state: None,
        web_window_size: None,
    };

    let _ = event_loop.spawn_app(state);
}
