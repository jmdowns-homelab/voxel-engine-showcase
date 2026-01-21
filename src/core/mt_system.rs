use std::{
    any::Any,
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

/// A thread-safe container for systems that can be dynamically downcast.
///
/// `MtSystem` provides type-erased access to a system of type `T` that can be shared
/// across threads. It's particularly useful for dependency injection in a multi-threaded context.
/// The container uses an `Arc<RwLock<Box<T>>>` internally to provide thread-safe access to the system.
///
/// # Type Parameters
/// - `T`: The type of the contained system, must be `Send + Sync` and may be unsized
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use voxel_engine_core::core::MtSystem;
/// 
/// // Create a new thread-safe system
/// let system = MtSystem::new(Box::new(42u32));
/// 
/// // Access the system immutably
/// let value: u32 = **system.get();
/// assert_eq!(value, 42);
/// 
/// // Access the system mutably
/// *system.get_mut() = Box::new(100u32);
/// assert_eq!(**system.get(), 100);
/// ```
///
/// ## Type Erasure and Downcasting
/// ```rust
/// use std::any::Any;
/// use voxel_engine_core::core::MtSystem;
/// 
/// // Create a system with a concrete type
/// let system = MtSystem::new(Box::new(42u32));
/// 
/// // Downcast to a different type (fails if types don't match)
/// if let Some(string_system) = system.downcast::<String>() {
///     // This won't execute because we're trying to downcast u32 to String
///     unreachable!();
/// }
/// 
/// // Successful downcast
/// let number_system = system.downcast::<u32>().unwrap();
/// assert_eq!(**number_system.get(), 42);
/// ```
///
/// # Performance Considerations
/// - Read operations (`get()`) can occur concurrently
/// - Write operations (`get_mut()`) are exclusive and will block other operations
/// - The `Box` indirection adds a small runtime overhead
/// - Consider using `MtResource` for non-system data that doesn't need type erasure
pub struct MtSystem<T: Send + Sync + ?Sized> {
    pub system: Arc<RwLock<Box<T>>>,
}

impl<T: Send + Sync + 'static + ?Sized> MtSystem<T> {
    /// Creates a new `MtSystem` containing the given boxed system.
    ///
    /// # Arguments
    /// * `system` - The system to be stored, boxed
    ///
    /// # Returns
    /// A new `MtSystem` containing the provided system
    pub fn new(system: Box<T>) -> Self {
        Self {
            system: Arc::new(RwLock::new(system)),
        }
    }

    /// Returns a read-only guard that allows accessing the contained system.
    ///
    /// # Panics
    /// Panics if the lock is poisoned or cannot be acquired.
    ///
    /// # Returns
    /// A guard that provides read access to the contained system
    #[allow(dead_code)]
    pub fn get(&self) -> RwLockReadGuard<'_, Box<T>> {
        self.system.read().unwrap()
    }

    /// Returns a mutable guard that allows modifying the contained system.
    ///
    /// # Panics
    /// Panics if the lock is poisoned or cannot be acquired.
    ///
    /// # Returns
    /// A guard that provides mutable access to the contained system
    pub fn get_mut(&self) -> RwLockWriteGuard<'_, Box<T>> {
        self.system.write().unwrap()
    }

    /// Returns a type-erased version of this system as an `Arc<dyn Any + Send + Sync>`.
    ///
    /// This is used internally for dynamic downcasting.
    ///
    /// # Returns
    /// A type-erased reference-counted pointer to the system
    fn get_any(&self) -> Arc<dyn Any + Send + Sync> {
        self.system.clone()
    }

    /// Attempts to downcast the contained system to a different type `U`.
    ///
    /// This method internally uses `downcast_unchecked` but wraps it in a safe interface.
    /// The caller doesn't need to provide any safety guarantees as the method handles
    /// the unsafe code internally.
    ///
    /// # Type Parameters
    /// - `U`: The target type to downcast to, must be `Send + Sync` and may be unsized
    ///
    /// # Returns
    /// `Some(MtSystem<U>)` if the downcast succeeds, `None` otherwise
    pub fn downcast<U: Send + Sync + 'static + ?Sized>(&self) -> Option<MtSystem<U>> {
        unsafe {
            Some(MtSystem {
                system: self.get_any().downcast_unchecked::<RwLock<Box<U>>>(),
            })
        }
    }
}

impl<T: Send + Sync> Clone for MtSystem<T> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
        }
    }
}
