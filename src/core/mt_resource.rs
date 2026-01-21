use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// A thread-safe, reference-counted resource container with read-write locking.
///
/// `MtResource` provides synchronized access to a value of type `T` that can be shared
/// across threads. It uses an `Arc<RwLock<T>>` internally to manage concurrent access.
/// This type is suitable for scenarios where multiple threads need read access to shared
/// data, with exclusive write access when needed.
///
/// # Type Parameters
/// - `T`: The type of the contained resource, must be `Send + Sync`
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use voxel_engine_core::core::MtResource;
/// 
/// // Create a new thread-safe counter
/// let counter = MtResource::new(0);
/// 
/// // Safely increment the counter from any thread
/// *counter.get_mut() += 1;
/// 
/// // Read the counter value
/// assert_eq!(*counter.get(), 1);
/// ```
///
/// ## Sharing Between Threads
/// ```
/// # use std::thread;
/// use voxel_engine_core::core::MtResource;
/// 
/// let counter = MtResource::new(0);
/// let counter_clone = counter.clone();
/// 
/// let handle = thread::spawn(move || {
///     *counter_clone.get_mut() += 1;
/// });
/// 
/// handle.join().unwrap();
/// assert_eq!(*counter.get(), 1);
/// ```
///
/// # Performance Considerations
/// - Read operations (`get()`) can occur concurrently
/// - Write operations (`get_mut()`) are exclusive and will block other operations
/// - Prefer using `get()` when possible to allow concurrent reads
/// - Consider using `RwLock` directly for more fine-grained control over locking
pub struct MtResource<T: Send + Sync> {
    pub resource: Arc<RwLock<T>>,
}

impl<T: Send + Sync + 'static> MtResource<T> {
    /// Creates a new `MtResource` containing the given value.
    ///
    /// # Arguments
    /// * `resource` - The value to be stored in the resource
    ///
    /// # Returns
    /// A new `MtResource` containing the provided value
    pub fn new(resource: T) -> Self {
        Self {
            resource: Arc::new(RwLock::new(resource)),
        }
    }

    /// Returns a read-only guard that allows reading the contained value.
    ///
    /// # Panics
    /// Panics if the lock is poisoned or cannot be acquired.
    ///
    /// # Returns
    /// A guard that provides read access to the contained value
    pub fn get(&self) -> RwLockReadGuard<'_, T> {
        self.resource.read().unwrap()
    }

    /// Returns a mutable guard that allows modifying the contained value.
    ///
    /// # Panics
    /// Panics if the lock is poisoned or cannot be acquired.
    ///
    /// # Returns
    /// A guard that provides mutable access to the contained value
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, T> {
        self.resource.write().unwrap()
    }
}

impl<T: Send + Sync> Clone for MtResource<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}
