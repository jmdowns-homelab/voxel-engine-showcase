//! # Injection System
//!
//! This module provides a dependency injection system that allows for type-safe storage and retrieval
//! of both thread-safe and single-threaded resources and systems.
//!
//! ## Key Components
//! - `MtInjectionSystem`: Thread-safe dependency injection container
//! - `StInjectionSystem`: Single-threaded dependency injection container
//!
//! ## Usage
//! ```rust
//! use voxel_engine_core::core::{
//!     injection_system::{MtInjectionSystem, StInjectionSystem},
//!     MtSystem, StSystem,
//! };
//!
//! // Thread-safe injection
//! let mt_injector = MtInjectionSystem::new();
//! let system = mt_injector.insert(42u32);
//! let retrieved: MtSystem<u32> = mt_injector.get().unwrap();
//!
//! // Single-threaded injection
//! let st_injector = StInjectionSystem::new();
//! let system = st_injector.insert("Hello".to_string());
//! let retrieved: StSystem<String> = st_injector.get().unwrap();
//! ```

use super::{MtResource, MtSystem, StResource, StSystem};
use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

/// A thread-safe dependency injection container for managing system dependencies.
///
/// `MtInjectionSystem` allows registering and retrieving thread-safe systems by their type.
/// It uses `MtResource` and `MtSystem` internally to ensure thread safety.
///
/// # Type Parameters
/// - The container stores systems that are `Send + Sync + 'static`
///
/// # Examples
/// ```
/// use voxel_engine_core::core::injection_system::MtInjectionSystem;
///
/// let injector = MtInjectionSystem::new();
/// let system = injector.insert(42u32);
/// let retrieved: u32 = *injector.get::<u32>().unwrap().get();
/// assert_eq!(retrieved, 42);
/// ```
pub struct MtInjectionSystem {
    systems: MtResource<HashMap<TypeId, MtSystem<dyn Any + Send + Sync>>>,
}

impl MtInjectionSystem {
    /// Creates a new, empty `MtInjectionSystem`.
    ///
    /// # Returns
    /// A new `MtInjectionSystem` instance
    pub fn new() -> Self {
        Self {
            systems: MtResource::new(HashMap::new()),
        }
    }

    /// Inserts a new system into the container.
    ///
    /// If a system of the same type already exists, it will be replaced.
    ///
    /// # Type Parameters
    /// - `T`: The type of system to insert, must be `Send + Sync + 'static`
    ///
    /// # Arguments
    /// * `system` - The system to insert
    ///
    /// # Returns
    /// An `MtSystem<T>` handle to the inserted system
    ///
    /// # Panics
    /// Panics if the system cannot be downcast back to its original type
    pub fn insert<T: Send + Sync + 'static>(&self, system: T) -> MtSystem<T> {
        let type_id = TypeId::of::<T>();
        let system: Box<dyn Any + Send + Sync> = Box::new(system);
        let system: MtSystem<dyn Any + Send + Sync> = MtSystem::new(system);
        self.systems.get_mut().insert(type_id, system);

        self.get::<T>().unwrap()
    }

    /// Retrieves a system of type `T` from the container.
    ///
    /// # Type Parameters
    /// - `T`: The type of system to retrieve, must be `Send + Sync + 'static`
    ///
    /// # Returns
    /// `Some(MtSystem<T>)` if the system is found, `None` otherwise
    ///
    /// # Panics
    /// Panics if the system exists but cannot be downcast to the requested type
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<MtSystem<T>> {
        if let Some(resource) = self.systems.get().get(&TypeId::of::<T>()) {
            Some(resource.downcast::<T>().unwrap().clone())
        } else {
            panic!("No system of type {:?} found", type_name::<T>());
        }
    }
}

impl Clone for MtInjectionSystem {
    fn clone(&self) -> Self {
        Self {
            systems: self.systems.clone(),
        }
    }
}

/// A single-threaded dependency injection container for managing system dependencies.
///
/// `StInjectionSystem` allows registering and retrieving single-threaded systems by their type.
/// It uses `StResource` and `StSystem` internally and is not thread-safe.
///
/// # Type Parameters
/// - The container stores systems that are `'static`
///
/// # Examples
/// ```
/// use voxel_engine_core::core::injection_system::StInjectionSystem;
///
/// let injector = StInjectionSystem::new();
/// let system = injector.insert("Hello".to_string());
/// let retrieved: String = injector.get::<String>().unwrap().get().to_string();
/// assert_eq!(retrieved, "Hello");
/// ```
pub struct StInjectionSystem {
    systems: StResource<HashMap<TypeId, StSystem<dyn Any>>>,
}

impl StInjectionSystem {
    /// Creates a new, empty `StInjectionSystem`.
    ///
    /// # Returns
    /// A new `StInjectionSystem` instance
    pub fn new() -> Self {
        Self {
            systems: StResource::new(HashMap::new()),
        }
    }

    /// Inserts a new system into the container.
    ///
    /// If a system of the same type already exists, it will be replaced.
    ///
    /// # Type Parameters
    /// - `T`: The type of system to insert, must be `'static`
    ///
    /// # Arguments
    /// * `system` - The system to insert
    ///
    /// # Returns
    /// An `StSystem<T>` handle to the inserted system
    ///
    /// # Panics
    /// Panics if the system cannot be downcast back to its original type
    pub fn insert<T: 'static>(&self, system: T) -> StSystem<T> {
        let type_id = TypeId::of::<T>();
        let system: Box<dyn Any> = Box::new(system);
        let system: StSystem<dyn Any> = StSystem::new(system);
        self.systems.get_mut().insert(type_id, system);

        self.get::<T>().unwrap()
    }

    /// Retrieves a system of type `T` from the container.
    ///
    /// # Type Parameters
    /// - `T`: The type of system to retrieve, must implement `Any`
    ///
    /// # Returns
    /// `Some(StSystem<T>)` if the system is found, `None` otherwise
    ///
    /// # Panics
    /// Panics if the system exists but cannot be downcast to the requested type
    pub fn get<T: Any>(&self) -> Option<StSystem<T>> {
        if let Some(resource) = self.systems.get().get(&TypeId::of::<T>()) {
            Some(resource.downcast::<T>().unwrap().clone())
        } else {
            panic!("No system of type {:?} found", type_name::<T>());
        }
    }
}

impl Clone for StInjectionSystem {
    fn clone(&self) -> Self {
        Self {
            systems: self.systems.clone(),
        }
    }
}
