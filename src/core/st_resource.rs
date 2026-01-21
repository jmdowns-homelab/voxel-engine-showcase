use std::{
    rc::Rc,
    sync::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A single-threaded, reference-counted resource with interior mutability.
///
/// `StResource` provides interior mutability for a value of type `T` in a single-threaded context.
/// It uses `Rc<RwLock<T>>` internally to manage the resource. This type is optimized for
/// single-threaded scenarios where the overhead of atomic operations in `Arc` is not needed.
///
/// # Type Parameters
/// - `T`: The type of the contained resource
///
/// # Examples
///
/// ## Basic Usage
/// ```
/// use voxel_engine_core::core::StResource;
/// 
/// // Create a new single-threaded counter
/// let counter = StResource::new(0);
/// 
/// // Modify the counter value
/// *counter.get_mut() += 1;
/// 
/// // Read the counter value
/// assert_eq!(*counter.get(), 1);
/// ```
///
/// ## Cloning and Shared Ownership
/// ```
/// use voxel_engine_core::core::StResource;
/// 
/// let resource = StResource::new(vec![1, 2, 3]);
/// let clone1 = resource.clone();
/// let clone2 = resource.clone();
/// 
/// // All clones share the same underlying data
/// clone1.get_mut().push(4);
/// assert_eq!(resource.get().len(), 4);
/// assert_eq!(clone2.get().len(), 4);
/// ```
///
/// # Panics
/// - Panics if a read lock is held while trying to acquire a write lock in the same thread
/// - Panics if a write lock is held while trying to acquire any lock in the same thread
///
/// # Performance Considerations
/// - More efficient than `MtResource` in single-threaded contexts
/// - Not thread-safe - do not use across thread boundaries
/// - Uses `RwLock` for interior mutability, which has some overhead
pub struct StResource<T> {
    pub resource: Rc<RwLock<T>>,
}

impl<T> StResource<T> {
    /// Creates a new `StResource` containing the given value.
    ///
    /// # Arguments
    /// * `resource` - The value to be stored in the resource
    ///
    /// # Returns
    /// A new `StResource` containing the provided value
    pub fn new(resource: T) -> Self {
        Self {
            resource: Rc::new(RwLock::new(resource)),
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

impl<T> Clone for StResource<T> {
    fn clone(&self) -> Self {
        Self {
            resource: self.resource.clone(),
        }
    }
}
