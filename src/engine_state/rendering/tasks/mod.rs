//! Background tasks for the rendering system.
//!
//! This module contains background tasks that handle potentially expensive
//! rendering-related operations, such as mesh generation and resource loading.
//! These tasks are designed to run in parallel to keep the main render thread
//! responsive.
//!
//! # Available Tasks
//! - `ChunkMeshGenerationTask`: Generates mesh data for chunks in the background

pub mod chunk_mesh_generation_task;
