//! # Graphics Resources Builder
//!
//! This module handles the creation and management of graphics resources required by the application.
//! It provides platform-agnostic interfaces for initializing WebGPU resources and managing
//! the graphics context.
//!
//! The main components are:
//! - `Graphics`: Holds all graphics-related resources
//! - `GraphicsBuilder`: Helper for asynchronous graphics initialization
//! - `MaybeGraphics`: Represents the various states of graphics initialization

use std::future::Future;
use std::sync::Arc;

#[cfg(target_family = "wasm")]
use {futures::future, log::error, wasm_bindgen::UnwrapThrowExt};

#[cfg(not(target_family = "wasm"))]
use std::path::Path;

#[cfg(target_family = "wasm")]
use serde::Deserialize;

#[cfg(target_family = "wasm")]
#[derive(Deserialize)]
struct VersionContract {
    shaders: String,
    textures: String,
}
use wgpu::{Adapter, Device, Features, Instance, Queue, Surface, SurfaceConfiguration};
use winit::{
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::Window,
};

#[cfg(target_family = "wasm")]
use crate::CANVAS_ID;

/// Contains all graphics-related resources required by the application.
///
/// This struct holds handles to WebGPU resources and other graphics-related state.
/// It's typically created during application initialization and passed to systems
/// that need to interact with the GPU.
#[allow(dead_code)]
#[derive(Default)]
pub struct Graphics {
    pub window: Option<Arc<Window>>,
    pub instance: Option<Instance>,
    pub surface: Option<Surface<'static>>,
    pub surface_config: Option<SurfaceConfiguration>,
    pub adapter: Option<Adapter>,
    pub device: Option<Device>,
    pub queue: Option<Queue>,
    pub shader_file_string: String,
    pub ui_shader_file_string: String,
    pub atlas_bytes: Vec<u8>,
    pub is_surface_configured: bool,
}

/// Asynchronously creates and initializes all required graphics resources.
///
/// This function handles the platform-specific details of setting up the WebGPU context,
/// including window creation, surface setup, and device initialization.
///
/// # Arguments
/// * `event_loop` - The active event loop used to create the window and surface
///
/// # Returns
/// A `Future` that resolves to the initialized `Graphics` when complete
fn create_graphics(event_loop: &ActiveEventLoop) -> impl Future<Output = Graphics> + 'static {
    #[allow(unused_mut)]
    let mut window_attrs = Window::default_attributes();

    #[cfg(target_family = "wasm")]
    {
        use web_sys::wasm_bindgen::JsCast;
        use winit::platform::web::WindowAttributesExtWebSys;

        let window = web_sys::window().unwrap_throw();
        let document = window.document().unwrap_throw();
        let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
        let html_canvas_element = canvas.unchecked_into();
        window_attrs = window_attrs.with_canvas(Some(html_canvas_element));
    }

    let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

    // The instance is a handle to our GPU
    // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        #[cfg(not(target_family = "wasm"))]
        backends: wgpu::Backends::PRIMARY,
        #[cfg(target_family = "wasm")]
        backends: wgpu::Backends::GL | wgpu::Backends::BROWSER_WEBGPU,
        flags: wgpu::InstanceFlags::empty(),
        backend_options: wgpu::BackendOptions::from_env_or_default(),
    });

    let surface = instance.create_surface(window.clone()).unwrap();

    async move {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let mut required_features = Features::empty();
        let mut required_limits = wgpu::Limits::default();

        #[cfg(feature = "wgpu_timestamp_query")]
        {
            required_features |= Features::TIMESTAMP_QUERY | wgpu::Features::MULTI_DRAW_INDIRECT;
        }

        if cfg!(not(target_family = "wasm")) {
            required_features |= wgpu::Features::POLYGON_MODE_LINE
                | wgpu::Features::MULTI_DRAW_INDIRECT
                | wgpu::Features::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING
                | wgpu::Features::TEXTURE_BINDING_ARRAY
                | wgpu::Features::VERTEX_WRITABLE_STORAGE;

            required_limits.max_binding_array_elements_per_shader_stage = 500_000;
        } else {
            required_limits = wgpu::Limits::downlevel_webgl2_defaults();
        }

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    required_features,
                    required_limits,
                    label: None,
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                    trace: wgpu::Trace::Off
                }
            )
            .await
            .unwrap();

        let size = window.inner_size();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        #[cfg(not(target_family = "wasm"))]
        {
            let mut shader_import_string = String::from("");

            if device.features().contains(Features::TEXTURE_BINDING_ARRAY) {
                shader_import_string.push_str("_texture_binding_array");
            }

            let shader_name = format!("assets/shaders/basic_shader{}.wgsl", shader_import_string);
            let ui_shader_name = "assets/shaders/ui/shader.wgsl";

            let shader_string = std::fs::read_to_string(Path::new(&shader_name)).unwrap();
            let ui_shader_string = std::fs::read_to_string(Path::new(ui_shader_name)).unwrap();

            let atlas_bytes = std::fs::read("assets/textures/data.atl").unwrap();

            surface.configure(&device, &surface_config);
            Graphics {
                window: Some(window),
                instance: Some(instance),
                surface: Some(surface),
                surface_config: Some(surface_config),
                adapter: Some(adapter),
                device: Some(device),
                queue: Some(queue),
                shader_file_string: shader_string,
                ui_shader_file_string: ui_shader_string,
                atlas_bytes,
                is_surface_configured: true,
            }
        }

        #[cfg(target_family = "wasm")]
        {
            let http_client = reqwest::Client::new();

            let versions_contract = http_client
                .get("https://jdowns.xyz/versions")
                .header("cache", "reload")
                .send()
                .await
                .unwrap()
                .json::<VersionContract>()
                .await
                .unwrap();

            let shader_request_body = r#"{"features": []}"#;
            let shader_async_response = http_client
                .post(format!(
                    "https://jdowns.xyz/shaders?with={}",
                    versions_contract.shaders
                ))
                .header("Content-Type", "application/json")
                .body(shader_request_body)
                .send();

            let ui_shader_async_response = http_client
                .post(format!(
                    "https://jdowns.xyz/ui-shader?with={}",
                    versions_contract.shaders
                ))
                .send();

            let atlas_async_response = http_client
                .get(format!(
                    "https://jdowns.xyz/atlas?with={}",
                    versions_contract.textures
                ))
                .send();

            let (shader_response, ui_shader_response, atlas_response) =
                future::join3(shader_async_response, ui_shader_async_response, atlas_async_response).await;

            let shader_string = match shader_response {
                Ok(response) => response.text().await.unwrap(),
                Err(e) => {
                    error!("Error fetching shader string: {:?}", e);
                    String::from("")
                }
            };

            let ui_shader_string = match ui_shader_response {
                Ok(response) => response.text().await.unwrap(),
                Err(e) => {
                    error!("Error fetching UI shader string: {:?}", e);
                    String::from("")
                }
            };

            let atlas_bytes = match atlas_response {
                Ok(response) => response.bytes().await.unwrap().to_vec(),
                Err(e) => {
                    error!("Error fetching atlas bytes: {:?}", e);
                    Vec::new()
                }
            };

            Graphics {
                window: Some(window),
                instance: Some(instance),
                surface: Some(surface),
                surface_config: Some(surface_config),
                adapter: Some(adapter),
                device: Some(device),
                queue: Some(queue),
                shader_file_string: shader_string,
                ui_shader_file_string: ui_shader_string,
                atlas_bytes,
                is_surface_configured: false,
            }
        }
    }
}

/// Helper struct for managing the asynchronous initialization of graphics resources.
///
/// This handles the platform-specific details of setting up the WebGPU context
/// and related resources.
pub struct GraphicsBuilder {
    event_loop_proxy: Option<EventLoopProxy<Graphics>>,
}

/// Represents the possible states of the graphics initialization process.
///
/// This enum is used to track the current state of graphics resources
/// throughout the application's lifecycle.
pub enum MaybeGraphics {
    /// Initial state before any initialization has been attempted
    #[allow(dead_code)]
    Uninitialized,
    
    /// State during asynchronous graphics initialization
    Builder(GraphicsBuilder),
    
    /// State when graphics resources are fully initialized and ready for use
    Graphics(Graphics),
    
    /// State after graphics resources have been moved to another owner
    Moved,
}

impl GraphicsBuilder {
    /// Creates a new GraphicsBuilder with the specified event loop proxy.
    ///
    /// # Arguments
    /// * `event_loop_proxy` - Used to send the initialized graphics resources back to the main thread
    ///
    /// # Returns
    /// A new `GraphicsBuilder` instance ready to begin graphics initialization
    pub fn new(event_loop_proxy: EventLoopProxy<Graphics>) -> Self {
        Self {
            event_loop_proxy: Some(event_loop_proxy),
        }
    }

    /// Initiates the asynchronous graphics initialization process.
    ///
    /// This method spawns a new task to create the graphics resources and sends
    /// them back to the main thread using the event loop proxy.
    ///
    /// # Arguments
    /// * `event_loop` - The active event loop used to create the graphics context
    ///
    /// # Panics
    /// Panics if the event loop proxy has already been taken or if sending fails
    pub fn build_and_send(&mut self, event_loop: &ActiveEventLoop) {
        let Some(event_loop_proxy) = self.event_loop_proxy.take() else {
            // event_loop_proxy is already spent - we already constructed Graphics
            return;
        };

        #[cfg(target_family = "wasm")]
        {
            let gfx_fut = create_graphics(event_loop);
            wasm_bindgen_futures::spawn_local(async move {
                let gfx = gfx_fut.await;
                assert!(event_loop_proxy.send_event(gfx).is_ok());
            });
        }

        #[cfg(not(target_family = "wasm"))]
        {
            let gfx = pollster::block_on(create_graphics(event_loop));
            assert!(event_loop_proxy.send_event(gfx).is_ok());
        }
    }
}
