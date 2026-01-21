//! # Task Management System
//!
//! This module provides a cross-platform task management system for executing work
//! asynchronously across multiple threads (native) or using web workers (WASM).
//! It's designed to be efficient, scalable, and easy to use while abstracting
//! away platform-specific details.
//!
//! ## Architecture Overview
//!
//! The task management system consists of several key components:
//! - `TaskManager`: Central coordinator for task distribution and worker management
//! - `Task`: A unit of work that can be executed asynchronously
//! - `TaskResult`: The result of a completed task, which can spawn additional tasks
//! - `TaskChannel`: Communication channel between the main thread and worker threads
//!
//! ## Platform-Specific Behavior
//!
//! ### Native (Desktop) Implementation
//! - Uses Rust's standard library `std::thread` for true multi-threading
//! - Creates a pool of worker threads (configurable count, defaults to number of CPU cores)
//! - Each worker has a dedicated channel for task distribution
//! - Supports true parallel execution across CPU cores
//! - Low-latency communication between threads
//!
//! ### Web (WASM) Implementation
//! - Uses `wasm_thread` crate to simulate multi-threading
//! - Limited by browser's Web Worker API and single-threaded nature of JavaScript
//! - Tasks are processed asynchronously but may not run in parallel
//! - Higher communication overhead between main thread and workers
//! - Automatically falls back to a single worker if Web Workers aren't available
//!
//! ## Task Lifecycle
//! 1. Tasks are created and published via `TaskManager::publish_task()`
//! 2. The manager distributes tasks to available worker channels using round-robin
//! 3. Workers process tasks asynchronously and return results
//! 4. Results are processed on the main thread in `process_completed_tasks()`
//! 5. Results can spawn new tasks or issue buffer write commands
//! 6. The cycle continues until all work is complete
//!
//! ## Performance Considerations
//! - **Task Granularity**: Balance between too small (high overhead) and too large (poor load balancing)
//! - **Native**: Ideal for CPU-bound tasks that benefit from true parallelism
//! - **Web**: Best for I/O-bound tasks; minimize data transfer between threads
//! - **Memory**: Each task should own its data to avoid excessive cloning
//! - **Blocking**: Avoid blocking operations in tasks that could starve other work
//!
//! ## Example Usage
//! ```rust
//! // Create a task manager with default settings
//! let mut task_manager = TaskManager::new(
//!     num_workers,
//!     mt_injection_system,
//!     st_injection_system,
//! );
//!
//! // Publish a task for background processing
//! task_manager.publish_task(Box::new(MyTask::new(...)));
//!
//! // In your main/game loop:
//! task_manager.process_completed_tasks(&mut buffer_state);
//! task_manager.process_queued_tasks();
//! ```

pub mod task;

use crate::core::injection_system::{MtInjectionSystem, StInjectionSystem};
use log::info;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use task::{Task, TaskResult};

#[cfg(target_family = "wasm")]
mod wasm_imports {
    pub use wasm_thread as thread;
    pub use wasm_thread::JoinHandle;
}

#[cfg(target_family = "wasm")]
use self::wasm_imports::*;

#[cfg(not(target_family = "wasm"))]
use std::thread::{self, JoinHandle};

use super::buffer_state::BufferState;

/// A communication channel between the main thread and a worker thread.
///
/// This is the core communication primitive that allows the `TaskManager` to
/// distribute work to background threads and receive the results.
///
/// # Fields
/// - `task_sender`: Sends tasks from main thread to worker
/// - `result_receiver`: Receives task results from worker
/// - `num_tasks_in_flight`: Tracks number of tasks currently being processed
/// - `_worker`: Handle to the worker thread (kept alive by this struct)
///
/// # Implementation Notes
/// - Each channel is backed by an OS-level thread (native) or Web Worker (WASM)
/// - Uses MPSC (multi-producer, single-consumer) channels for communication
/// - Automatically cleans up resources when dropped
#[derive(Debug)]
pub struct TaskChannel {
    task_sender: Sender<Box<dyn Task + Send>>,
    result_receiver: Receiver<Box<dyn TaskResult + Send>>,
    num_tasks_in_flight: usize,
    _worker: JoinHandle<()>,
}

/// Manages a pool of worker threads and coordinates task execution.
///
/// The `TaskManager` is responsible for:
/// - Creating and managing worker threads
/// - Distributing tasks across available workers
/// - Collecting and processing task results
/// - Handling task queuing when all workers are busy
/// - Managing the lifecycle of worker threads
///
/// # Fields
/// - `channels`: Set of active worker channels
/// - `queued_tasks`: Tasks waiting for an available worker
/// - `current_channel`: Index for round-robin scheduling
/// - `st_injection_system`: Single-threaded services (main thread only)
/// - `mt_injection_system`: Thread-safe services
///
/// # Implementation Notes
/// - Thread-safe: Can be used from any thread
/// - Drop-safe: Automatically cleans up worker threads
/// - Panic-safe: Worker thread panics won't crash the application
pub struct TaskManager {
    channels: Vec<TaskChannel>,
    queued_tasks: VecDeque<Box<dyn Task + Send>>,
    current_channel: usize,
    st_injection_system: StInjectionSystem,
    mt_injection_system: MtInjectionSystem,
}

/// Maximum number of tasks that can be in flight per worker channel.
///
/// This is set to 1 to ensure tasks are processed in order within each channel.
/// Increasing this value would allow for pipelining but would require more
/// sophisticated task dependency management.
pub const MAX_TASKS_IN_FLIGHT: usize = 1;

impl TaskManager {
    /// Creates a new `TaskManager` with the specified number of worker threads.
    ///
    /// # Arguments
    /// * `num_workers` - Number of worker threads to create. On web targets, this is typically 1-2
    ///   due to browser limitations. On native, this can be set to the number of CPU cores.
    /// * `st_injection_system` - Single-threaded injection system for main-thread services
    /// * `mt_injection_system` - Thread-safe injection system for worker-thread services
    ///
    /// # Panics
    /// Panics if the underlying thread creation fails.
    ///
    /// # Platform Notes
    /// - **Native**: Creates actual OS threads
    /// - **Web**: Creates Web Workers (if available) or falls back to a single worker
    pub fn new(
        num_workers: usize,
        st_injection_system: StInjectionSystem,
        mt_injection_system: MtInjectionSystem,
    ) -> Self {
        let mut channels = Vec::with_capacity(num_workers);

        for _ in 0..num_workers {
            let (task_tx, task_rx) = channel::<Box<dyn Task + Send>>();
            let (result_tx, result_rx) = channel::<Box<dyn TaskResult + Send>>();

            let task_closure = move || {
                while let Ok(task) = task_rx.recv() {
                    let result = task.process();
                    let _ = result_tx.send(result);
                }
            };

            log::info!(
                "Available parallelism: {:?}",
                thread::available_parallelism()
            );

            #[cfg(target_family = "wasm")]
            let worker = thread::spawn(task_closure);

            #[cfg(not(target_family = "wasm"))]
            let worker = thread::spawn(task_closure);

            channels.push(TaskChannel {
                task_sender: task_tx,
                result_receiver: result_rx,
                num_tasks_in_flight: 0,
                _worker: worker,
            });
        }

        TaskManager {
            channels,
            queued_tasks: VecDeque::new(),
            current_channel: 0,
            st_injection_system,
            mt_injection_system,
        }
    }

    /// Attempts to send a task to a specific worker channel.
    ///
    /// This is a low-level method that tries to send a task to a specific worker.
    /// Most users should use `publish_task()` instead, which handles worker
    /// selection automatically.
    ///
    /// # Arguments
    /// * `task` - The task to send to the worker
    /// * `channel_idx` - Index of the target worker channel (must be valid)
    ///
    /// # Returns
    /// - `Ok(())` if the task was successfully sent to the worker
    /// - `Err(task)` if the send failed (e.g., worker disconnected)
    ///
    /// # Notes
    /// - Automatically increments the in-flight task counter on success
    /// - Returns the original task on failure, allowing for requeueing
    /// - Panics if `channel_idx` is out of bounds
    ///
    /// # Example
    /// ```rust
    /// if let Err(task) = task_manager.try_send_task(task, 0) {
    ///     // Handle send failure (e.g., requeue or log error)
    ///     log::error!("Failed to send task to worker 0");
    /// }
    fn try_send_task(
        &mut self,
        task: Box<dyn Task + Send>,
        channel_idx: usize,
    ) -> Result<(), Box<dyn Task + Send>> {
        match self.channels[channel_idx].task_sender.send(task) {
            Ok(_) => {
                self.channels[channel_idx].num_tasks_in_flight += 1;
                Ok(())
            }
            Err(task) => {
                Err(task.0)
            }
        }
    }

    /// Finds an available worker channel that can accept a new task.
    ///
    /// This implements a round-robin scheduling strategy starting from the last
    /// used channel to ensure even distribution of tasks across all workers.
    /// Channels that have reached their maximum number of in-flight tasks are
    /// automatically skipped.
    ///
    /// # Returns
    /// - `Some(usize)` index of an available channel that can accept a new task
    /// - `None` if all channels are busy or there are no channels available
    ///
    /// # Implementation Details
    /// - Uses a round-robin approach to distribute load evenly
    /// - Skips channels that have reached `MAX_TASKS_IN_FLIGHT`
    /// - Handles the case where all channels are busy
    /// - Wraps around to the beginning when reaching the end of the channel list
    ///
    /// # Performance
    /// - O(n) where n is the number of worker channels
    /// - Typically very fast since the number of workers is small (usually CPU core count)
    /// - No allocations or system calls
    fn find_available_channel(&self) -> Option<usize> {
        if self.channels.is_empty() {
            return None;
        }

        // Check if all channels are full
        if self
            .channels
            .iter()
            .all(|channel| channel.num_tasks_in_flight >= MAX_TASKS_IN_FLIGHT)
        {
            return None;
        }

        // Find next available channel using round-robin
        let start_channel = self.current_channel;
        let mut current = start_channel;

        loop {
            if self.channels[current].num_tasks_in_flight < MAX_TASKS_IN_FLIGHT {
                return Some(current);
            }
            current = (current + 1) % self.channels.len();
            if current == start_channel {
                // This shouldn't happen due to the earlier check
                info!("All channels are full, but missed the first check");
                return None;
            }
        }
    }

    /// Publishes a new task for execution.
    ///
    /// This is the primary method for scheduling work to be done in the background.
    /// The task will be executed as soon as a worker becomes available, or queued
    /// if all workers are busy.
    ///
    /// # Arguments
    /// * `task` - The task to be executed. Must implement the `Task` trait.
    ///
    /// # Returns
    /// - `true` if the task was immediately scheduled on an available worker
    /// - `false` if the task was queued because all workers are busy
    ///
    /// # Thread Safety
    /// - Safe to call from any thread
    /// - Internally synchronized - no additional locking needed
    /// - Non-blocking - returns immediately in all cases
    ///
    /// # Example
    /// ```rust
    /// // Create and publish a task
    /// let task = MyTask::new(/* ... */);
    /// let was_scheduled = task_manager.publish_task(Box::new(task));
    ///
    /// if !was_scheduled {
    ///     log::debug!("Task queued - all workers busy");
    /// }
    /// ```
    ///
    /// # Performance
    /// - Very fast in the common case (worker available)
    /// - May allocate if the task needs to be queued
    /// - Thread contention is minimal due to lock-free design
    pub fn publish_task(&mut self, task: Box<dyn Task + Send>) -> bool {
        if self.channels.is_empty() {
            self.queued_tasks.push_back(task);
            return false;
        }

        match self.find_available_channel() {
            Some(channel_idx) => {
                match self.try_send_task(task, channel_idx) {
                    Ok(_) => {
                        self.current_channel = (channel_idx + 1) % self.channels.len();
                        true
                    }
                    Err(task) => {
                        self.queued_tasks.push_back(task);
                        false
                    }
                }
            }
            None => {
                self.queued_tasks.push_back(task);
                false
            }
        }
    }

    /// Processes any queued tasks if workers are available.
    ///
    /// This method should be called periodically (typically once per frame) to
    /// ensure that queued tasks are processed as workers become available. It
    /// will attempt to schedule as many queued tasks as possible until either
    /// the queue is empty or all workers are busy.
    ///
    /// # Implementation Details
    /// - Processes tasks in FIFO order (oldest first)
    /// - Stops at the first task that can't be scheduled (all workers busy)
    /// - Automatically handles worker disconnection
    /// - Maintains task order within each worker channel
    ///
    /// # Usage
    /// Call this in your main/game loop to ensure continuous processing:
    /// ```rust
    /// // In your game loop:
    /// task_manager.process_queued_tasks();
    /// ```
    ///
    /// # Performance
    /// - O(n) where n is the number of queued tasks processed
    /// - Very fast when queue is empty (immediate return)
    /// - May allocate if tasks need to be moved to the queue
    pub fn process_queued_tasks(&mut self) {
        if self.queued_tasks.is_empty() {
            return;
        }

        // First check if we have any available channels
        match self.find_available_channel() {
            None => {
            } // No available channels, keep tasks queued
            Some(mut channel_idx) => {

                // Process tasks while we have available channels
                while let Some(task) = self.queued_tasks.pop_front() {
                    match self.try_send_task(task, channel_idx) {
                        Ok(_) => {
                            // Check if next channel is available
                            match self.find_available_channel() {
                                Some(next_idx) => channel_idx = next_idx,
                                None => break, // No more available channels
                            }
                        }
                        Err(task) => {
                            // Channel is disconnected, put task back and stop processing
                            self.queued_tasks.push_front(task);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Processes all completed task results from worker threads.
    ///
    /// This is a critical method that must be called on the main thread to
    /// process the results of completed tasks. It handles any buffer write
    /// commands and can spawn additional tasks as needed.
    ///
    /// # Arguments
    /// * `buffer_state` - The buffer state to apply write commands to. This is
    ///   typically the same buffer state used throughout your application.
    ///
    /// # Implementation Details
    /// - Processes results in the order they were received
    /// - Executes buffer write commands immediately
    /// - Can spawn new tasks if the result requests it
    /// - Handles worker disconnection gracefully
    ///
    /// # Usage
    /// Call this in your main/game loop to process completed tasks:
    /// ```rust
    /// // In your game loop:
    /// task_manager.process_completed_tasks(&mut buffer_state);
    /// ```
    ///
    /// # Thread Safety
    /// - Must be called from the main thread
    /// - Not thread-safe - do not call from multiple threads
    ///
    /// # Performance
    /// - O(m + n) where m is the number of results and n is the number of new tasks
    /// - Performance depends on the complexity of the task result handlers
    /// - May allocate when processing results or spawning new tasks
    pub fn process_completed_tasks(&mut self, buffer_state: &BufferState) {
        let mut tasks_to_queue = Vec::new();
        for channel in &mut self.channels {
            while let Ok(result) = channel.result_receiver.try_recv() {
                channel.num_tasks_in_flight -= 1;
                let (new_tasks, write_commands) =
                    result.handle_result(&self.mt_injection_system, &self.st_injection_system);
                for command in write_commands {
                    //log::error!("Write command: {:?}", command);
                    buffer_state.write(command);
                }
                for task in new_tasks {
                    tasks_to_queue.push(task);
                }
            }
        }

        for task in tasks_to_queue {
            self.publish_task(task);
        }
    }
}
