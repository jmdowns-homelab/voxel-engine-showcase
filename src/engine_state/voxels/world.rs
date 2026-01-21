//! # World Module
//!
//! This module provides the `World` struct which manages a collection of chunks in the voxel world.
//! It serves as the central coordinator for chunk loading, unloading, and access.
//!
//! ## Architecture
//!
//! The world uses a sparse storage approach where only chunks that have been accessed or
//! modified are kept in memory. This allows for effectively infinite world sizes while
//! maintaining reasonable memory usage.
//!
//! ## Chunk Generation
//!
//! Multiple terrain generation strategies are supported:
//! - Perlin noise for natural-looking terrain
//! - Checkerboard pattern for testing
//! - Solid chunks (all blocks filled)
//! - Empty chunks (all blocks air)
//!
//! ## Performance Considerations
//!
//! - Chunks are stored in thread-safe containers to enable concurrent access
//! - Chunk lookup is O(1) using a hash map
//! - Only chunks near the player are typically loaded to conserve memory

use crate::core::MtResource;
use crate::engine_state::voxels::chunk::Chunk;
use cgmath::Point3;
use std::collections::HashMap;

/// Represents a voxel world composed of multiple chunks.
///
/// The world is stored as a sparse 3D grid of chunks, where each chunk is a 16x16x16
/// collection of blocks. Chunks are loaded on-demand as the player moves through the world.
///
/// # Examples
///
/// ```
/// let mut world = World::new();
/// 
/// // Add a chunk at position (0,0,0)
/// world.add_chunk_at(Point3::new(0, 0, 0));
/// 
/// // Retrieve the chunk
/// if let Some(chunk) = world.get_chunk_at(Point3::new(0, 0, 0)) {
///     // Use the chunk...
/// }
/// ```
pub struct World {
    /// A mapping from chunk coordinates to chunk data.
    /// Chunks are stored in a thread-safe reference-counted wrapper to allow
    /// shared access between systems.
    pub chunks: HashMap<Point3<i32>, MtResource<Chunk>>,
}

/// The method used to generate new chunks.
/// 
/// Possible values:
/// - "perlin": Uses Perlin noise for natural terrain generation
/// - "checkerboard": Alternates between solid and air blocks
/// - "solid": Generates completely solid chunks
/// - "empty": Generates completely empty chunks
const CHUNK_GENERATION_METHOD: &str = "perlin";

impl World {
    /// Creates a new, empty world.
    /// 
    /// # Returns
    /// 
    /// A new `World` instance with no chunks loaded.
    ///
    /// # Examples
    ///
    /// ```
    /// let world = World::new();
    /// assert_eq!(world.chunks.len(), 0);
    /// ```
    pub fn new() -> Self {
        World {
            chunks: HashMap::new(),
        }
    }

    /// Adds a new chunk at the specified chunk coordinates if one doesn't already exist.
    /// 
    /// The chunk is generated using the currently configured generation method.
    /// If a chunk already exists at the specified position, this method does nothing.
    /// 
    /// # Arguments
    /// 
    /// * `position` - The chunk coordinates where the new chunk should be added
    ///
    /// # Performance
    ///
    /// - Chunk generation can be computationally expensive, especially with Perlin noise
    /// - Consider using this method in a background thread for large batch operations
    ///
    /// # Examples
    ///
    /// ```
    /// let mut world = World::new();
    /// world.add_chunk_at(Point3::new(0, 0, 0));
    /// assert!(world.get_chunk_at(Point3::new(0, 0, 0)).is_some());
    /// ```
    pub fn add_chunk_at(&mut self, position: Point3<i32>) {
        if self.chunks.contains_key(&position) {
            return;
        }

        let chunk = match CHUNK_GENERATION_METHOD {
            "perlin" => Chunk::perlin(&position),
            "checkerboard" => Chunk::checkerboard(&position),
            "solid" => Chunk::solid(&position),
            "empty" => Chunk::empty(&position),
            _ => Chunk::empty(&position),
        };

        self.chunks.insert(position, MtResource::new(chunk));
    }

    /// Retrieves a reference to the chunk at the specified chunk coordinates.
    /// 
    /// # Arguments
    /// 
    /// * `pos` - The chunk coordinates to look up
    /// 
    /// # Returns
    /// 
    /// A clone of the `MtResource<Chunk>` if the chunk exists, or `None` if not.
    ///
    /// # Thread Safety
    ///
    /// The returned chunk is wrapped in a thread-safe reference-counted container,
    /// allowing it to be safely shared between systems and threads.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut world = World::new();
    /// world.add_chunk_at(Point3::new(0, 0, 0));
    /// 
    /// // Access the chunk
    /// if let Some(chunk) = world.get_chunk_at(Point3::new(0, 0, 0)) {
    ///     // The chunk can be safely shared between threads
    ///     let chunk_clone = chunk.clone();
    ///     
    ///     // Read-only access
    ///     let block = chunk.get().get_block_at(Point3::new(0, 0, 0));
    ///     
    ///     // Mutable access (requires exclusive access)
    ///     chunk.get_mut().set_block_at(Point3::new(0, 0, 0), some_block);
    /// }
    /// ```
    pub fn get_chunk_at(&self, pos: Point3<i32>) -> Option<MtResource<Chunk>> {
        self.chunks.get(&pos).cloned()
    }
}
