//! # Core Module
//! 
//! This module provides fundamental concurrency primitives and resource management types
//! used throughout the voxel engine. It includes thread-safe and single-threaded variants
//! of resource containers and systems.
//!
//! ## Key Components
//! - `MtResource`: Thread-safe reference-counted resource with read-write locking
//! - `MtSystem`: Thread-safe system container with type erasure support
//! - `StResource`: Single-threaded reference-counted resource with interior mutability
//! - `StSystem`: Single-threaded system container with type erasure support
//! - `MtInjectionSystem`: Thread-safe dependency injection container
//! - `StInjectionSystem`: Single-threaded dependency injection container
//!
//! ## Usage
//! ```rust
//! use voxel_engine_core::core::{
//!     MtResource, MtSystem, StResource, StSystem,
//!     injection_system::{MtInjectionSystem, StInjectionSystem}
//! };
//!
//! // Thread-safe resource
//! let counter = MtResource::new(0);
//! *counter.get_mut() += 1;
//! assert_eq!(*counter.get(), 1);
//!
//! // Single-threaded system
//! let system = StSystem::new(Box::new(42u32));
//! assert_eq!(**system.get(), 42);
//! 
//! // Dependency injection
//! let injector = MtInjectionSystem::new();
//! injector.insert(42u32);
//! let value: u32 = *injector.get::<u32>().unwrap().get();
//! assert_eq!(value, 42);
//! ```

pub mod injection_system;

// Sub-modules for each core type
pub mod mt_resource;
pub mod mt_system;
pub mod st_resource;
pub mod st_system;

// Re-export types for easier access
pub use mt_resource::MtResource;
pub use mt_system::MtSystem;
pub use st_resource::StResource;
pub use st_system::StSystem;
