//! # Voxel Engine Core
//!
//! This module contains the core voxel engine functionality, providing the foundation
//! for representing, manipulating, and rendering a voxel-based world.
//!
//! ## Architecture
//!
//! The voxel system is organized into several key components:
//!
//! * **Block**: Defines individual voxel types, properties, and behaviors
//! * **Chunk**: Manages fixed-size 3D arrays of blocks for efficient memory use and processing
//! * **World**: Coordinates chunks and provides a unified interface for the entire voxel space
//! * **Tasks**: Handles asynchronous operations like chunk generation and mesh creation
//!
//! ## Performance Considerations
//!
//! The voxel engine is designed with performance as a primary concern:
//!
//! * Chunks are loaded/unloaded dynamically based on player position
//! * Mesh generation is performed asynchronously to avoid frame rate drops
//! * Spatial data structures do not enable O(1) lookups, but are optimized for space efficiency
//! * Greedy meshing reduces vertex count for rendering efficiency
//! * Memory pooling minimizes allocations for frequently used objects
//!
//! ## Data Flow
//!
//! 1. World receives requests for block access or modification
//! 2. World delegates to appropriate chunk (loading if necessary)
//! 3. Changes trigger mesh regeneration tasks
//! 4. Completed meshes are sent to the renderer
//!
//! ## Thread Safety
//!
//! The voxel system is designed to be thread-safe:
//!
//! * Immutable chunk data can be accessed concurrently
//! * Mutable operations use proper synchronization
//! * Task system handles work distribution across threads

pub mod block;
pub mod chunk;
pub mod tasks;
pub mod world;
