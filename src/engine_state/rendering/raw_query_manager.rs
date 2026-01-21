//! Low-level GPU timing query implementation.
//!
//! This module provides functionality for measuring GPU execution time using timestamp queries.
//! It's used internally by `QueryManager` to provide high-level timing information.

use std::sync::{Arc, Mutex};
use std::time::Duration;

use wgpu::{Queue, RenderPassTimestampWrites};

use crate::{
    core::StSystem,
    engine_state::buffer_state::BufferState,
};

use super::query_manager::QueryResults;

/// Manages low-level GPU timing queries.
///
/// This struct handles the creation and management of WebGPU timestamp queries,
/// which are used to measure the execution time of GPU operations.
pub struct RawQueryManager {
    /// Manages the WebGPU query set and related resources
    timestamp_queries: Queries,
    /// Whether a new GPU query should be started
    should_query_gpu: bool,
    /// Tracks if a buffer mapping operation is in progress
    is_buffer_currently_mapping: Arc<Mutex<bool>>,
    /// Indicates if timestamp results are ready to be read
    are_timestamps_ready_to_read: Arc<Mutex<bool>>,
    /// Reference to the buffer state manager
    buffer_state: StSystem<BufferState>,
}

impl RawQueryManager {
    /// Creates a new `RawQueryManager` instance.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    ///
    /// # Returns
    /// A new `RawQueryManager` instance
    pub fn new(device: &wgpu::Device, buffer_state: StSystem<BufferState>) -> Self {
        buffer_state.get_mut().create_buffer(
            RESOLVE_BUFFER,
            wgpu::BufferDescriptor {
                label: Some("query resolve buffer"),
                size: size_of::<u64>() as u64 * RawQueryResults::NUM_QUERIES,
                usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::QUERY_RESOLVE,
                mapped_at_creation: false,
            },
        );
        buffer_state.get_mut().create_buffer(
            DESTINATION_BUFFER,
            wgpu::BufferDescriptor {
                label: Some("query dest buffer"),
                size: size_of::<u64>() as u64 * RawQueryResults::NUM_QUERIES,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            },
        );

        RawQueryManager {
            timestamp_queries: Queries::new(device, RawQueryResults::NUM_QUERIES),
            are_timestamps_ready_to_read: Arc::new(Mutex::new(false)),
            should_query_gpu: true,
            is_buffer_currently_mapping: Arc::new(Mutex::new(false)),
            buffer_state,
        }
    }

    /// Gets the timestamp writes configuration for a render pass.
    ///
    /// # Returns
    /// `Some(RenderPassTimestampWrites)` if timing is enabled, `None` otherwise
    pub fn request_timestamp_writes(&self) -> Option<RenderPassTimestampWrites> {
        if self.should_query_gpu {
            return Some(wgpu::RenderPassTimestampWrites {
                query_set: &self.timestamp_queries.set,
                beginning_of_pass_write_index: Some(0),
                end_of_pass_write_index: Some(1),
            });
        }

        None
    }

    /// Submits a request to resolve the timestamp queries.
    ///
    /// # Arguments
    /// * `encoder` - The command encoder to record the resolve operation
    pub fn request_gpu_query(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if self.should_query_gpu {
            self.timestamp_queries
                .resolve(encoder, &self.buffer_state.get());
            self.should_query_gpu = false;
        }
    }

    /// Attempts to read the timing results from the GPU.
    ///
    /// # Arguments
    /// * `queue` - The WebGPU queue for buffer operations
    ///
    /// # Returns
    /// `Some(QueryResults)` if results are available, `None` otherwise
    pub fn request_read_results(&mut self, queue: StSystem<Queue>) -> Option<QueryResults> {
        if *self.are_timestamps_ready_to_read.lock().unwrap() && !self.should_query_gpu {
            let timestamps = self
                .timestamp_queries
                .wait_for_results(&self.buffer_state.get());
            let duration =
                RawQueryResults::from_raw_results(timestamps).get_duration_in_millis(&queue.get());
            *self.are_timestamps_ready_to_read.lock().unwrap() = false;
            self.should_query_gpu = true;

            return Some(QueryResults {
                render_pipeline_duration: duration,
            });
        }

        if !*self.is_buffer_currently_mapping.lock().unwrap() && !self.should_query_gpu {
            *self.is_buffer_currently_mapping.lock().unwrap() = true;
            let buffer_mapping = self.is_buffer_currently_mapping.clone();
            let timestamps_ready = self.are_timestamps_ready_to_read.clone();
            self.buffer_state
                .get()
                .map_async(DESTINATION_BUFFER, wgpu::MapMode::Read, move |_| {
                    *timestamps_ready.lock().unwrap() = true;
                    *buffer_mapping.lock().unwrap() = false;
                });
        }

        None
    }
}

/// Manages a set of WebGPU timestamp queries.
///
/// Handles the creation and management of the WebGPU query set used for
/// timestamp queries, including resolving and reading back results.
struct Queries {
    /// The WebGPU query set containing the timestamp queries
    set: wgpu::QuerySet,
    /// Number of queries in the set
    num_queries: u64,
}

/// Name of the resolve buffer in the buffer state
const RESOLVE_BUFFER: &str = "resolve_buffer";
/// Name of the destination buffer in the buffer state
const DESTINATION_BUFFER: &str = "destination_buffer";

/// Represents the raw timing results from a GPU timestamp query.
struct RawQueryResults {
    /// Array containing the start and end timestamps in GPU ticks
    render_start_end_timestamps: [u64; 2],
}

impl RawQueryResults {
    /// Number of timestamp queries used (start and end)
    const NUM_QUERIES: u64 = 2;

    /// Creates a new `RawQueryResults` from raw timestamp values.
    ///
    /// # Arguments
    /// * `timestamps` - Vector containing the raw timestamp values
    ///
    /// # Returns
    /// A new `RawQueryResults` instance
    ///
    /// # Panics
    /// Panics if the input vector doesn't contain exactly 2 timestamps
    #[allow(clippy::redundant_closure)] // False positive
    fn from_raw_results(timestamps: Vec<u64>) -> Self {
        assert_eq!(timestamps.len(), Self::NUM_QUERIES as usize);

        let mut next_slot = 0;
        let mut get_next_slot = || {
            let slot = timestamps[next_slot];
            next_slot += 1;
            slot
        };
        let render_start_end_timestamps = [get_next_slot(), get_next_slot()];

        RawQueryResults {
            render_start_end_timestamps,
        }
    }

    /// Converts the raw timestamp difference to a `Duration` in microseconds.
    ///
    /// # Arguments
    /// * `queue` - The WebGPU queue used to get the timestamp period
    ///
    /// # Returns
    /// A `Duration` representing the time difference between start and end timestamps
    fn get_duration_in_millis(&self, queue: &wgpu::Queue) -> Duration {
        let period = queue.get_timestamp_period();
        let elapsed_us = |start, end: u64| end.wrapping_sub(start) as f64 * period as f64 / 1000.0;
        
        Duration::from_micros(elapsed_us(
            self.render_start_end_timestamps[0],
            self.render_start_end_timestamps[1],
        ) as u64)
    }
}

impl Queries {
    /// Creates a new `Queries` instance with the specified number of timestamp queries.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device used to create the query set
    /// * `num_queries` - Number of timestamp queries to allocate (typically 2 for start/end)
    ///
    /// # Returns
    /// A new `Queries` instance with an initialized WebGPU query set
    fn new(device: &wgpu::Device, num_queries: u64) -> Self {
        Queries {
            set: device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("Timestamp query set"),
                count: num_queries as _,
                ty: wgpu::QueryType::Timestamp,
            }),
            num_queries,
        }
    }

    /// Resolves timestamp queries into a buffer for CPU readback.
    ///
    /// This method records commands to resolve the timestamp queries into a buffer
    /// that can be read back by the CPU. It should be called after the commands being
    /// measured have been submitted to the GPU.
    ///
    /// # Arguments
    /// * `encoder` - The command encoder to record the resolve operation
    /// * `buffer_state` - The buffer state manager containing the resolve and destination buffers
    ///
    /// # Panics
    /// Panics if the required buffers are not found in the buffer state
    fn resolve(&self, encoder: &mut wgpu::CommandEncoder, buffer_state: &BufferState) {
        let destination_buffer = buffer_state.get_buffer(DESTINATION_BUFFER);
        let resolve_buffer = buffer_state.get_buffer(RESOLVE_BUFFER);
        
        // Resolve the query set into the resolve buffer
        // The range must not be larger than the number of valid queries in the set
        // See: https://github.com/gfx-rs/wgpu/issues/3993
        encoder.resolve_query_set(
            &self.set,
            0..(RawQueryResults::NUM_QUERIES as u32),
            resolve_buffer,
            0,
        );
        
        // Copy the resolved timestamps to a CPU-accessible buffer
        encoder.copy_buffer_to_buffer(
            resolve_buffer,
            0,  // Source offset
            destination_buffer,
            0,  // Destination offset
            resolve_buffer.size(),
        );
    }

    /// Waits for the timestamp query results to be available and returns them.
    ///
    /// This method maps the destination buffer and reads back the timestamp values.
    /// It blocks until the GPU has finished writing the results.
    ///
    /// # Arguments
    /// * `buffer_state` - The buffer state manager containing the destination buffer
    ///
    /// # Returns
    /// A vector containing the raw timestamp values in the order they were written
    ///
    /// # Panics
    /// Panics if the destination buffer cannot be mapped or read
    fn wait_for_results(&self, buffer_state: &BufferState) -> Vec<u64> {
        let destination_buffer = buffer_state.get_buffer(DESTINATION_BUFFER);
        
        // Map the buffer and read the timestamp values
        let timestamps = {
            // Calculate the range of the buffer containing the timestamp data
            let buffer_range = ..(size_of::<u64>() as wgpu::BufferAddress * self.num_queries);
            let timestamp_view = destination_buffer
                .slice(buffer_range)
                .get_mapped_range();
                
            // Convert the raw bytes into a vector of u64 timestamps
            bytemuck::cast_slice(&timestamp_view).to_vec()
        };

        // Unmap the buffer when we're done reading from it
        destination_buffer.unmap();

        timestamps
    }
}
