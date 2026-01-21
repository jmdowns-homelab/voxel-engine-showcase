use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

/// A single-threaded container for systems that can be dynamically downcast.
///
/// `StSystem` provides type-erased access to a system of type `T` in a single-threaded context.
/// It's particularly useful for dependency injection when thread safety is not required.
/// The container uses `Rc<RefCell<Box<T>>>` internally to provide interior mutability.
///
/// # Type Parameters
/// - `T`: The type of the contained system, may be unsized
///
/// # Examples
///
/// ## Basic Usage
/// ```rust
/// use voxel_engine_core::core::StSystem;
/// 
/// // Create a new single-threaded system
/// let system = StSystem::new(Box::new(42u32));
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
/// use voxel_engine_core::core::StSystem;
/// 
/// // Create a system with a concrete type
/// let system = StSystem::new(Box::new("Hello, World!".to_string()));
/// 
/// // Downcast to a different type (fails if types don't match)
/// if let Some(number_system) = system.downcast::<u32>() {
///     // This won't execute because we're trying to downcast String to u32
///     unreachable!();
/// }
/// 
/// // Successful downcast
/// let string_system = system.downcast::<String>().unwrap();
/// assert_eq!(**string_system.get(), "Hello, World!");
/// ```
///
/// # Panics
/// - Panics if a borrow is held while trying to mutably borrow
/// - Panics if a mutable borrow is held while trying to borrow
///
/// # Performance Considerations
/// - More efficient than `MtSystem` in single-threaded contexts
/// - Not thread-safe - do not use across thread boundaries
/// - The `RefCell` adds a small runtime overhead for borrow checking
/// - Consider using `StResource` for non-system data that doesn't need type erasure
pub struct StSystem<T: ?Sized> {
    pub system: Rc<RefCell<Box<T>>>,
}

impl<T: 'static + ?Sized> StSystem<T> {
    /// Returns an immutable reference to the contained system.
    ///
    /// # Panics
    /// Panics if the value is currently mutably borrowed.
    ///
    /// # Returns
    /// An immutable reference to the contained system
    pub fn get(&self) -> Ref<'_, Box<T>> {
        self.system.borrow()
    }

    /// Returns a mutable reference to the contained system.
    ///
    /// # Panics
    /// Panics if the value is currently borrowed.
    ///
    /// # Returns
    /// A mutable reference to the contained system
    pub fn get_mut(&self) -> RefMut<'_, Box<T>> {
        self.system.borrow_mut()
    }

    /// Returns a type-erased version of this system as an `Rc<dyn Any>`.
    ///
    /// This is used internally for dynamic downcasting.
    ///
    /// # Returns
    /// A type-erased reference-counted pointer to the system
    pub fn get_any(&self) -> Rc<dyn Any> {
        self.system.clone()
    }

    /// Attempts to downcast the contained system to a different type `U`.
    ///
    /// This method internally uses `downcast_unchecked` but wraps it in a safe interface.
    /// The caller doesn't need to provide any safety guarantees as the method handles
    /// the unsafe code internally.
    ///
    /// # Type Parameters
    /// - `U`: The target type to downcast to, may be unsized
    ///
    /// # Returns
    /// `Some(StSystem<U>)` if the downcast succeeds, `None` otherwise
    pub fn downcast<U: 'static + ?Sized>(&self) -> Option<StSystem<U>> {
        unsafe {
            Some(StSystem {
                system: self.get_any().downcast_unchecked::<RefCell<Box<U>>>(),
            })
        }
    }
}

impl<T: ?Sized> StSystem<T> {
    /// Creates a new `StSystem` containing the given boxed system.
    ///
    /// # Arguments
    /// * `system` - The system to be stored, boxed
    ///
    /// # Returns
    /// A new `StSystem` containing the provided system
    pub fn new(system: Box<T>) -> Self {
        Self {
            system: Rc::new(RefCell::new(system)),
        }
    }
}

impl<T> Clone for StSystem<T> {
    fn clone(&self) -> Self {
        Self {
            system: self.system.clone(),
        }
    }
}
