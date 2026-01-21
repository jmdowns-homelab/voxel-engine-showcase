//! # Buffer State Module
//!
//! This module provides a centralized system for managing GPU buffers in the voxel engine.
//! It handles buffer creation, writing, mapping, and analytics to ensure efficient GPU memory usage.
//!
//! ## Key Features
//!
//! * Centralized buffer management with named references
//! * Buffer usage analytics and memory tracking
//! * Safe buffer writing with bounds checking
//! * Support for asynchronous buffer mapping
//!
//! ## Architecture
//!
//! The `BufferState` struct serves as a registry for all GPU buffers used by the engine.
//! Buffers are referenced by name (static string) and can be created, written to, and
//! mapped asynchronously. The module also provides analytics about buffer usage to help
//! optimize memory consumption.
//!
//! ## Performance Considerations
//!
//! * Minimizes redundant buffer allocations through centralized management
//! * Tracks buffer usage to identify optimization opportunities
//! * Provides safe abstractions for buffer operations while maintaining performance

use std::collections::HashMap;

use bytemuck::NoUninit;
use wgpu::{
    util::{DeviceExt, DrawIndexedIndirectArgs},
    Buffer, BufferAsyncError, Device, MapMode, Queue, WasmNotSend,
};

use crate::core::{StResource, StSystem};
use std::fmt::Debug;

/// Analytics data for a GPU buffer
///
/// Tracks memory allocation, usage, and write operations for a buffer
/// to help identify optimization opportunities.
#[derive(Debug)]
struct BufferAnalytics {
    /// Total memory allocated for the buffer in bytes
    pub allocated_memory: u64,
    /// Actual memory used in the buffer in bytes (based on writes)
    pub used_memory: u64,
    /// Number of times the buffer has been written to
    pub times_written: u64,
}

/// Central manager for GPU buffers in the voxel engine
///
/// Provides a registry for creating, accessing, and writing to GPU buffers.
/// Buffers are referenced by name (static string) and their usage is tracked
/// for optimization purposes.
///
/// # Examples
///
/// ```
/// let buffer_state = BufferState::new(device, queue);
///
/// // Create a vertex buffer
/// buffer_state.create_buffer_init(
///     "vertex_buffer",
///     wgpu::util::BufferInitDescriptor {
///         label: Some("Vertex Buffer"),
///         contents: bytemuck::cast_slice(&vertices),
///         usage: wgpu::BufferUsages::VERTEX,
///     },
/// );
///
/// // Later, access the buffer
/// let vertex_buffer = buffer_state.get_buffer("vertex_buffer");
/// ```
pub struct BufferState {
    /// Reference to the GPU device
    pub device: StSystem<Device>,
    /// Reference to the GPU command queue
    pub queue: StSystem<Queue>,
    /// Map of buffer names to buffer objects
    pub buffers: HashMap<&'static str, Buffer>,
    /// Analytics data for each buffer
    buffer_analytics: StResource<HashMap<&'static str, BufferAnalytics>>,
}

impl BufferState {
    /// Creates a new buffer state manager
    ///
    /// # Arguments
    ///
    /// * `device` - Reference to the GPU device
    /// * `queue` - Reference to the GPU command queue
    ///
    /// # Returns
    ///
    /// A new `BufferState` instance with empty buffer collections
    pub fn new(device: StSystem<Device>, queue: StSystem<Queue>) -> Self {
        Self {
            device,
            queue,
            buffers: HashMap::new(),
            buffer_analytics: StResource::new(HashMap::new()),
        }
    }

    /// Creates an empty buffer with the specified descriptor
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Unique name for the buffer
    /// * `buffer_descriptor` - Buffer configuration descriptor
    ///
    /// # Examples
    ///
    /// ```
    /// buffer_state.create_buffer(
    ///     "storage_buffer",
    ///     wgpu::BufferDescriptor {
    ///         label: Some("Storage Buffer"),
    ///         size: 1024,
    ///         usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    ///         mapped_at_creation: false,
    ///     },
    /// );
    /// ```
    pub fn create_buffer(
        &mut self,
        buffer_name: &'static str,
        buffer_descriptor: wgpu::BufferDescriptor,
    ) {
        let buffer_analytics = BufferAnalytics {
            allocated_memory: buffer_descriptor.size,
            used_memory: 0,
            times_written: 0,
        };
        let buffer = self.device.get().create_buffer(&buffer_descriptor);

        self.buffers.insert(buffer_name, buffer);
        self.buffer_analytics
            .get_mut()
            .insert(buffer_name, buffer_analytics);
    }

    /// Creates a buffer and initializes it with data
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Unique name for the buffer
    /// * `init_descriptor` - Buffer initialization descriptor with data
    ///
    /// # Examples
    ///
    /// ```
    /// buffer_state.create_buffer_init(
    ///     "vertex_buffer",
    ///     wgpu::util::BufferInitDescriptor {
    ///         label: Some("Vertex Buffer"),
    ///         contents: bytemuck::cast_slice(&vertices),
    ///         usage: wgpu::BufferUsages::VERTEX,
    ///     },
    /// );
    /// ```
    pub fn create_buffer_init(
        &mut self,
        buffer_name: &'static str,
        init_descriptor: wgpu::util::BufferInitDescriptor,
    ) {
        let buffer_analytics = BufferAnalytics {
            allocated_memory: init_descriptor.contents.len() as u64,
            used_memory: init_descriptor.contents.len() as u64,
            times_written: 1,
        };
        let buffer = self.device.get().create_buffer_init(&init_descriptor);

        self.buffers.insert(buffer_name, buffer);
        self.buffer_analytics
            .get_mut()
            .insert(buffer_name, buffer_analytics);
    }

    /// Writes data to a buffer using a command structure
    ///
    /// # Arguments
    ///
    /// * `buffer_command` - Command containing buffer name, offset, and data
    ///
    /// # Panics
    ///
    /// Panics if the buffer does not exist or if the write would exceed buffer bounds
    pub fn write(&self, buffer_command: BufferWriteCommand) {
        self.write_buffer(
            buffer_command.buffer_name,
            buffer_command.offset,
            buffer_command.data.as_bytes(),
        );
    }

    /// Writes raw byte data to a buffer
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Name of the buffer to write to
    /// * `offset` - Byte offset in the buffer to start writing
    /// * `data` - Raw byte data to write
    ///
    /// # Panics
    ///
    /// Panics if the buffer does not exist or if the write would exceed buffer bounds
    ///
    /// # Examples
    ///
    /// ```
    /// buffer_state.write_buffer(
    ///     "uniform_buffer",
    ///     0,
    ///     bytemuck::cast_slice(&[my_uniform_data]),
    /// );
    /// ```
    pub fn write_buffer(
        &self,
        buffer_name: &'static str,
        offset: wgpu::BufferAddress,
        data: &[u8],
    ) {
        let buffer = self.buffers.get(buffer_name).unwrap();
        let mut buffer_dictionary = self.buffer_analytics.get_mut();
        let buffer_analytics = buffer_dictionary.get_mut(buffer_name).unwrap();

        let buffer_size = buffer_analytics.allocated_memory;
        let data_size = data.len() as u64;

        if offset + data_size > buffer_size {
            panic!(
                "Buffer write out of bounds for buffer name '{}'",
                buffer_name
            );
        }

        let queue = self.queue.get();
        queue.write_buffer(buffer, offset, data);
        buffer_analytics.used_memory = buffer_analytics.used_memory.max(offset + data_size);
        buffer_analytics.times_written += 1;
    }

    /// Maps a buffer asynchronously for CPU access
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Name of the buffer to map
    /// * `mode` - Mapping mode (read or write)
    /// * `callback` - Function to call when mapping is complete
    ///
    /// # Panics
    ///
    /// Panics if the buffer does not exist
    pub fn map_async(
        &self,
        buffer_name: &'static str,
        mode: MapMode,
        callback: impl FnOnce(Result<(), BufferAsyncError>) + WasmNotSend + 'static,
    ) {
        self.buffers
            .get(buffer_name)
            .unwrap()
            .slice(..)
            .map_async(mode, callback)
    }

    /// Gets a reference to a buffer by name
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Name of the buffer to retrieve
    ///
    /// # Returns
    ///
    /// Reference to the requested buffer
    ///
    /// # Panics
    ///
    /// Panics if the buffer does not exist
    pub fn get_buffer(&self, buffer_name: &'static str) -> &Buffer {
        self.buffers.get(buffer_name).unwrap()
    }

    /// Gets a binding resource for the entire buffer
    ///
    /// # Arguments
    ///
    /// * `buffer_name` - Name of the buffer to get binding for
    ///
    /// # Returns
    ///
    /// A binding resource for the entire buffer
    ///
    /// # Panics
    ///
    /// Panics if the buffer does not exist
    pub fn get_entire_binding(&self, buffer_name: &'static str) -> wgpu::BindingResource {
        let buffer = self.buffers.get(buffer_name).unwrap();
        buffer.as_entire_binding()
    }

    /// Gets the total allocated memory across all buffers
    ///
    /// # Returns
    ///
    /// Total allocated memory in bytes
    pub fn get_total_allocated_memory(&self) -> u64 {
        self.buffer_analytics
            .get()
            .iter()
            .fold(0, |acc, (_, buffer_analytics)| {
                acc + buffer_analytics.allocated_memory
            })
    }

    /// Gets the total used memory across all buffers
    ///
    /// # Returns
    ///
    /// Total used memory in bytes
    pub fn get_total_used_memory(&self) -> u64 {
        self.buffer_analytics
            .get()
            .iter()
            .fold(0, |acc, (_, buffer_analytics)| {
                acc + buffer_analytics.used_memory
            })
    }
}

/// Command for writing data to a buffer
///
/// This structure encapsulates all information needed to write data to a buffer,
/// including the buffer name, offset, and the data itself.
pub struct BufferWriteCommand {
    /// Descriptive name for the command (for debugging)
    pub name: String,
    /// Name of the target buffer
    pub buffer_name: &'static str,
    /// Byte offset in the buffer to start writing
    pub offset: u64,
    /// Data to write to the buffer
    pub data: Box<dyn AsBytes + Send + Sync>,
}

impl Debug for BufferWriteCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BufferWriteCommand")
            .field("name", &self.name)
            .field("buffer_name", &self.buffer_name)
            .field("offset", &self.offset)
            .finish()
    }
}

/// Trait for types that can be converted to bytes for buffer writing
///
/// This trait is implemented for common types that can be safely converted to
/// raw bytes for GPU buffer operations.
pub trait AsBytes {
    /// Converts the value to a byte slice
    ///
    /// # Returns
    ///
    /// A slice of bytes representing the value
    fn as_bytes(&self) -> &[u8];
}

impl<T> AsBytes for Vec<T>
where
    T: NoUninit + Send + Sync,
{
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

impl<T, const N: usize> AsBytes for [T; N]
where
    T: NoUninit + Send + Sync,
{
    fn as_bytes(&self) -> &[u8] {
        bytemuck::cast_slice(self)
    }
}

impl AsBytes for DrawIndexedIndirectArgs {
    fn as_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}
