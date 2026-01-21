//! High-level GPU timing query management.
//!
//! This module provides a high-level interface for measuring GPU execution time
//! using WebGPU timestamp queries. It handles feature detection and provides
//! a fallback path when timestamp queries are not supported.

use std::time::Duration;

use wgpu::{Queue, RenderPassTimestampWrites};

use crate::{
    core::StSystem,
    engine_state::buffer_state::BufferState,
};

use super::raw_query_manager::RawQueryManager;

/// Contains the results of GPU timing queries.
///
/// This struct holds the duration of various GPU operations that were measured.
pub struct QueryResults {
    /// Duration of the render pipeline execution
    #[allow(dead_code)]
    pub render_pipeline_duration: Duration,
}

/// Manages GPU timing queries with feature detection.
///
/// This struct provides a high-level interface for measuring GPU execution time,
/// with automatic fallback when timestamp queries are not supported.
pub struct QueryManager {
    /// Handles the actual timestamp queries if supported
    raw_query_manager: Option<RawQueryManager>,
    /// Whether the GPU supports timestamp queries
    can_query_gpu: bool,
}

impl QueryManager {
    /// Creates a new `QueryManager` instance.
    ///
    /// # Arguments
    /// * `device` - The WebGPU device
    /// * `buffer_state` - Shared state for buffer management
    ///
    /// # Returns
    /// A new `QueryManager` instance
    pub fn new(device: &wgpu::Device, buffer_state: StSystem<BufferState>) -> Self {
        let mut can_query_gpu = false;
        let mut raw_query_manager = None;
        if device.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
            can_query_gpu = true;
            raw_query_manager = Some(RawQueryManager::new(device, buffer_state));
        }
        QueryManager {
            can_query_gpu,
            raw_query_manager,
        }
    }

    /// Gets the timestamp writes configuration for a render pass.
    ///
    /// # Returns
    /// `Some(RenderPassTimestampWrites)` if timing is supported and enabled, `None` otherwise
    pub fn request_timestamp_writes(&self) -> Option<RenderPassTimestampWrites> {
        if !self.can_query_gpu {
            return None;
        }

        self.raw_query_manager
            .as_ref()
            .unwrap()
            .request_timestamp_writes()
    }

    /// Submits a request to resolve the timestamp queries.
    ///
    /// # Arguments
    /// * `encoder` - The command encoder to record the resolve operation
    pub fn request_gpu_query(&mut self, encoder: &mut wgpu::CommandEncoder) {
        if !self.can_query_gpu {
            return;
        }

        self.raw_query_manager
            .as_mut()
            .unwrap()
            .request_gpu_query(encoder);
    }

    /// Attempts to read the timing results from the GPU.
    ///
    /// # Arguments
    /// * `queue` - The WebGPU queue for buffer operations
    ///
    /// # Returns
    /// `Some(QueryResults)` if results are available, `None` otherwise
    pub fn request_read_results(&mut self, queue: StSystem<Queue>) -> Option<QueryResults> {
        if !self.can_query_gpu {
            return None;
        }

        self.raw_query_manager
            .as_mut()
            .unwrap()
            .request_read_results(queue)
    }
}
