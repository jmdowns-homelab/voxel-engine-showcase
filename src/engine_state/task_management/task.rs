//! # Task System Core Traits
//!
//! This module defines the fundamental building blocks of the task system,
//! which provides a framework for executing work asynchronously across multiple threads.
//!
//! ## Core Components
//! - `Task`: Represents a unit of work that can be executed asynchronously
//! - `TaskResult`: Represents the result of a completed task
//!
//! ## Task Lifecycle
//! 1. A `Task` is created and scheduled via `TaskManager::publish_task()`
//! 2. The task's `process()` method is called on a worker thread
//! 3. The task returns a boxed `TaskResult`
//! 4. The result's `handle_result()` is called on the main thread
//! 5. The result can spawn new tasks or issue buffer write commands
//!
//! ## Thread Safety
//! - `Task` must be `Send` to be transferred between threads
//! - `TaskResult` must be `Send` to be transferred back to the main thread
//! - All shared state must be properly synchronized

use crate::{
    core::injection_system::{MtInjectionSystem, StInjectionSystem},
    engine_state::buffer_state::BufferWriteCommand,
};

/// A trait representing a unit of work that can be executed asynchronously.
///
/// Tasks are the primary mechanism for offloading work from the main thread to
/// background workers. They should be designed to be self-contained and own all
/// the data they need to perform their work.
///
/// # Implementation Guidelines
/// - Must be `Send` to be transferred between threads
/// - Should be relatively coarse-grained to amortize task scheduling overhead
/// - Should avoid holding references to data that might be modified elsewhere
/// - Should be `'static` (no non-static references)

pub trait Task: Send {
        /// Processes the task and returns a result.
    ///
    /// This method contains the actual work to be performed asynchronously.
    /// It runs on a background thread and should avoid blocking operations
    /// that could starve other tasks.
    ///
    /// # Implementation Notes
    /// - Must be thread-safe and not access thread-local data
    /// - Should handle any errors internally and return an appropriate result
    /// - Can use `?` to propagate errors if they implement `std::error::Error`
    ///
    /// # Returns
    /// A boxed `TaskResult` that will be processed on the main thread.
    fn process(&self) -> Box<dyn TaskResult + Send>;
}

/// A trait representing the result of processing a `Task`.
///
/// Task results are processed on the main thread and can perform actions such as:
/// - Spawning new tasks for further processing
/// - Issuing buffer write commands to update rendering state
/// - Accessing engine services through the provided injection systems
///
/// # Implementation Guidelines
/// - Must be `Send` to be transferred back to the main thread
/// - Should be as lightweight as possible
/// - Can hold references to data created during task processing
/// - Should avoid expensive computations in `handle_result()`
    /// Handles the result of a completed task on the main thread.
    ///
    /// This method is called on the main thread and has access to the full
    /// engine state through the injection systems.
    ///
    /// # Arguments
    /// * `mt_injection_system`: Thread-safe services that can be accessed from any thread
    /// * `st_injection_system`: Single-threaded services (main thread only)
    ///
    /// # Returns
    /// A tuple containing:
    /// 1. A vector of new tasks to schedule (can be empty)
    /// 2. A vector of buffer write commands to execute (can be empty)
    ///
    /// # Implementation Notes
    /// - Runs on the main thread - keep it fast to avoid frame drops
    /// - Can schedule additional tasks for further processing
    /// - Should handle any errors internally
pub trait TaskResult: Send {
    fn handle_result(
        self: Box<Self>,
        mt_injection_system: &MtInjectionSystem,
        st_injection_system: &StInjectionSystem,
    ) -> (Vec<Box<dyn Task>>, Vec<BufferWriteCommand>);
}
