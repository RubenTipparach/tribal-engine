use ash::{vk, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::{CStr, CString};
use winit::window::Window;
use glam::{Mat4, Quat, Vec2, Vec3};
use imgui::Context;

use crate::mesh::{Mesh, Vertex};
use crate::material::MaterialProperties;
use crate::core::lighting::{DirectionalLight, PointLight};
use crate::imgui_renderer::ImGuiRenderer;
use crate::ui::UiManager;
use crate::gizmo::GizmoMesh;

const MAX_FRAMES_IN_FLIGHT: usize = 2;

/// Push constants for mesh rendering (model matrix + material properties)
#[repr(C)]
#[derive(Copy, Clone)]
struct MeshPushConstants {
    model: glam::Mat4,           // 64 bytes
    albedo: glam::Vec3,          // 12 bytes
    metallic: f32,               // 4 bytes
    roughness: f32,              // 4 bytes
    ambient_strength: f32,       // 4 bytes
    gi_strength: f32,            // 4 bytes
}

unsafe impl bytemuck::Pod for MeshPushConstants {}
unsafe impl bytemuck::Zeroable for MeshPushConstants {}

/// Push constants for widget rendering (model, view, projection + color)
#[repr(C)]
#[derive(Copy, Clone)]
struct WidgetPushConstants {
    model: glam::Mat4,       // 64 bytes
    view: glam::Mat4,        // 64 bytes
    projection: glam::Mat4,  // 64 bytes
    color: glam::Vec4,       // 16 bytes
}

unsafe impl bytemuck::Pod for WidgetPushConstants {}
unsafe impl bytemuck::Zeroable for WidgetPushConstants {}

/// Star shader uniform buffer object
#[repr(C)]
#[derive(Copy, Clone)]
struct StarUniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    time: f32,
    star_color: Vec3,
    gamma: f32,
    scale: f32,
    exposure: f32,
    speed_hi: f32,
    speed_low: f32,
    zoom: f32,
    _padding: f32,
}

unsafe impl bytemuck::Pod for StarUniformBufferObject {}
unsafe impl bytemuck::Zeroable for StarUniformBufferObject {}

pub struct VulkanRenderer {
    _entry: Entry,
    instance: ash::Instance,
    debug_utils: Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    graphics_queue: vk::Queue,
    present_queue: vk::Queue,
    swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain_images: Vec<vk::Image>,
    swapchain_image_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    render_pass: vk::RenderPass,
    descriptor_set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    graphics_pipeline: vk::Pipeline,
    wireframe_pipeline: vk::Pipeline,  // Wireframe rendering pipeline
    // Gizmo - store all three mesh types
    gizmo_translate_mesh: Mesh,
    gizmo_rotate_mesh: Mesh,
    gizmo_scale_mesh: Mesh,
    gizmo_vertex_buffer: vk::Buffer,
    gizmo_vertex_buffer_memory: vk::DeviceMemory,
    gizmo_vertex_buffer_size: vk::DeviceSize,
    gizmo_index_buffer: vk::Buffer,
    gizmo_index_buffer_memory: vk::DeviceMemory,
    gizmo_index_buffer_size: vk::DeviceSize,
    gizmo_descriptor_set_layout: vk::DescriptorSetLayout,
    gizmo_pipeline_layout: vk::PipelineLayout,
    gizmo_pipeline: vk::Pipeline,
    gizmo_uniform_buffers: Vec<vk::Buffer>,
    gizmo_uniform_buffers_memory: Vec<vk::DeviceMemory>,
    gizmo_descriptor_sets: Vec<vk::DescriptorSet>,
    framebuffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    image_available_semaphores: Vec<vk::Semaphore>,
    render_finished_semaphores: Vec<vk::Semaphore>,
    in_flight_fences: Vec<vk::Fence>,
    images_in_flight: Vec<vk::Fence>,
    current_frame: usize,
    frame_count: u64,
    fps_frame_count: u64,
    last_time: std::time::Instant,
    last_frame_time: std::time::Instant,
    window: Window,
    // Mesh registry for cube objects
    cube_mesh: Mesh,
    cube_vertex_buffer: vk::Buffer,
    cube_vertex_buffer_memory: vk::DeviceMemory,
    cube_index_buffer: vk::Buffer,
    cube_index_buffer_memory: vk::DeviceMemory,
    // Custom mesh storage (path -> (mesh, vertex_buffer, index_buffer, memories))
    custom_meshes: std::collections::HashMap<String, (Mesh, vk::Buffer, vk::DeviceMemory, vk::Buffer, vk::DeviceMemory)>,
    // Directional light visualization
    dir_light_mesh: Mesh,
    dir_light_vertex_buffer: vk::Buffer,
    dir_light_vertex_buffer_memory: vk::DeviceMemory,
    dir_light_index_buffer: vk::Buffer,
    dir_light_index_buffer_memory: vk::DeviceMemory,
    // Legacy fields for compatibility
    mesh: Mesh,
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    index_buffer: vk::Buffer,
    index_buffer_memory: vk::DeviceMemory,
    uniform_buffers: Vec<vk::Buffer>,
    uniform_buffers_memory: Vec<vk::DeviceMemory>,
    descriptor_pool: vk::DescriptorPool,
    descriptor_sets: Vec<vk::DescriptorSet>,
    depth_image: vk::Image,
    depth_image_memory: vk::DeviceMemory,
    depth_image_view: vk::ImageView,
    depth_sampler: vk::Sampler,
    // SSAO resources
    ssao_image: vk::Image,
    ssao_image_memory: vk::DeviceMemory,
    ssao_image_view: vk::ImageView,
    ssao_blur_image: vk::Image,
    ssao_blur_image_memory: vk::DeviceMemory,
    ssao_blur_image_view: vk::ImageView,
    ssao_sampler: vk::Sampler,
    ssao_render_pass: vk::RenderPass,
    ssao_framebuffer: vk::Framebuffer,
    ssao_descriptor_set_layout: vk::DescriptorSetLayout,
    ssao_pipeline_layout: vk::PipelineLayout,
    ssao_pipeline: vk::Pipeline,
    ssao_uniform_buffers: Vec<vk::Buffer>,
    ssao_uniform_buffers_memory: Vec<vk::DeviceMemory>,
    ssao_descriptor_pool: vk::DescriptorPool,
    ssao_descriptor_sets: Vec<vk::DescriptorSet>,
    ssao_blur_render_pass: vk::RenderPass,
    ssao_blur_framebuffer: vk::Framebuffer,
    ssao_blur_descriptor_set_layout: vk::DescriptorSetLayout,
    ssao_blur_pipeline_layout: vk::PipelineLayout,
    ssao_blur_pipeline: vk::Pipeline,
    ssao_blur_descriptor_pool: vk::DescriptorPool,
    ssao_blur_descriptor_sets: Vec<vk::DescriptorSet>,
    // Intermediate texture for horizontal blur pass (vertical pass reads from this)
    ssao_blur_intermediate_image: vk::Image,
    ssao_blur_intermediate_image_memory: vk::DeviceMemory,
    ssao_blur_intermediate_image_view: vk::ImageView,
    ssao_blur_horizontal_render_pass: vk::RenderPass,
    ssao_blur_horizontal_framebuffer: vk::Framebuffer,
    ssao_blur_horizontal_descriptor_pool: vk::DescriptorPool,
    ssao_blur_horizontal_descriptor_sets: Vec<vk::DescriptorSet>,
    directional_light: DirectionalLight,
    point_lights: Vec<PointLight>,
    // ImGui
    imgui_context: Context,
    imgui_renderer: ImGuiRenderer,
    imgui_platform: imgui_winit_support::WinitPlatform,
    // Render pass plugin system
    render_passes: crate::core::RenderPassRegistry,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct UniformBufferObject {
    view: Mat4,
    proj: Mat4,
    view_pos: Vec3,
    _padding: f32,
    dir_light_direction: Vec3,
    _padding2: f32,
    dir_light_color: Vec3,
    dir_light_intensity: f32,
    dir_light_shadow_color: Vec3,
    star_density: f32,
    star_brightness: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
    nebula_primary_color: Vec3,
    nebula_intensity: f32,
    nebula_secondary_color: Vec3,
    background_brightness: f32,
    point_light_count: u32,
    ssao_enabled: u32,
    _padding3: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct GizmoUniformBufferObject {
    model: Mat4,
    view: Mat4,
    proj: Mat4,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct PointLightData {
    position: Vec3,
    _padding: f32,
    color: Vec3,
    intensity: f32,
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct SSAOUniformBufferObject {
    proj: Mat4,             // 64 bytes, offset 0
    ssao_radius: f32,       // 4 bytes, offset 64
    ssao_bias: f32,         // 4 bytes, offset 68
    ssao_power: f32,        // 4 bytes, offset 72
    ssao_kernel_size: u32,  // 4 bytes, offset 76
    // Total: 80 bytes (needs padding to 16-byte alignment)
}

const MAX_POINT_LIGHTS: usize = 4;

impl VulkanRenderer {
    pub fn new(window: Window) -> anyhow::Result<Self> {
        unsafe {
            let entry = Entry::load()?;
            
            // Create instance
            let app_name = CString::new("Tribal Engine")?;
            let engine_name = CString::new("Tribal")?;
            let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(vk::make_api_version(0, 1, 0, 0))
            .engine_name(&engine_name)
            .engine_version(vk::make_api_version(0, 1, 0, 0))
            .api_version(vk::API_VERSION_1_2);
            
            let extension_names = ash_window::enumerate_required_extensions(
                window.display_handle()?.as_raw()
            )?;
            
            let mut extensions: Vec<*const i8> = extension_names.to_vec();
            
            #[cfg(debug_assertions)]
            extensions.push(ash::ext::debug_utils::NAME.as_ptr());
            
            #[cfg(debug_assertions)]
            let layer_names = [CString::new("VK_LAYER_KHRONOS_validation")?];
            #[cfg(debug_assertions)]
            let layer_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|name| name.as_ptr())
            .collect();
            
            let mut create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_extension_names(&extensions);
            
            #[cfg(debug_assertions)]
            {
                create_info = create_info.enabled_layer_names(&layer_names_raw);
            }
            
            let instance = entry.create_instance(&create_info, None)?;
            
            // Setup debug messenger
            let debug_utils = Self::setup_debug_messenger(&entry, &instance)?;
            
            // Create surface
            let surface = ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )?;
            let surface_loader = ash::khr::surface::Instance::new(&entry, &instance);
            
            // Pick physical device
            let physical_device = Self::pick_physical_device(&instance, &surface_loader, surface)?;
            
            // Create logical device
            let (device, graphics_queue, present_queue) =
            Self::create_logical_device(&instance, physical_device, &surface_loader, surface)?;
            
            // Create swapchain
            let swapchain_loader = ash::khr::swapchain::Device::new(&instance, &device);
            let (swapchain, swapchain_images, swapchain_format, swapchain_extent) =
            Self::create_swapchain(
                &window,
                &instance,
                physical_device,
                &device,
                &surface_loader,
                surface,
                &swapchain_loader,
            )?;
            
            // Create image views
            let swapchain_image_views =
            Self::create_image_views(&device, &swapchain_images, swapchain_format)?;
            
            // Create render pass
            let render_pass = Self::create_render_pass(&device, swapchain_format)?;
            
            // Create descriptor set layout
            let descriptor_set_layout = Self::create_descriptor_set_layout(&device)?;
            
            // Create graphics pipeline
            let (pipeline_layout, graphics_pipeline) =
            Self::create_graphics_pipeline(&device, swapchain_extent, render_pass, descriptor_set_layout)?;

            // Create wireframe pipeline (reuses same pipeline layout)
            let wireframe_pipeline = Self::create_wireframe_pipeline(&device, swapchain_extent, render_pass, pipeline_layout)?;

            // Create depth resources
            let (depth_image, depth_image_memory, depth_image_view) = Self::create_depth_resources(
                &instance,
                physical_device,
                &device,
                swapchain_extent,
            )?;

            // Create depth sampler for nebula
            let depth_sampler = Self::create_depth_sampler(&device)?;

            // Create SSAO resources
            let (ssao_image, ssao_image_memory, ssao_image_view) = Self::create_ssao_image(
                &instance,
                physical_device,
                &device,
                swapchain_extent,
            )?;
            let (ssao_blur_image, ssao_blur_image_memory, ssao_blur_image_view) = Self::create_ssao_image(
                &instance,
                physical_device,
                &device,
                swapchain_extent,
            )?;
            // Intermediate texture for horizontal blur pass
            let (ssao_blur_intermediate_image, ssao_blur_intermediate_image_memory, ssao_blur_intermediate_image_view) = Self::create_ssao_image(
                &instance,
                physical_device,
                &device,
                swapchain_extent,
            )?;
            let ssao_sampler = Self::create_ssao_sampler(&device)?;

            // Create SSAO render passes and pipelines
            let ssao_render_pass = Self::create_ssao_render_pass(&device)?;
            let ssao_blur_render_pass = Self::create_ssao_render_pass(&device)?; // Vertical blur pass
            let ssao_blur_horizontal_render_pass = Self::create_ssao_render_pass(&device)?; // Horizontal blur pass

            let ssao_framebuffer = Self::create_ssao_framebuffer(
                &device,
                ssao_render_pass,
                ssao_image_view,
                swapchain_extent,
            )?;
            // Horizontal blur writes to intermediate texture
            let ssao_blur_horizontal_framebuffer = Self::create_ssao_framebuffer(
                &device,
                ssao_blur_horizontal_render_pass,
                ssao_blur_intermediate_image_view,
                swapchain_extent,
            )?;
            // Vertical blur reads from intermediate and writes to final blur texture
            let ssao_blur_framebuffer = Self::create_ssao_framebuffer(
                &device,
                ssao_blur_render_pass,
                ssao_blur_image_view,
                swapchain_extent,
            )?;

            let ssao_descriptor_set_layout = Self::create_ssao_descriptor_set_layout(&device)?;
            let ssao_blur_descriptor_set_layout = Self::create_ssao_blur_descriptor_set_layout(&device)?;

            let (ssao_pipeline_layout, ssao_pipeline) = Self::create_ssao_pipeline(
                &device,
                swapchain_extent,
                ssao_render_pass,
                ssao_descriptor_set_layout,
            )?;
            let (ssao_blur_pipeline_layout, ssao_blur_pipeline) = Self::create_ssao_blur_pipeline(
                &device,
                swapchain_extent,
                ssao_blur_render_pass,
                ssao_blur_descriptor_set_layout,
            )?;

            // Create SSAO uniform buffers (reuse same UBO as main rendering)
            let (ssao_uniform_buffers, ssao_uniform_buffers_memory) = Self::create_uniform_buffers(
                &instance,
                physical_device,
                &device,
                MAX_FRAMES_IN_FLIGHT,
            )?;

            // Create SSAO descriptor pools and sets
            let ssao_descriptor_pool = Self::create_ssao_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
            let ssao_descriptor_sets = Self::create_ssao_descriptor_sets(
                &device,
                ssao_descriptor_pool,
                ssao_descriptor_set_layout,
                &ssao_uniform_buffers,
                depth_image_view,
                depth_sampler,
                MAX_FRAMES_IN_FLIGHT,
            )?;

            // Horizontal blur: reads from SSAO, writes to intermediate
            let ssao_blur_horizontal_descriptor_pool = Self::create_ssao_blur_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
            let ssao_blur_horizontal_descriptor_sets = Self::create_ssao_blur_descriptor_sets(
                &device,
                ssao_blur_horizontal_descriptor_pool,
                ssao_blur_descriptor_set_layout,
                ssao_image_view,
                ssao_sampler,
                depth_image_view,
                depth_sampler,
                MAX_FRAMES_IN_FLIGHT,
            )?;

            // Vertical blur: reads from intermediate, writes to final
            let ssao_blur_descriptor_pool = Self::create_ssao_blur_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
            let ssao_blur_descriptor_sets = Self::create_ssao_blur_descriptor_sets(
                &device,
                ssao_blur_descriptor_pool,
                ssao_blur_descriptor_set_layout,
                ssao_blur_intermediate_image_view,
                ssao_sampler,
                depth_image_view,
                depth_sampler,
                MAX_FRAMES_IN_FLIGHT,
            )?;

            // Create framebuffers
            let framebuffers = Self::create_framebuffers(
                &device,
                &swapchain_image_views,
                depth_image_view,
                render_pass,
                swapchain_extent,
            )?;
            
            // Create command pool
            let command_pool = Self::create_command_pool(&instance, physical_device, &device, &surface_loader, surface)?;
            
            // Create cube mesh (will be used for all cube objects)
            let cube_mesh = Mesh::create_cube();

            // Create cube vertex buffer
            let (cube_vertex_buffer, cube_vertex_buffer_memory) = Self::create_vertex_buffer(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                &cube_mesh.vertices,
            )?;

            // Create cube index buffer
            let (cube_index_buffer, cube_index_buffer_memory) = Self::create_index_buffer(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                &cube_mesh.indices,
            )?;

            // Legacy: keep old mesh references for compatibility
            let mesh = cube_mesh.clone();
            let vertex_buffer = cube_vertex_buffer;
            let vertex_buffer_memory = cube_vertex_buffer_memory;
            let index_buffer = cube_index_buffer;
            let index_buffer_memory = cube_index_buffer_memory;
            
            // Create uniform buffers
            let (uniform_buffers, uniform_buffers_memory) = Self::create_uniform_buffers(
                &instance,
                physical_device,
                &device,
                MAX_FRAMES_IN_FLIGHT,
            )?;
            
            // Create descriptor pool and sets
            let descriptor_pool = Self::create_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
            let descriptor_sets = Self::create_descriptor_sets(
                &device,
                descriptor_pool,
                descriptor_set_layout,
                &uniform_buffers,
                ssao_blur_image_view,
                ssao_sampler,
                MAX_FRAMES_IN_FLIGHT,
            )?;
            // Create all three gizmo meshes
            let (translate_vertices, translate_indices) = GizmoMesh::generate_translate_arrows();
            let gizmo_translate_mesh = Mesh {
                vertices: translate_vertices,
                indices: translate_indices,
            };

            let (rotate_vertices, rotate_indices) = GizmoMesh::generate_rotate_circles();
            let gizmo_rotate_mesh = Mesh {
                vertices: rotate_vertices,
                indices: rotate_indices,
            };

            let (scale_vertices, scale_indices) = GizmoMesh::generate_scale_boxes();
            let gizmo_scale_mesh = Mesh {
                vertices: scale_vertices,
                indices: scale_indices,
            };

            // Use translate mesh for initial buffer creation (largest mesh will be used)
            let max_vertices = gizmo_translate_mesh.vertices.len()
                .max(gizmo_rotate_mesh.vertices.len())
                .max(gizmo_scale_mesh.vertices.len());
            let max_indices = gizmo_translate_mesh.indices.len()
                .max(gizmo_rotate_mesh.indices.len())
                .max(gizmo_scale_mesh.indices.len());

            // Create buffers large enough for the largest mesh
            let zero_vertex = Vertex {
                position: Vec3::ZERO,
                normal: Vec3::Y,
                uv: Vec2::ZERO,
            };
            let mut temp_vertices = gizmo_translate_mesh.vertices.clone();
            temp_vertices.resize(max_vertices, zero_vertex);
            let mut temp_indices = gizmo_translate_mesh.indices.clone();
            temp_indices.resize(max_indices, 0);

            // Create HOST_VISIBLE buffers for gizmos since we update them dynamically
            let gizmo_vertex_buffer_size = (std::mem::size_of::<Vertex>() * max_vertices) as vk::DeviceSize;
            let (gizmo_vertex_buffer, gizmo_vertex_buffer_memory) = Self::create_buffer(
                &instance,
                physical_device,
                &device,
                gizmo_vertex_buffer_size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            let gizmo_index_buffer_size = (std::mem::size_of::<u32>() * max_indices) as vk::DeviceSize;
            let (gizmo_index_buffer, gizmo_index_buffer_memory) = Self::create_buffer(
                &instance,
                physical_device,
                &device,
                gizmo_index_buffer_size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;

            // Initialize with translate mesh data
            let vertex_data = device.map_memory(
                gizmo_vertex_buffer_memory,
                0,
                gizmo_vertex_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                temp_vertices.as_ptr(),
                vertex_data as *mut Vertex,
                temp_vertices.len(),
            );
            device.unmap_memory(gizmo_vertex_buffer_memory);

            let index_data = device.map_memory(
                gizmo_index_buffer_memory,
                0,
                gizmo_index_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                temp_indices.as_ptr(),
                index_data as *mut u32,
                temp_indices.len(),
            );
            device.unmap_memory(gizmo_index_buffer_memory);

            let gizmo_descriptor_set_layout = Self::create_descriptor_set_layout(&device)?;
            let (gizmo_pipeline_layout, gizmo_pipeline) =
            Self::create_gizmo_pipeline(&device, swapchain_extent, render_pass, gizmo_descriptor_set_layout)?;

            let (gizmo_uniform_buffers, gizmo_uniform_buffers_memory) = Self::create_gizmo_uniform_buffers(
                &instance,
                physical_device,
                &device,
                MAX_FRAMES_IN_FLIGHT,
            )?;

            let gizmo_descriptor_pool = Self::create_descriptor_pool(&device, MAX_FRAMES_IN_FLIGHT)?;
            let gizmo_descriptor_sets = Self::create_descriptor_sets(
                &device,
                gizmo_descriptor_pool,
                gizmo_descriptor_set_layout,
                &gizmo_uniform_buffers,
                ssao_blur_image_view,
                ssao_sampler,
                MAX_FRAMES_IN_FLIGHT,
            )?;


            // Create command buffers
            let command_buffers = Self::create_command_buffers(&device, command_pool, MAX_FRAMES_IN_FLIGHT)?;
            
            // Create sync objects
            let (image_available_semaphores, render_finished_semaphores, in_flight_fences) =
            Self::create_sync_objects(&device, MAX_FRAMES_IN_FLIGHT)?;
            
            // Initialize lighting
            let directional_light = DirectionalLight {
                direction: Vec3::new(-0.3, -1.0, -0.3).normalize(),
                color: Vec3::new(1.0, 0.95, 0.9),
                intensity: 1.0,
                shadow_color: Vec3::new(0.1, 0.1, 0.15),
            };
            
            let point_lights = vec![
            PointLight {
                position: Vec3::new(2.0, 2.0, 2.0),
                color: Vec3::new(1.0, 0.3, 0.3),
                intensity: 5.0,
            },
            PointLight {
                position: Vec3::new(-2.0, 2.0, -2.0),
                color: Vec3::new(0.3, 0.3, 1.0),
                intensity: 5.0,
            },
            ];
            
            // Initialize images_in_flight with null fences
            let images_in_flight = vec![vk::Fence::null(); swapchain_images.len()];
            
            // Initialize ImGui
            let mut imgui_context = Context::create();
            imgui_context.set_ini_filename(None);
            
            let mut imgui_platform = imgui_winit_support::WinitPlatform::init(&mut imgui_context);
            imgui_platform.attach_window(
                imgui_context.io_mut(),
                &window,
                imgui_winit_support::HiDpiMode::Default,
            );

            // Create directional light visualization mesh
            let dir_light_mesh = Mesh::create_directional_light_viz();
            let (dir_light_vertex_buffer, dir_light_vertex_buffer_memory) = Self::create_vertex_buffer(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                &dir_light_mesh.vertices,
            )?;
            let (dir_light_index_buffer, dir_light_index_buffer_memory) = Self::create_index_buffer(
                &instance,
                physical_device,
                &device,
                command_pool,
                graphics_queue,
                &dir_light_mesh.indices,
            )?;

            // Set up ImGui fonts first
            imgui_context.fonts().add_font(&[imgui::FontSource::DefaultFontData {
                config: Some(imgui::FontConfig {
                    size_pixels: 18.0,
                    ..Default::default()
                }),
            }]);
            
            let imgui_renderer = ImGuiRenderer::new(
                &mut imgui_context,
                &device,
                &instance,
                physical_device,
                render_pass,
                command_pool,
                graphics_queue,
                swapchain_extent,
            )?;

            // Initialize render pass plugin system
            let mut render_passes = crate::core::RenderPassRegistry::new();

            // Register passes
            render_passes.register(Box::new(crate::core::passes::SkyboxPass::new()));
            render_passes.register(Box::new(crate::core::passes::NebulaPass::new()));
            render_passes.register(Box::new(crate::core::passes::MeshPass::new()));
            render_passes.register(Box::new(crate::core::passes::StarPass::new(MAX_FRAMES_IN_FLIGHT)));

            // Initialize all passes
            let ctx = crate::core::RenderContext {
                device: &device,
                instance: &instance,
                physical_device,
                command_pool,
                graphics_queue,
                extent: swapchain_extent,
                depth_image_view: Some(depth_image_view),
                depth_sampler: Some(depth_sampler),
                mesh_pipeline: Some(graphics_pipeline),
                mesh_pipeline_layout: Some(pipeline_layout),
                mesh_descriptor_sets: Some(&descriptor_sets),
                custom_meshes: None,  // No meshes loaded yet at initialization
            };
            render_passes.initialize_all(&ctx, render_pass, swapchain_extent)?;

            Ok(Self {
                _entry: entry,
                instance,
                debug_utils,
                surface,
                surface_loader,
                physical_device,
                device,
                graphics_queue,
                present_queue,
                swapchain,
                swapchain_loader,
                swapchain_images,
                swapchain_image_views,
                swapchain_format,
                swapchain_extent,
                render_pass,
                descriptor_set_layout,
                pipeline_layout,
                graphics_pipeline,
                wireframe_pipeline,
                gizmo_translate_mesh,
                gizmo_rotate_mesh,
                gizmo_scale_mesh,
                gizmo_vertex_buffer,
                gizmo_vertex_buffer_memory,
                gizmo_vertex_buffer_size,
                gizmo_index_buffer,
                gizmo_index_buffer_memory,
                gizmo_index_buffer_size,
                gizmo_descriptor_set_layout,
                gizmo_pipeline_layout,
                gizmo_pipeline,
                gizmo_uniform_buffers,
                gizmo_uniform_buffers_memory,
                gizmo_descriptor_sets,
                framebuffers,
                command_pool,
                command_buffers,
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                images_in_flight,
                current_frame: 0,
                frame_count: 0,
                fps_frame_count: 0,
                last_time: std::time::Instant::now(),
                last_frame_time: std::time::Instant::now(),
                window,
                cube_mesh,
                cube_vertex_buffer,
                cube_vertex_buffer_memory,
                cube_index_buffer,
                cube_index_buffer_memory,
                custom_meshes: std::collections::HashMap::new(),
                dir_light_mesh,
                dir_light_vertex_buffer,
                dir_light_vertex_buffer_memory,
                dir_light_index_buffer,
                dir_light_index_buffer_memory,
                mesh,
                vertex_buffer,
                vertex_buffer_memory,
                index_buffer,
                index_buffer_memory,
                uniform_buffers,
                uniform_buffers_memory,
                descriptor_pool,
                descriptor_sets,
                depth_image,
                depth_image_memory,
                depth_image_view,
                depth_sampler,
                ssao_image,
                ssao_image_memory,
                ssao_image_view,
                ssao_blur_image,
                ssao_blur_image_memory,
                ssao_blur_image_view,
                ssao_sampler,
                ssao_render_pass,
                ssao_framebuffer,
                ssao_descriptor_set_layout,
                ssao_pipeline_layout,
                ssao_pipeline,
                ssao_uniform_buffers,
                ssao_uniform_buffers_memory,
                ssao_descriptor_pool,
                ssao_descriptor_sets,
                ssao_blur_render_pass,
                ssao_blur_framebuffer,
                ssao_blur_descriptor_set_layout,
                ssao_blur_pipeline_layout,
                ssao_blur_pipeline,
                ssao_blur_descriptor_pool,
                ssao_blur_descriptor_sets,
                ssao_blur_intermediate_image,
                ssao_blur_intermediate_image_memory,
                ssao_blur_intermediate_image_view,
                ssao_blur_horizontal_render_pass,
                ssao_blur_horizontal_framebuffer,
                ssao_blur_horizontal_descriptor_pool,
                ssao_blur_horizontal_descriptor_sets,
                directional_light,
                point_lights,
                imgui_context,
                imgui_renderer,
                imgui_platform,
                render_passes,
            })
        }
    }
    
    unsafe fn setup_debug_messenger(
        entry: &Entry,
        instance: &ash::Instance,
    ) -> anyhow::Result<Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>> {
        #[cfg(debug_assertions)]
        {
            let debug_utils_loader = ash::ext::debug_utils::Instance::new(entry, instance);
            
            let debug_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
            .message_severity(
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            )
            .message_type(
                vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
                | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE,
            )
            .pfn_user_callback(Some(vulkan_debug_callback));
            
            let debug_messenger = debug_utils_loader.create_debug_utils_messenger(&debug_info, None)?;
            Ok(Some((debug_utils_loader, debug_messenger)))
        }
        
        #[cfg(not(debug_assertions))]
        Ok(None)
    }
    
    unsafe fn pick_physical_device(
        instance: &ash::Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<vk::PhysicalDevice> {
        let devices = instance.enumerate_physical_devices()?;
        
        for device in devices {
            if Self::is_device_suitable(instance, device, surface_loader, surface)? {
                return Ok(device);
            }
        }
        
        anyhow::bail!("No suitable GPU found")
    }
    
    unsafe fn is_device_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<bool> {
        let props = instance.get_physical_device_properties(device);
        let features = instance.get_physical_device_features(device);
        
        let queue_families = Self::find_queue_families(instance, device, surface_loader, surface)?;
        
        let extensions_supported = Self::check_device_extension_support(instance, device)?;
        
        let swapchain_adequate = if extensions_supported {
            let formats = surface_loader.get_physical_device_surface_formats(device, surface)?;
            let present_modes = surface_loader.get_physical_device_surface_present_modes(device, surface)?;
            !formats.is_empty() && !present_modes.is_empty()
        } else {
            false
        };
        
        Ok(props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU
            && features.geometry_shader == vk::TRUE
            && queue_families.is_complete()
            && extensions_supported
            && swapchain_adequate)
        }
        
        unsafe fn find_queue_families(
            instance: &ash::Instance,
            device: vk::PhysicalDevice,
            surface_loader: &ash::khr::surface::Instance,
            surface: vk::SurfaceKHR,
        ) -> anyhow::Result<QueueFamilyIndices> {
            let queue_families = instance.get_physical_device_queue_family_properties(device);
            
            let mut indices = QueueFamilyIndices::default();
            
            for (i, queue_family) in queue_families.iter().enumerate() {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    indices.graphics_family = Some(i as u32);
                }
                
                let present_support = surface_loader.get_physical_device_surface_support(
                    device,
                    i as u32,
                    surface,
                )?;
                
                if present_support {
                    indices.present_family = Some(i as u32);
                }
                
                if indices.is_complete() {
                    break;
                }
            }
            
            Ok(indices)
        }
        
        unsafe fn check_device_extension_support(
            instance: &ash::Instance,
            device: vk::PhysicalDevice,
        ) -> anyhow::Result<bool> {
            let available_extensions = instance.enumerate_device_extension_properties(device)?;
            
            let required_extensions = [ash::khr::swapchain::NAME];
            
            for required in required_extensions {
                let found = available_extensions.iter().any(|ext| {
                    let name = CStr::from_ptr(ext.extension_name.as_ptr());
                    name == required
                });
                
                if !found {
                    return Ok(false);
                }
            }
            
            Ok(true)
        }
        
        unsafe fn create_logical_device(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            surface_loader: &ash::khr::surface::Instance,
            surface: vk::SurfaceKHR,
        ) -> anyhow::Result<(ash::Device, vk::Queue, vk::Queue)> {
            let indices = Self::find_queue_families(instance, physical_device, surface_loader, surface)?;
            
            let mut unique_queue_families = std::collections::HashSet::new();
            unique_queue_families.insert(indices.graphics_family.unwrap());
            unique_queue_families.insert(indices.present_family.unwrap());
            
            // Create priority arrays with proper lifetime
            let queue_priorities: Vec<Vec<f32>> = unique_queue_families
            .iter()
            .map(|_| vec![1.0])
            .collect();
            
            let queue_create_infos: Vec<vk::DeviceQueueCreateInfo> = unique_queue_families
            .iter()
            .enumerate()
            .map(|(i, &queue_family)| {
                vk::DeviceQueueCreateInfo::default()
                .queue_family_index(queue_family)
                .queue_priorities(&queue_priorities[i])
            })
            .collect();
            
            let device_features = vk::PhysicalDeviceFeatures::default();
            
            let device_extensions = [ash::khr::swapchain::NAME.as_ptr()];
            
            let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_features(&device_features)
            .enabled_extension_names(&device_extensions);
            
            let device = instance.create_device(physical_device, &device_create_info, None)?;
            
            let graphics_queue = device.get_device_queue(indices.graphics_family.unwrap(), 0);
            let present_queue = device.get_device_queue(indices.present_family.unwrap(), 0);
            
            Ok((device, graphics_queue, present_queue))
        }
        
        unsafe fn create_swapchain(
            window: &Window,
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            surface_loader: &ash::khr::surface::Instance,
            surface: vk::SurfaceKHR,
            swapchain_loader: &ash::khr::swapchain::Device,
        ) -> anyhow::Result<(vk::SwapchainKHR, Vec<vk::Image>, vk::Format, vk::Extent2D)> {
            let capabilities = surface_loader.get_physical_device_surface_capabilities(physical_device, surface)?;
            let formats = surface_loader.get_physical_device_surface_formats(physical_device, surface)?;
            let present_modes = surface_loader.get_physical_device_surface_present_modes(physical_device, surface)?;
            
            let surface_format = formats
            .iter()
            .find(|f| {
                f.format == vk::Format::B8G8R8A8_SRGB
                && f.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
            })
            .unwrap_or(&formats[0]);
            
            let present_mode = present_modes
            .iter()
            .copied()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
            
            let extent = {
                if capabilities.current_extent.width != u32::MAX {
                    capabilities.current_extent
                } else {
                    let size = window.inner_size();
                    vk::Extent2D {
                        width: size.width.clamp(
                            capabilities.min_image_extent.width,
                            capabilities.max_image_extent.width,
                        ),
                        height: size.height.clamp(
                            capabilities.min_image_extent.height,
                            capabilities.max_image_extent.height,
                        ),
                    }
                }
            };
            
            let image_count = (capabilities.min_image_count + 1).min(
                if capabilities.max_image_count > 0 {
                    capabilities.max_image_count
                } else {
                    u32::MAX
                },
            );
            
            let indices = Self::find_queue_families(instance, physical_device, surface_loader, surface)?;
            let queue_family_indices = [
            indices.graphics_family.unwrap(),
            indices.present_family.unwrap(),
            ];
            
            let (image_sharing_mode, queue_family_index_count, p_queue_family_indices) =
            if indices.graphics_family != indices.present_family {
                (vk::SharingMode::CONCURRENT, 2, queue_family_indices.as_ptr())
            } else {
                (vk::SharingMode::EXCLUSIVE, 0, std::ptr::null())
            };
            
            let create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(image_sharing_mode)
            .queue_family_indices(&queue_family_indices[..queue_family_index_count as usize])
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
            
            let swapchain = swapchain_loader.create_swapchain(&create_info, None)?;
            let images = swapchain_loader.get_swapchain_images(swapchain)?;
            
            Ok((swapchain, images, surface_format.format, extent))
        }
        
        unsafe fn create_image_views(
            device: &ash::Device,
            images: &[vk::Image],
            format: vk::Format,
        ) -> anyhow::Result<Vec<vk::ImageView>> {
            images
            .iter()
            .map(|&image| {
                let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
                
                device.create_image_view(&create_info, None).map_err(|e| anyhow::anyhow!("Failed to create image view: {}", e))
            })
            .collect()
        }
        
        unsafe fn create_render_pass(
            device: &ash::Device,
            format: vk::Format,
        ) -> anyhow::Result<vk::RenderPass> {
            let color_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
            
            let depth_attachment = vk::AttachmentDescription::default()
            .format(vk::Format::D32_SFLOAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            
            let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
            
            let depth_attachment_ref = vk::AttachmentReference::default()
            .attachment(1)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
            
            let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref))
            .depth_stencil_attachment(&depth_attachment_ref);
            
            let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );
            
            let attachments = [color_attachment, depth_attachment];
            let create_info = vk::RenderPassCreateInfo::default()
            .attachments(&attachments)
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));
            
            Ok(device.create_render_pass(&create_info, None)?)
        }
        
        unsafe fn create_descriptor_set_layout(device: &ash::Device) -> anyhow::Result<vk::DescriptorSetLayout> {
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

            // Binding 1: SSAO texture sampler
            let ssao_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            let bindings = [ubo_binding, ssao_binding];
            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
            .bindings(&bindings);

            Ok(device.create_descriptor_set_layout(&create_info, None)?)
        }

        unsafe fn create_ssao_descriptor_set_layout(device: &ash::Device) -> anyhow::Result<vk::DescriptorSetLayout> {
            // Binding 0: UBO for view/proj matrices and SSAO parameters
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            // Binding 1: Depth texture sampler
            let depth_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            let bindings = [ubo_binding, depth_binding];
            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);

            Ok(device.create_descriptor_set_layout(&create_info, None)?)
        }

        unsafe fn create_ssao_blur_descriptor_set_layout(device: &ash::Device) -> anyhow::Result<vk::DescriptorSetLayout> {
            // Binding 0: SSAO texture sampler input
            let ssao_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            // Binding 1: Depth texture for edge detection
            let depth_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(1)
                .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::FRAGMENT);

            let bindings = [ssao_binding, depth_binding];
            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);

            Ok(device.create_descriptor_set_layout(&create_info, None)?)
        }

        unsafe fn create_star_descriptor_set_layout(device: &ash::Device) -> anyhow::Result<vk::DescriptorSetLayout> {
            // Binding 0: Star uniform buffer (model, view, proj, star properties)
            let ubo_binding = vk::DescriptorSetLayoutBinding::default()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT);

            let bindings = [ubo_binding];
            let create_info = vk::DescriptorSetLayoutCreateInfo::default()
                .bindings(&bindings);

            Ok(device.create_descriptor_set_layout(&create_info, None)?)
        }

        unsafe fn create_graphics_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            render_pass: vk::RenderPass,
            descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
            // Shader code will be compiled from GLSL
            let vert_shader_code = include_bytes!("../../shaders/mesh.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/mesh.frag.spv");
            
            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;
            
            let entry_point = CString::new("main")?;
            
            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&entry_point);
            
            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&entry_point);
            
            let shader_stages = [vert_stage_info, frag_stage_info];
            
            let binding_description = Vertex::get_binding_description();
            let attribute_descriptions = Vertex::get_attribute_descriptions();
            
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);
            
            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
            
            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };
            
            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };
            
            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));
            
            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);
            
            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

            let set_layouts = [descriptor_set_layout];

            // Define push constant range for model matrix + material properties
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX | vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(std::mem::size_of::<MeshPushConstants>() as u32);

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts)
                .push_constant_ranges(std::slice::from_ref(&push_constant_range));

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;
            
            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);
            
            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create graphics pipeline: {:?}", e.1))?;
            
            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);
            
            Ok((pipeline_layout, pipelines[0]))
        }

        unsafe fn create_wireframe_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            render_pass: vk::RenderPass,
            pipeline_layout: vk::PipelineLayout, // Reuse same layout as graphics pipeline
        ) -> anyhow::Result<vk::Pipeline> {
            let vert_shader_code = include_bytes!("../../shaders/wireframe.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/wireframe.frag.spv");

            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

            let entry_point = CString::new("main")?;

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            let binding_description = Vertex::get_binding_description();
            let attribute_descriptions = Vertex::get_attribute_descriptions();

            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
                .vertex_attribute_descriptions(&attribute_descriptions);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(std::slice::from_ref(&viewport))
                .scissors(std::slice::from_ref(&scissor));

            // WIREFRAME MODE - this is the key difference!
            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::LINE)  // LINE mode for wireframe!
                .line_width(1.5)  // Slightly thicker lines
                .cull_mode(vk::CullModeFlags::NONE)  // Don't cull for wireframe
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // Wireframe should write depth but at a slight offset to avoid z-fighting
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(false) // Don't write depth for wireframe overlay
                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
                .depth_bounds_test_enable(false)
                .stencil_test_enable(false);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .depth_stencil_state(&depth_stencil)
                .color_blend_state(&color_blending)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0);

            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create wireframe pipeline: {:?}", e.1))?;

            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);

            Ok(pipelines[0])
        }

        unsafe fn create_ssao_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            ssao_render_pass: vk::RenderPass,
            descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
            let vert_shader_code = include_bytes!("../../shaders/ssao.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/ssao.frag.spv");

            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

            let entry_point = CString::new("main")?;

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            // No vertex input - fullscreen triangle generated in vertex shader
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(std::slice::from_ref(&viewport))
                .scissors(std::slice::from_ref(&scissor));

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // No depth test for SSAO pass
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(false)
                .depth_write_enable(false);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::R) // Only write red channel
                .blend_enable(false);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let set_layouts = [descriptor_set_layout];

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts);

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .depth_stencil_state(&depth_stencil)
                .color_blend_state(&color_blending)
                .layout(pipeline_layout)
                .render_pass(ssao_render_pass)
                .subpass(0);

            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create SSAO pipeline: {:?}", e.1))?;

            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);

            Ok((pipeline_layout, pipelines[0]))
        }

        unsafe fn create_ssao_blur_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            blur_render_pass: vk::RenderPass,
            descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
            let vert_shader_code = include_bytes!("../../shaders/ssao_blur.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/ssao_blur.frag.spv");

            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

            let entry_point = CString::new("main")?;

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            // No vertex input - fullscreen triangle
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default();

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(std::slice::from_ref(&viewport))
                .scissors(std::slice::from_ref(&scissor));

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // No depth test for blur pass
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(false)
                .depth_write_enable(false);

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::R)
                .blend_enable(false);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let set_layouts = [descriptor_set_layout];

            // Add push constant for blur direction (int = 4 bytes)
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::FRAGMENT)
                .offset(0)
                .size(std::mem::size_of::<i32>() as u32);

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts)
                .push_constant_ranges(std::slice::from_ref(&push_constant_range));

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .depth_stencil_state(&depth_stencil)
                .color_blend_state(&color_blending)
                .layout(pipeline_layout)
                .render_pass(blur_render_pass)
                .subpass(0);

            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create SSAO blur pipeline: {:?}", e.1))?;

            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);

            Ok((pipeline_layout, pipelines[0]))
        }

        unsafe fn create_shader_module(
            device: &ash::Device,
            code: &[u8],
        ) -> anyhow::Result<vk::ShaderModule> {
            let code_aligned = ash::util::read_spv(&mut std::io::Cursor::new(code))?;
            let create_info = vk::ShaderModuleCreateInfo::default().code(&code_aligned);
            Ok(device.create_shader_module(&create_info, None)?)
        }
        
        unsafe fn create_gizmo_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            render_pass: vk::RenderPass,
            descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
            let vert_shader_code = include_bytes!("../../shaders/gizmo.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/gizmo.frag.spv");

            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

            let entry_point = CString::new("main")?;

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            let binding_desc = Vertex::get_binding_description();
            let attribute_desc = Vertex::get_attribute_descriptions();

            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_desc))
            .vertex_attribute_descriptions(&attribute_desc);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(std::slice::from_ref(&viewport))
            .scissors(std::slice::from_ref(&scissor));

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE) // No culling for gizmo
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
            .sample_shading_enable(false)
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // Enable depth test so rotation rings sort correctly, but use ALWAYS to render on top of scene
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::ALWAYS); // Always pass depth test (render on top), but still write depth

            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .blend_enable(false);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
            .logic_op_enable(false)
            .attachments(std::slice::from_ref(&color_blend_attachment));

            let set_layouts = [descriptor_set_layout];

            // Add push constant for hovered axis
            let push_constant_range = vk::PushConstantRange::default()
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .offset(0)
                .size(std::mem::size_of::<i32>() as u32);

            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
            .set_layouts(&set_layouts)
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multisampling)
            .depth_stencil_state(&depth_stencil)
            .color_blend_state(&color_blending)
            .layout(pipeline_layout)
            .render_pass(render_pass)
            .subpass(0);

            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create gizmo pipeline: {:?}", e.1))?;

            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);

            Ok((pipeline_layout, pipelines[0]))
        }

        unsafe fn create_star_pipeline(
            device: &ash::Device,
            extent: vk::Extent2D,
            render_pass: vk::RenderPass,
            descriptor_set_layout: vk::DescriptorSetLayout,
        ) -> anyhow::Result<(vk::PipelineLayout, vk::Pipeline)> {
            let vert_shader_code = include_bytes!("../../shaders/star.vert.spv");
            let frag_shader_code = include_bytes!("../../shaders/star.frag.spv");

            let vert_shader_module = Self::create_shader_module(device, vert_shader_code)?;
            let frag_shader_module = Self::create_shader_module(device, frag_shader_code)?;

            let entry_point = CString::new("main")?;

            let vert_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vert_shader_module)
                .name(&entry_point);

            let frag_stage_info = vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(frag_shader_module)
                .name(&entry_point);

            let shader_stages = [vert_stage_info, frag_stage_info];

            // Vertex input: position, normal, uv (same as regular meshes)
            let binding_description = vk::VertexInputBindingDescription::default()
                .binding(0)
                .stride(std::mem::size_of::<Vertex>() as u32)
                .input_rate(vk::VertexInputRate::VERTEX);

            let attribute_descriptions = [
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(0)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(0),
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(1)
                    .format(vk::Format::R32G32B32_SFLOAT)
                    .offset(12),
                vk::VertexInputAttributeDescription::default()
                    .binding(0)
                    .location(2)
                    .format(vk::Format::R32G32_SFLOAT)
                    .offset(24),
            ];

            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
                .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
                .vertex_attribute_descriptions(&attribute_descriptions);

            let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false);

            let viewport = vk::Viewport {
                x: 0.0,
                y: 0.0,
                width: extent.width as f32,
                height: extent.height as f32,
                min_depth: 0.0,
                max_depth: 1.0,
            };

            let scissor = vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent,
            };

            let viewport_state = vk::PipelineViewportStateCreateInfo::default()
                .viewports(std::slice::from_ref(&viewport))
                .scissors(std::slice::from_ref(&scissor));

            let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::BACK)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .depth_bias_enable(false);

            let multisampling = vk::PipelineMultisampleStateCreateInfo::default()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);

            // Depth testing for stars with unified projection
            let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS);

            // Additive blending for star glow
            let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
                .color_write_mask(vk::ColorComponentFlags::RGBA)
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ONE)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::ONE)
                .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                .alpha_blend_op(vk::BlendOp::ADD);

            let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
                .logic_op_enable(false)
                .attachments(std::slice::from_ref(&color_blend_attachment));

            let set_layouts = [descriptor_set_layout];
            let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&set_layouts);

            let pipeline_layout = device.create_pipeline_layout(&pipeline_layout_info, None)?;

            let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .depth_stencil_state(&depth_stencil)
                .color_blend_state(&color_blending)
                .layout(pipeline_layout)
                .render_pass(render_pass)
                .subpass(0);

            let pipelines = device.create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            ).map_err(|e| anyhow::anyhow!("Failed to create star pipeline: {:?}", e.1))?;

            device.destroy_shader_module(vert_shader_module, None);
            device.destroy_shader_module(frag_shader_module, None);

            Ok((pipeline_layout, pipelines[0]))
        }


        unsafe fn create_framebuffers(
            device: &ash::Device,
            image_views: &[vk::ImageView],
            depth_image_view: vk::ImageView,
            render_pass: vk::RenderPass,
            extent: vk::Extent2D,
        ) -> anyhow::Result<Vec<vk::Framebuffer>> {
            image_views
            .iter()
            .map(|&image_view| {
                let attachments = [image_view, depth_image_view];
                let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);
                
                device.create_framebuffer(&create_info, None).map_err(|e| anyhow::anyhow!("Failed to create framebuffer: {}", e))
            })
            .collect()
        }
        
        unsafe fn create_command_pool(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            surface_loader: &ash::khr::surface::Instance,
            surface: vk::SurfaceKHR,
        ) -> anyhow::Result<vk::CommandPool> {
            let indices = Self::find_queue_families(instance, physical_device, surface_loader, surface)?;
            
            let create_info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(indices.graphics_family.unwrap());
            
            Ok(device.create_command_pool(&create_info, None)?)
        }
        
        unsafe fn create_vertex_buffer(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            command_pool: vk::CommandPool,
            graphics_queue: vk::Queue,
            vertices: &[Vertex],
        ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
            let buffer_size = (std::mem::size_of::<Vertex>() * vertices.len()) as vk::DeviceSize;
            
            let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            
            let data = device.map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(vertices.as_ptr(), data as *mut Vertex, vertices.len());
            device.unmap_memory(staging_buffer_memory);
            
            let (vertex_buffer, vertex_buffer_memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            
            Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, vertex_buffer, buffer_size)?;
            
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
            
            Ok((vertex_buffer, vertex_buffer_memory))
        }
        
        unsafe fn create_index_buffer(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            command_pool: vk::CommandPool,
            graphics_queue: vk::Queue,
            indices: &[u32],
        ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
            let buffer_size = (std::mem::size_of::<u32>() * indices.len()) as vk::DeviceSize;
            
            let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_SRC,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            )?;
            
            let data = device.map_memory(
                staging_buffer_memory,
                0,
                buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(indices.as_ptr(), data as *mut u32, indices.len());
            device.unmap_memory(staging_buffer_memory);
            
            let (index_buffer, index_buffer_memory) = Self::create_buffer(
                instance,
                physical_device,
                device,
                buffer_size,
                vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?;
            
            Self::copy_buffer(device, command_pool, graphics_queue, staging_buffer, index_buffer, buffer_size)?;
            
            device.destroy_buffer(staging_buffer, None);
            device.free_memory(staging_buffer_memory, None);
            
            Ok((index_buffer, index_buffer_memory))
        }

        /// Load a custom mesh from OBJ file and create GPU buffers
        /// Returns the calculated bounds (min, max) of the mesh
        pub unsafe fn load_custom_mesh(&mut self, path: &str) -> anyhow::Result<(glam::Vec3, glam::Vec3)> {
            // Check if already loaded
            if let Some((mesh, _, _, _, _)) = self.custom_meshes.get(path) {
                return Ok(mesh.calculate_bounds());
            }

            println!("Loading custom mesh: {}", path);

            // Load mesh from file
            let mesh = Mesh::from_obj(path)?;

            // Calculate bounds before moving mesh
            let bounds = mesh.calculate_bounds();

            // Create vertex buffer
            let (vertex_buffer, vertex_memory) = Self::create_vertex_buffer(
                &self.instance,
                self.physical_device,
                &self.device,
                self.command_pool,
                self.graphics_queue,
                &mesh.vertices,
            )?;

            // Create index buffer
            let (index_buffer, index_memory) = Self::create_index_buffer(
                &self.instance,
                self.physical_device,
                &self.device,
                self.command_pool,
                self.graphics_queue,
                &mesh.indices,
            )?;

            // Store in registry
            self.custom_meshes.insert(
                path.to_string(),
                (mesh, vertex_buffer, vertex_memory, index_buffer, index_memory),
            );

            println!("Custom mesh loaded successfully: {} (bounds: {:?} to {:?})", path, bounds.0, bounds.1);
            Ok(bounds)
        }

        unsafe fn create_uniform_buffers(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
            let buffer_size = std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize;
            
            let mut buffers = vec![];
            let mut memories = vec![];
            
            for _ in 0..count {
                let (buffer, memory) = Self::create_buffer(
                    instance,
                    physical_device,
                    device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;
                buffers.push(buffer);
                memories.push(memory);
            }
            
            Ok((buffers, memories))
        }
        unsafe fn create_star_uniform_buffers(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
            let buffer_size = std::mem::size_of::<StarUniformBufferObject>() as vk::DeviceSize;

            let mut buffers = vec![];
            let mut memories = vec![];

            for _ in 0..count {
                let (buffer, memory) = Self::create_buffer(
                    instance,
                    physical_device,
                    device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;
                buffers.push(buffer);
                memories.push(memory);
            }

            Ok((buffers, memories))
        }

        unsafe fn create_gizmo_uniform_buffers(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<(Vec<vk::Buffer>, Vec<vk::DeviceMemory>)> {
            let buffer_size = std::mem::size_of::<GizmoUniformBufferObject>() as vk::DeviceSize;

            let mut buffers = vec![];
            let mut memories = vec![];

            for _ in 0..count {
                let (buffer, memory) = Self::create_buffer(
                    instance,
                    physical_device,
                    device,
                    buffer_size,
                    vk::BufferUsageFlags::UNIFORM_BUFFER,
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )?;
                buffers.push(buffer);
                memories.push(memory);
            }

            Ok((buffers, memories))
        }

        unsafe fn create_buffer(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            size: vk::DeviceSize,
            usage: vk::BufferUsageFlags,
            properties: vk::MemoryPropertyFlags,
        ) -> anyhow::Result<(vk::Buffer, vk::DeviceMemory)> {
            let buffer_info = vk::BufferCreateInfo::default()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);
            
            let buffer = device.create_buffer(&buffer_info, None)?;
            let mem_requirements = device.get_buffer_memory_requirements(buffer);
            
            let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                instance,
                physical_device,
                mem_requirements.memory_type_bits,
                properties,
            )?);
            
            let buffer_memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_buffer_memory(buffer, buffer_memory, 0)?;
            
            Ok((buffer, buffer_memory))
        }
        
        unsafe fn find_memory_type(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            type_filter: u32,
            properties: vk::MemoryPropertyFlags,
        ) -> anyhow::Result<u32> {
            let mem_properties = instance.get_physical_device_memory_properties(physical_device);
            
            for i in 0..mem_properties.memory_type_count {
                if (type_filter & (1 << i)) != 0
                && mem_properties.memory_types[i as usize]
                .property_flags
                .contains(properties)
                {
                    return Ok(i);
                }
            }
            
            anyhow::bail!("Failed to find suitable memory type")
        }
        
        unsafe fn copy_buffer(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            graphics_queue: vk::Queue,
            src_buffer: vk::Buffer,
            dst_buffer: vk::Buffer,
            size: vk::DeviceSize,
        ) -> anyhow::Result<()> {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_pool(command_pool)
            .command_buffer_count(1);
            
            let command_buffers = device.allocate_command_buffers(&alloc_info)?;
            let command_buffer = command_buffers[0];
            
            let begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
            
            device.begin_command_buffer(command_buffer, &begin_info)?;
            
            let copy_region = vk::BufferCopy::default().size(size);
            device.cmd_copy_buffer(
                command_buffer,
                src_buffer,
                dst_buffer,
                std::slice::from_ref(&copy_region),
            );
            
            device.end_command_buffer(command_buffer)?;
            
            let submit_info = vk::SubmitInfo::default()
            .command_buffers(std::slice::from_ref(&command_buffer));
            
            device.queue_submit(graphics_queue, std::slice::from_ref(&submit_info), vk::Fence::null())?;
            device.queue_wait_idle(graphics_queue)?;
            
            device.free_command_buffers(command_pool, &command_buffers);
            
            Ok(())
        }
        
        unsafe fn create_descriptor_pool(
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<vk::DescriptorPool> {
            let pool_sizes = [
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(count as u32),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(count as u32),
            ];

            let create_info = vk::DescriptorPoolCreateInfo::default()
            .pool_sizes(&pool_sizes)
            .max_sets(count as u32);

            Ok(device.create_descriptor_pool(&create_info, None)?)
        }
        unsafe fn create_descriptor_sets(
            device: &ash::Device,
            pool: vk::DescriptorPool,
            layout: vk::DescriptorSetLayout,
            buffers: &[vk::Buffer],
            ssao_image_view: vk::ImageView,
            ssao_sampler: vk::Sampler,
            count: usize,
        ) -> anyhow::Result<Vec<vk::DescriptorSet>> {
            let layouts = vec![layout; count];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(pool)
            .set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

            for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
                let buffer_info = vk::DescriptorBufferInfo::default()
                .buffer(buffers[i])
                .offset(0)
                .range(std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize);

                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(ssao_image_view)
                    .sampler(ssao_sampler);

                let descriptor_writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_info)),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_set)
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info)),
                ];

                device.update_descriptor_sets(&descriptor_writes, &[]);
            }

            Ok(descriptor_sets)
        }
        unsafe fn create_star_descriptor_pool(
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<vk::DescriptorPool> {
            let pool_sizes = [
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(count as u32),
            ];

            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(count as u32);

            Ok(device.create_descriptor_pool(&create_info, None)?)
        }

        unsafe fn create_star_descriptor_sets(
            device: &ash::Device,
            pool: vk::DescriptorPool,
            layout: vk::DescriptorSetLayout,
            buffers: &[vk::Buffer],
            count: usize,
        ) -> anyhow::Result<Vec<vk::DescriptorSet>> {
            let layouts = vec![layout; count];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

            for (i, &descriptor_set) in descriptor_sets.iter().enumerate() {
                // Binding 0: Uniform buffer
                let buffer_info = vk::DescriptorBufferInfo::default()
                    .buffer(buffers[i])
                    .offset(0)
                    .range(std::mem::size_of::<StarUniformBufferObject>() as vk::DeviceSize);

                let buffer_write = vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_set)
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                    .buffer_info(std::slice::from_ref(&buffer_info));

                let descriptor_writes = [buffer_write];
                device.update_descriptor_sets(&descriptor_writes, &[]);
            }

            Ok(descriptor_sets)
        }

        unsafe fn create_command_buffers(
            device: &ash::Device,
            command_pool: vk::CommandPool,
            count: usize,
        ) -> anyhow::Result<Vec<vk::CommandBuffer>> {
            let alloc_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(count as u32);
            
            Ok(device.allocate_command_buffers(&alloc_info)?)
        }
        
        unsafe fn create_sync_objects(
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<(Vec<vk::Semaphore>, Vec<vk::Semaphore>, Vec<vk::Fence>)> {
            let semaphore_info = vk::SemaphoreCreateInfo::default();
            let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
            
            let mut image_available = vec![];
            let mut render_finished = vec![];
            let mut in_flight = vec![];
            
            for _ in 0..count {
                image_available.push(device.create_semaphore(&semaphore_info, None)?);
                render_finished.push(device.create_semaphore(&semaphore_info, None)?);
                in_flight.push(device.create_fence(&fence_info, None)?);
            }
            
            Ok((image_available, render_finished, in_flight))
        }
        
        unsafe fn create_depth_resources(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            extent: vk::Extent2D,
        ) -> anyhow::Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
            let format = vk::Format::D32_SFLOAT;
            
            let image_info = vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .tiling(vk::ImageTiling::OPTIMAL)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .samples(vk::SampleCountFlags::TYPE_1);
            
            let depth_image = device.create_image(&image_info, None)?;
            let mem_requirements = device.get_image_memory_requirements(depth_image);
            
            let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(mem_requirements.size)
            .memory_type_index(Self::find_memory_type(
                instance,
                physical_device,
                mem_requirements.memory_type_bits,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            )?);
            
            let depth_image_memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_image_memory(depth_image, depth_image_memory, 0)?;
            
            let view_info = vk::ImageViewCreateInfo::default()
            .image(depth_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            });
            
            let depth_image_view = device.create_image_view(&view_info, None)?;
            
            Ok((depth_image, depth_image_memory, depth_image_view))
        }

        unsafe fn create_depth_sampler(device: &ash::Device) -> anyhow::Result<vk::Sampler> {
            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::NEAREST)
                .min_filter(vk::Filter::NEAREST)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .anisotropy_enable(false)
                .max_anisotropy(1.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(0.0);

            Ok(device.create_sampler(&sampler_info, None)?)
        }

        unsafe fn create_ssao_image(
            instance: &ash::Instance,
            physical_device: vk::PhysicalDevice,
            device: &ash::Device,
            extent: vk::Extent2D,
        ) -> anyhow::Result<(vk::Image, vk::DeviceMemory, vk::ImageView)> {
            let format = vk::Format::R8_UNORM; // Single channel for AO

            let image_info = vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .extent(vk::Extent3D {
                    width: extent.width,
                    height: extent.height,
                    depth: 1,
                })
                .mip_levels(1)
                .array_layers(1)
                .format(format)
                .tiling(vk::ImageTiling::OPTIMAL)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED)
                .sharing_mode(vk::SharingMode::EXCLUSIVE)
                .samples(vk::SampleCountFlags::TYPE_1);

            let image = device.create_image(&image_info, None)?;
            let mem_requirements = device.get_image_memory_requirements(image);

            let alloc_info = vk::MemoryAllocateInfo::default()
                .allocation_size(mem_requirements.size)
                .memory_type_index(Self::find_memory_type(
                    instance,
                    physical_device,
                    mem_requirements.memory_type_bits,
                    vk::MemoryPropertyFlags::DEVICE_LOCAL,
                )?);

            let image_memory = device.allocate_memory(&alloc_info, None)?;
            device.bind_image_memory(image, image_memory, 0)?;

            let view_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            let image_view = device.create_image_view(&view_info, None)?;

            Ok((image, image_memory, image_view))
        }

        unsafe fn create_ssao_sampler(device: &ash::Device) -> anyhow::Result<vk::Sampler> {
            let sampler_info = vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                .anisotropy_enable(false)
                .max_anisotropy(1.0)
                .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .mip_lod_bias(0.0)
                .min_lod(0.0)
                .max_lod(0.0);

            Ok(device.create_sampler(&sampler_info, None)?)
        }

        unsafe fn create_ssao_render_pass(device: &ash::Device) -> anyhow::Result<vk::RenderPass> {
            // Single color attachment for SSAO output (R8_UNORM)
            let color_attachment = vk::AttachmentDescription::default()
                .format(vk::Format::R8_UNORM)
                .samples(vk::SampleCountFlags::TYPE_1)
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);

            let color_attachment_ref = vk::AttachmentReference::default()
                .attachment(0)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

            let subpass = vk::SubpassDescription::default()
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .color_attachments(std::slice::from_ref(&color_attachment_ref));

            let dependency = vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

            let create_info = vk::RenderPassCreateInfo::default()
                .attachments(std::slice::from_ref(&color_attachment))
                .subpasses(std::slice::from_ref(&subpass))
                .dependencies(std::slice::from_ref(&dependency));

            Ok(device.create_render_pass(&create_info, None)?)
        }

        unsafe fn create_ssao_framebuffer(
            device: &ash::Device,
            render_pass: vk::RenderPass,
            image_view: vk::ImageView,
            extent: vk::Extent2D,
        ) -> anyhow::Result<vk::Framebuffer> {
            let attachments = [image_view];

            let framebuffer_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(&attachments)
                .width(extent.width)
                .height(extent.height)
                .layers(1);

            Ok(device.create_framebuffer(&framebuffer_info, None)?)
        }

        unsafe fn create_ssao_descriptor_pool(
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<vk::DescriptorPool> {
            let pool_sizes = [
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::UNIFORM_BUFFER)
                    .descriptor_count(count as u32),
                vk::DescriptorPoolSize::default()
                    .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .descriptor_count(count as u32),
            ];

            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(&pool_sizes)
                .max_sets(count as u32);

            Ok(device.create_descriptor_pool(&create_info, None)?)
        }

        unsafe fn create_ssao_blur_descriptor_pool(
            device: &ash::Device,
            count: usize,
        ) -> anyhow::Result<vk::DescriptorPool> {
            // Need 2 image samplers per set: SSAO input + depth texture
            let pool_size = vk::DescriptorPoolSize::default()
                .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                .descriptor_count((count * 2) as u32);

            let create_info = vk::DescriptorPoolCreateInfo::default()
                .pool_sizes(std::slice::from_ref(&pool_size))
                .max_sets(count as u32);

            Ok(device.create_descriptor_pool(&create_info, None)?)
        }

        unsafe fn create_ssao_descriptor_sets(
            device: &ash::Device,
            pool: vk::DescriptorPool,
            layout: vk::DescriptorSetLayout,
            uniform_buffers: &[vk::Buffer],
            depth_image_view: vk::ImageView,
            depth_sampler: vk::Sampler,
            count: usize,
        ) -> anyhow::Result<Vec<vk::DescriptorSet>> {
            let layouts = vec![layout; count];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

            for i in 0..count {
                let buffer_info = vk::DescriptorBufferInfo::default()
                    .buffer(uniform_buffers[i])
                    .offset(0)
                    .range(std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize);

                let image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(depth_image_view)
                    .sampler(depth_sampler);

                let descriptor_writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_sets[i])
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                        .buffer_info(std::slice::from_ref(&buffer_info)),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_sets[i])
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&image_info)),
                ];

                device.update_descriptor_sets(&descriptor_writes, &[]);
            }

            Ok(descriptor_sets)
        }

        unsafe fn create_ssao_blur_descriptor_sets(
            device: &ash::Device,
            pool: vk::DescriptorPool,
            layout: vk::DescriptorSetLayout,
            ssao_image_view: vk::ImageView,
            ssao_sampler: vk::Sampler,
            depth_image_view: vk::ImageView,
            depth_sampler: vk::Sampler,
            count: usize,
        ) -> anyhow::Result<Vec<vk::DescriptorSet>> {
            let layouts = vec![layout; count];
            let alloc_info = vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(pool)
                .set_layouts(&layouts);

            let descriptor_sets = device.allocate_descriptor_sets(&alloc_info)?;

            for i in 0..count {
                // Binding 0: SSAO input
                let ssao_image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(ssao_image_view)
                    .sampler(ssao_sampler);

                // Binding 1: Depth texture for edge detection
                let depth_image_info = vk::DescriptorImageInfo::default()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(depth_image_view)
                    .sampler(depth_sampler);

                let descriptor_writes = [
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_sets[i])
                        .dst_binding(0)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&ssao_image_info)),
                    vk::WriteDescriptorSet::default()
                        .dst_set(descriptor_sets[i])
                        .dst_binding(1)
                        .dst_array_element(0)
                        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                        .image_info(std::slice::from_ref(&depth_image_info)),
                ];

                device.update_descriptor_sets(&descriptor_writes, &[]);
            }

            Ok(descriptor_sets)
        }

        unsafe fn update_uniform_buffer(&mut self, image_index: usize, game: &crate::game::Game) -> anyhow::Result<()> {
            let view = game.get_view_matrix();

            // CRITICAL: Use the EXACT SAME projection matrix as gizmo and picking!
            let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
            let proj = game.camera.projection_matrix(aspect);

            // Get light direction from scene object rotation, or use default
            let dir_light_direction = if let Some(light_id) = game.scene.find_by_type(crate::scene::ObjectType::DirectionalLight) {
                if let Some(light_obj) = game.scene.get_object(light_id) {
                    // Light arrow points down -Y, rotate it by the object's rotation
                    let dir = light_obj.transform.rotation * glam::Vec3::NEG_Y;
                    dir.normalize()
                } else {
                    self.directional_light.direction
                }
            } else {
                self.directional_light.direction
            };

            let ubo = UniformBufferObject {
                view,
                proj,
                view_pos: game.get_camera_position(),
                _padding: 0.0,
                dir_light_direction,
                _padding2: 0.0,
                dir_light_color: game.directional_light.color,
                dir_light_intensity: game.directional_light.intensity,
                dir_light_shadow_color: game.directional_light.shadow_color,
                star_density: game.skybox_config.star_density,
                star_brightness: game.skybox_config.star_brightness,
                _pad0: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
                nebula_primary_color: game.skybox_config.nebula_primary_color,
                nebula_intensity: game.skybox_config.nebula_intensity,
                nebula_secondary_color: game.skybox_config.nebula_secondary_color,
                background_brightness: game.skybox_config.background_brightness,
                point_light_count: self.point_lights.len().min(MAX_POINT_LIGHTS) as u32,
                ssao_enabled: if game.ssao_config.enabled { 1 } else { 0 },
                _padding3: [0; 2],
            };
            
            let data = self.device.map_memory(
                self.uniform_buffers_memory[image_index],
                0,
                std::mem::size_of::<UniformBufferObject>() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(&ubo, data as *mut UniformBufferObject, 1);
            self.device.unmap_memory(self.uniform_buffers_memory[image_index]);
            
            Ok(())
        }

        unsafe fn update_gizmo_uniform_buffer(&mut self, image_index: usize, game: &crate::game::Game) -> anyhow::Result<()> {
            let view = game.get_view_matrix();

            // CRITICAL: Use the EXACT SAME projection matrix as picking!
            // This ensures rendered gizmo position matches raycast picking position
            let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
            let proj = game.camera.projection_matrix(aspect);

            // Gizmo transform based on mode:
            // - Translate: world-space orientation (easier to use)
            // - Rotate/Scale: object-space orientation (rotate with object)
            // Scale the gizmo based on distance from camera to maintain constant screen size
            let model = if let Some(obj) = game.scene.selected_object() {
                // Calculate distance from camera to object
                let distance = (obj.transform.position - game.camera.position()).length();
                // Scale factor: make gizmo size proportional to distance (0.15 is a tuning factor)
                let gizmo_scale = distance * 0.15;

                match game.gizmo_state.mode {
                    crate::gizmo::GizmoMode::Translate => {
                        // World-space: only position + uniform scale
                        Mat4::from_scale_rotation_translation(
                            Vec3::splat(gizmo_scale),
                            Quat::IDENTITY,
                            obj.transform.position
                        )
                    }
                    crate::gizmo::GizmoMode::Rotate | crate::gizmo::GizmoMode::Scale => {
                        // Object-space: position + rotation + uniform scale
                        Mat4::from_scale_rotation_translation(
                            Vec3::splat(gizmo_scale),
                            obj.transform.rotation,
                            obj.transform.position
                        )
                    }
                }
            } else {
                Mat4::IDENTITY
            };

            let ubo = GizmoUniformBufferObject {
                model,
                view,
                proj,
            };

            let data = self.device.map_memory(
                self.gizmo_uniform_buffers_memory[image_index],
                0,
                std::mem::size_of::<GizmoUniformBufferObject>() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(&ubo, data as *mut GizmoUniformBufferObject, 1);
            self.device.unmap_memory(self.gizmo_uniform_buffers_memory[image_index]);

            Ok(())
        }

        unsafe fn update_ssao_uniform_buffer(&mut self, image_index: usize, game: &crate::game::Game) -> anyhow::Result<()> {
            let aspect = self.swapchain_extent.width as f32 / self.swapchain_extent.height as f32;
            let proj = game.camera.projection_matrix(aspect);

            // Create SSAO-specific UBO
            let ubo = SSAOUniformBufferObject {
                proj,
                ssao_radius: game.ssao_config.radius,
                ssao_bias: game.ssao_config.bias,
                ssao_power: game.ssao_config.power,
                ssao_kernel_size: game.ssao_config.kernel_size,
            };

            let data = self.device.map_memory(
                self.ssao_uniform_buffers_memory[image_index],
                0,
                std::mem::size_of::<SSAOUniformBufferObject>() as vk::DeviceSize,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(&ubo, data as *mut SSAOUniformBufferObject, 1);
            self.device.unmap_memory(self.ssao_uniform_buffers_memory[image_index]);

            Ok(())
        }

        /// Update gizmo vertex and index buffers with the current mesh data
        unsafe fn update_gizmo_buffers(&mut self, mesh: &Mesh) -> anyhow::Result<()> {
            // Update vertex buffer - map the full buffer size, not just the mesh size
            let vertex_data = self.device.map_memory(
                self.gizmo_vertex_buffer_memory,
                0,
                self.gizmo_vertex_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                mesh.vertices.as_ptr(),
                vertex_data as *mut Vertex,
                mesh.vertices.len(),
            );
            self.device.unmap_memory(self.gizmo_vertex_buffer_memory);

            // Update index buffer - map the full buffer size, not just the mesh size
            let index_data = self.device.map_memory(
                self.gizmo_index_buffer_memory,
                0,
                self.gizmo_index_buffer_size,
                vk::MemoryMapFlags::empty(),
            )?;
            std::ptr::copy_nonoverlapping(
                mesh.indices.as_ptr(),
                index_data as *mut u32,
                mesh.indices.len(),
            );
            self.device.unmap_memory(self.gizmo_index_buffer_memory);

            Ok(())
        }

        /// Render movement widget for tactical turn-based movement
        pub fn render(&mut self, game: &mut crate::game::Game) -> anyhow::Result<()> {
            // Load any new custom meshes
            unsafe {
                let mesh_objects = game.get_visible_meshes();
                for (mesh_path, _) in mesh_objects.iter() {
                    if !self.custom_meshes.contains_key(mesh_path) {
                        match self.load_custom_mesh(mesh_path) {
                            Ok((bounds_min, bounds_max)) => {
                                // Update ship bounds in game
                                game.update_ship_bounds(mesh_path, bounds_min, bounds_max);
                            }
                            Err(e) => {
                                eprintln!("Failed to load mesh {}: {}", mesh_path, e);
                            }
                        }
                    }
                }
            }

            // Frame rate limiting to 120 FPS
            let target_frame_time = std::time::Duration::from_secs_f64(1.0 / 120.0);
            let elapsed = self.last_frame_time.elapsed();
            if elapsed < target_frame_time {
                std::thread::sleep(target_frame_time - elapsed);
            }
            self.last_frame_time = std::time::Instant::now();

            unsafe {
                self.device.wait_for_fences(
                    &[self.in_flight_fences[self.current_frame]],
                    true,
                    u64::MAX,
                )?;
                
                let result = self.swapchain_loader.acquire_next_image(
                    self.swapchain,
                    u64::MAX,
                    self.image_available_semaphores[self.current_frame],
                    vk::Fence::null(),
                );
                
                let image_index = match result {
                    Ok((image_index, _)) => image_index,
                    Err(vk::Result::ERROR_OUT_OF_DATE_KHR) => {
                        self.recreate_swapchain()?;
                        return Ok(());
                    }
                    Err(e) => return Err(anyhow::anyhow!("Failed to acquire swap chain image: {}", e)),
                };
                
                // Wait for this image if it's currently being rendered
                if self.images_in_flight[image_index as usize] != vk::Fence::null() {
                    self.device.wait_for_fences(
                        &[self.images_in_flight[image_index as usize]],
                        true,
                        u64::MAX,
                    )?;
                }
                
                // Mark the image as now being in use by this frame
                self.images_in_flight[image_index as usize] = self.in_flight_fences[self.current_frame];
                
                self.update_uniform_buffer(self.current_frame, game)?;
                self.update_gizmo_uniform_buffer(self.current_frame, game)?;
                self.update_ssao_uniform_buffer(self.current_frame, game)?;

                // Update render passes (plugin system)
                let ctx = crate::core::RenderContext {
                    device: &self.device,
                    instance: &self.instance,
                    physical_device: self.physical_device,
                    command_pool: self.command_pool,
                    graphics_queue: self.graphics_queue,
                    extent: self.swapchain_extent,
                    depth_image_view: Some(self.depth_image_view),
                    depth_sampler: Some(self.depth_sampler),
                    mesh_pipeline: Some(self.graphics_pipeline),
                    mesh_pipeline_layout: Some(self.pipeline_layout),
                    mesh_descriptor_sets: Some(&self.descriptor_sets),
                    custom_meshes: Some(&self.custom_meshes),
                };
                self.render_passes.update_all(&ctx, self.current_frame, game)?;

                // Prepare ImGui frame
                self.imgui_platform.prepare_frame(self.imgui_context.io_mut(), &self.window)?;
                self.build_ui(game);
                
                self.device.reset_fences(&[self.in_flight_fences[self.current_frame]])?;
                
                self.device.reset_command_buffer(
                    self.command_buffers[self.current_frame],
                    vk::CommandBufferResetFlags::empty(),
                )?;
                
                self.record_command_buffer(self.command_buffers[self.current_frame], image_index as usize, game)?;
                
                let wait_semaphores = [self.image_available_semaphores[self.current_frame]];
                let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
                let signal_semaphores = [self.render_finished_semaphores[self.current_frame]];
                let command_buffers = [self.command_buffers[self.current_frame]];
                
                let submit_info = vk::SubmitInfo::default()
                .wait_semaphores(&wait_semaphores)
                .wait_dst_stage_mask(&wait_stages)
                .command_buffers(&command_buffers)
                .signal_semaphores(&signal_semaphores);
                
                self.device.queue_submit(
                    self.graphics_queue,
                    &[submit_info],
                    self.in_flight_fences[self.current_frame],
                )?;
                
                let swapchains = [self.swapchain];
                let image_indices = [image_index];
                let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(&signal_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);
                
                let result = self.swapchain_loader.queue_present(self.present_queue, &present_info);
                
                match result {
                    Ok(true) | Err(vk::Result::ERROR_OUT_OF_DATE_KHR) | Err(vk::Result::SUBOPTIMAL_KHR) => {
                        self.recreate_swapchain()?;
                    }
                    Err(e) => return Err(anyhow::anyhow!("Failed to present swap chain image: {}", e)),
                    _ => {}
                }
                
                self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
                self.frame_count += 1;
                self.fps_frame_count += 1;
                
                let now = std::time::Instant::now();
                let elapsed = now.duration_since(self.last_time);
                
                if elapsed.as_secs_f64() >= 0.5 {
                    let fps = self.fps_frame_count as f64 / elapsed.as_secs_f64();
                    self.window.set_title(&format!("Tribal Engine | FPS: {:.0}", fps));
                    self.fps_frame_count = 0;
                    self.last_time = now;
                }
            }
            
            Ok(())
        }
        
        unsafe fn record_command_buffer(&mut self, command_buffer: vk::CommandBuffer, image_index: usize, game: &crate::game::Game) -> anyhow::Result<()> {
            let begin_info = vk::CommandBufferBeginInfo::default();
            
            self.device.begin_command_buffer(command_buffer, &begin_info)?;
            
            let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.01, 0.01, 0.02, 1.0],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            },
            ];
            
            let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[image_index])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(&clear_values);
            
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
            
            // 1. Render passes (skybox, nebula, meshes, etc.) - plugin system
            let ctx = crate::core::RenderContext {
                device: &self.device,
                instance: &self.instance,
                physical_device: self.physical_device,
                command_pool: self.command_pool,
                graphics_queue: self.graphics_queue,
                extent: self.swapchain_extent,
                depth_image_view: Some(self.depth_image_view),
                depth_sampler: Some(self.depth_sampler),
                mesh_pipeline: Some(self.graphics_pipeline),
                mesh_pipeline_layout: Some(self.pipeline_layout),
                mesh_descriptor_sets: Some(&self.descriptor_sets),
                custom_meshes: Some(&self.custom_meshes),
            };
            self.render_passes.render_all(&ctx, command_buffer, self.current_frame, game)?;

            // Mesh rendering (cubes, custom meshes) and stars now handled by render pass plugins

            // Transition depth image for shader reading
            let depth_barrier = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.depth_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[depth_barrier],
            );

            // 3. Nebula is now rendered by the render pass plugin system above

            // Transition depth image back to depth attachment for next frame
            let depth_barrier_back = vk::ImageMemoryBarrier::default()
                .old_layout(vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL)
                .new_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(self.depth_image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                })
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE);

            self.device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[depth_barrier_back],
            );

            // 4. Render gizmo (if enabled, object selected, and in edit mode)
            let in_edit_mode = game.game_manager.mode == crate::game_manager::GameMode::Edit;
            if in_edit_mode && game.gizmo_state.enabled && game.scene.selected_object().is_some() {
                // Select the appropriate mesh based on current mode and get index count
                let (index_count, mesh_vertices, mesh_indices) = match game.gizmo_state.mode {
                    crate::gizmo::GizmoMode::Translate => (
                        self.gizmo_translate_mesh.indices.len() as u32,
                        self.gizmo_translate_mesh.vertices.clone(),
                        self.gizmo_translate_mesh.indices.clone(),
                    ),
                    crate::gizmo::GizmoMode::Rotate => (
                        self.gizmo_rotate_mesh.indices.len() as u32,
                        self.gizmo_rotate_mesh.vertices.clone(),
                        self.gizmo_rotate_mesh.indices.clone(),
                    ),
                    crate::gizmo::GizmoMode::Scale => (
                        self.gizmo_scale_mesh.indices.len() as u32,
                        self.gizmo_scale_mesh.vertices.clone(),
                        self.gizmo_scale_mesh.indices.clone(),
                    ),
                };

                // Create a temporary mesh to pass to update function
                let temp_mesh = Mesh {
                    vertices: mesh_vertices,
                    indices: mesh_indices,
                };

                // Update buffers with current mesh data
                self.update_gizmo_buffers(&temp_mesh)?;

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.gizmo_pipeline,
                );

                let gizmo_vertex_buffers = [self.gizmo_vertex_buffer];
                let gizmo_offsets = [0];
                self.device.cmd_bind_vertex_buffers(command_buffer, 0, &gizmo_vertex_buffers, &gizmo_offsets);
                self.device.cmd_bind_index_buffer(command_buffer, self.gizmo_index_buffer, 0, vk::IndexType::UINT32);

                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.gizmo_pipeline_layout,
                    0,
                    &[self.gizmo_descriptor_sets[self.current_frame]],
                    &[],
                );

                // Push hovered axis constant (0=none, 1=X, 2=Y, 3=Z)
                let hovered_axis = match game.gizmo_state.hovered_axis {
                    crate::gizmo::GizmoAxis::None => 0i32,
                    crate::gizmo::GizmoAxis::X => 1i32,
                    crate::gizmo::GizmoAxis::Y => 2i32,
                    crate::gizmo::GizmoAxis::Z => 3i32,
                };
                let push_constants = hovered_axis.to_le_bytes();
                self.device.cmd_push_constants(
                    command_buffer,
                    self.gizmo_pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    &push_constants,
                );

                self.device.cmd_draw_indexed(command_buffer, index_count, 1, 0, 0, 0);
            }

            // 5. Render directional light visualization (yellow wireframe) - editor only
            if in_edit_mode {
                if let Some(light_transform) = game.get_directional_light() {
                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.wireframe_pipeline,
                );

                let vertex_buffers = [self.dir_light_vertex_buffer];
                let offsets = [0];
                self.device.cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, &offsets);
                self.device.cmd_bind_index_buffer(command_buffer, self.dir_light_index_buffer, 0, vk::IndexType::UINT32);

                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.pipeline_layout,
                    0,
                    &[self.descriptor_sets[self.current_frame]],
                    &[],
                );

                // Push light transform matrix
                let model_array = [light_transform];
                let push_constants = bytemuck::cast_slice(&model_array);
                self.device.cmd_push_constants(
                    command_buffer,
                    self.pipeline_layout,
                    vk::ShaderStageFlags::VERTEX,
                    0,
                    push_constants,
                );

                self.device.cmd_draw_indexed(command_buffer, self.dir_light_mesh.indices.len() as u32, 1, 0, 0, 0);
                }
            }

            // Render ImGui
            let draw_data = self.imgui_context.render();
            self.imgui_renderer.render(
                &self.device,
                &self.instance,
                self.physical_device,
                command_buffer,
                self.command_pool,
                self.graphics_queue,
                draw_data,
            )?;

            self.device.cmd_end_render_pass(command_buffer);

            // SSAO Pass - only if enabled
            if game.ssao_config.enabled {
                // SSAO Pass - compute ambient occlusion from depth buffer
                let ssao_clear_values = [vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [1.0, 0.0, 0.0, 0.0], // Clear to 1.0 (no occlusion)
                    },
                }];

                let ssao_render_pass_info = vk::RenderPassBeginInfo::default()
                    .render_pass(self.ssao_render_pass)
                    .framebuffer(self.ssao_framebuffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain_extent,
                    })
                    .clear_values(&ssao_clear_values);

                self.device.cmd_begin_render_pass(
                    command_buffer,
                    &ssao_render_pass_info,
                    vk::SubpassContents::INLINE,
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_pipeline,
                );

                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_pipeline_layout,
                    0,
                    &[self.ssao_descriptor_sets[self.current_frame]],
                    &[],
                );

                // Draw fullscreen triangle (no vertex buffer needed)
                self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

                self.device.cmd_end_render_pass(command_buffer);

                // SSAO Blur Pass 1: Horizontal blur (SSAO -> intermediate)
                let ssao_blur_clear_values = [vk::ClearValue {
                    color: vk::ClearColorValue {
                        float32: [1.0, 0.0, 0.0, 0.0],
                    },
                }];

                let ssao_blur_horizontal_render_pass_info = vk::RenderPassBeginInfo::default()
                    .render_pass(self.ssao_blur_horizontal_render_pass)
                    .framebuffer(self.ssao_blur_horizontal_framebuffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain_extent,
                    })
                    .clear_values(&ssao_blur_clear_values);

                self.device.cmd_begin_render_pass(
                    command_buffer,
                    &ssao_blur_horizontal_render_pass_info,
                    vk::SubpassContents::INLINE,
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_blur_pipeline,
                );

                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_blur_pipeline_layout,
                    0,
                    &[self.ssao_blur_horizontal_descriptor_sets[self.current_frame]],
                    &[],
                );

                // Push constant: direction = 0 (horizontal)
                let horizontal_direction: i32 = 0;
                self.device.cmd_push_constants(
                    command_buffer,
                    self.ssao_blur_pipeline_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(
                        &horizontal_direction as *const i32 as *const u8,
                        std::mem::size_of::<i32>(),
                    ),
                );

                // Draw fullscreen triangle
                self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

                self.device.cmd_end_render_pass(command_buffer);

                // SSAO Blur Pass 2: Vertical blur (intermediate -> final)
                let ssao_blur_vertical_render_pass_info = vk::RenderPassBeginInfo::default()
                    .render_pass(self.ssao_blur_render_pass)
                    .framebuffer(self.ssao_blur_framebuffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain_extent,
                    })
                    .clear_values(&ssao_blur_clear_values);

                self.device.cmd_begin_render_pass(
                    command_buffer,
                    &ssao_blur_vertical_render_pass_info,
                    vk::SubpassContents::INLINE,
                );

                self.device.cmd_bind_pipeline(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_blur_pipeline,
                );

                self.device.cmd_bind_descriptor_sets(
                    command_buffer,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.ssao_blur_pipeline_layout,
                    0,
                    &[self.ssao_blur_descriptor_sets[self.current_frame]],
                    &[],
                );

                // Push constant: direction = 1 (vertical)
                let vertical_direction: i32 = 1;
                self.device.cmd_push_constants(
                    command_buffer,
                    self.ssao_blur_pipeline_layout,
                    vk::ShaderStageFlags::FRAGMENT,
                    0,
                    std::slice::from_raw_parts(
                        &vertical_direction as *const i32 as *const u8,
                        std::mem::size_of::<i32>(),
                    ),
                );

                // Draw fullscreen triangle
                self.device.cmd_draw(command_buffer, 3, 1, 0, 0);

                self.device.cmd_end_render_pass(command_buffer);
            }

            self.device.end_command_buffer(command_buffer)?;

            Ok(())
        }
        
        pub fn handle_resize(&mut self) {
            unsafe {
                // Wait for device to be idle before recreating resources
                let _ = self.device.device_wait_idle();
                
                // Get new window size
                let size = self.window.inner_size();
                
                // Don't recreate if minimized (width or height is 0)
                if size.width == 0 || size.height == 0 {
                    return;
                }
                
                // Recreate swapchain with new size
                if let Err(e) = self.recreate_swapchain() {
                    eprintln!("Failed to recreate swapchain on resize: {}", e);
                }
            }
        }
        
        pub fn window(&self) -> &Window {
            &self.window
        }

        /// Get the actual render viewport dimensions (swapchain extent)
        /// This should be used for mouse picking to match rendering projection
        pub fn viewport_size(&self) -> (f32, f32) {
            (self.swapchain_extent.width as f32, self.swapchain_extent.height as f32)
        }

        pub fn handle_imgui_event(&mut self, window: &Window, event: &winit::event::Event<()>) {
            self.imgui_platform.handle_event(self.imgui_context.io_mut(), window, event);
        }
        
        pub fn imgui_wants_mouse(&self) -> bool {
            self.imgui_context.io().want_capture_mouse
        }

        pub fn imgui_wants_keyboard(&self) -> bool {
            self.imgui_context.io().want_capture_keyboard
        }

        pub fn build_ui(&mut self, game: &mut crate::game::Game) {
            let viewport_width = self.swapchain_extent.width as f32;
            let viewport_height = self.swapchain_extent.height as f32;
            UiManager::build_ui(&mut self.imgui_context, game, viewport_width, viewport_height);
        }
        
        unsafe fn recreate_swapchain(&mut self) -> anyhow::Result<()> {
            self.device.device_wait_idle()?;
            
            self.cleanup_swapchain();
            
            let (swapchain, swapchain_images, swapchain_format, swapchain_extent) =
            Self::create_swapchain(
                &self.window,
                &self.instance,
                self.physical_device,
                &self.device,
                &self.surface_loader,
                self.surface,
                &self.swapchain_loader,
            )?;
            
            let swapchain_image_views =
            Self::create_image_views(&self.device, &swapchain_images, swapchain_format)?;
            
            let (depth_image, depth_image_memory, depth_image_view) = Self::create_depth_resources(
                &self.instance,
                self.physical_device,
                &self.device,
                swapchain_extent,
            )?;
            
            let framebuffers = Self::create_framebuffers(
                &self.device,
                &swapchain_image_views,
                depth_image_view,
                self.render_pass,
                swapchain_extent,
            )?;
            
            // Recreate main graphics pipeline with new extent
            self.device.destroy_pipeline(self.graphics_pipeline, None);
            self.device.destroy_pipeline(self.wireframe_pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            let (pipeline_layout, graphics_pipeline) =
            Self::create_graphics_pipeline(&self.device, swapchain_extent, self.render_pass, self.descriptor_set_layout)?;
            let wireframe_pipeline = Self::create_wireframe_pipeline(&self.device, swapchain_extent, self.render_pass, pipeline_layout)?;
            self.pipeline_layout = pipeline_layout;
            self.graphics_pipeline = graphics_pipeline;
            self.wireframe_pipeline = wireframe_pipeline;

            // Recreate gizmo pipeline with new extent
            self.device.destroy_pipeline(self.gizmo_pipeline, None);
            self.device.destroy_pipeline_layout(self.gizmo_pipeline_layout, None);
            let (gizmo_pipeline_layout, gizmo_pipeline) =
            Self::create_gizmo_pipeline(&self.device, swapchain_extent, self.render_pass, self.gizmo_descriptor_set_layout)?;

            self.swapchain = swapchain;
            self.swapchain_images = swapchain_images.clone();
            self.swapchain_format = swapchain_format;
            self.swapchain_extent = swapchain_extent;
            self.swapchain_image_views = swapchain_image_views;
            self.depth_image = depth_image;
            self.depth_image_memory = depth_image_memory;
            self.depth_image_view = depth_image_view;
            self.framebuffers = framebuffers;
            self.images_in_flight = vec![vk::Fence::null(); swapchain_images.len()];
            
            // Update pipelines
            self.pipeline_layout = pipeline_layout;
            self.graphics_pipeline = graphics_pipeline;
            self.gizmo_pipeline_layout = gizmo_pipeline_layout;
            self.gizmo_pipeline = gizmo_pipeline;

            // Recreate ImGui pipeline with new swapchain extent
            self.imgui_renderer.recreate_pipeline(&self.device, self.render_pass, swapchain_extent)?;

            // Update render passes with new pipeline and extent
            let ctx = crate::core::RenderContext {
                device: &self.device,
                instance: &self.instance,
                physical_device: self.physical_device,
                command_pool: self.command_pool,
                graphics_queue: self.graphics_queue,
                extent: swapchain_extent,
                depth_image_view: Some(depth_image_view),
                depth_sampler: Some(self.depth_sampler),
                mesh_pipeline: Some(graphics_pipeline),
                mesh_pipeline_layout: Some(pipeline_layout),
                mesh_descriptor_sets: Some(&self.descriptor_sets),
                custom_meshes: Some(&self.custom_meshes),
            };
            self.render_passes.recreate_swapchain_all(&ctx, self.render_pass, swapchain_extent)?;

            Ok(())
        }
        
        unsafe fn cleanup_swapchain(&mut self) {
            self.device.destroy_image_view(self.depth_image_view, None);
            self.device.destroy_image(self.depth_image, None);
            self.device.free_memory(self.depth_image_memory, None);
            
            for &framebuffer in &self.framebuffers {
                self.device.destroy_framebuffer(framebuffer, None);
            }
            
            for &image_view in &self.swapchain_image_views {
                self.device.destroy_image_view(image_view, None);
            }
            
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
    
    impl Drop for VulkanRenderer {
        fn drop(&mut self) {
            unsafe {
                self.device.device_wait_idle().unwrap();
                
                // Cleanup ImGui
                self.imgui_renderer.cleanup(&self.device);
                
                self.cleanup_swapchain();
                
                self.device.destroy_buffer(self.index_buffer, None);
                self.device.free_memory(self.index_buffer_memory, None);
                
                self.device.destroy_buffer(self.vertex_buffer, None);
                self.device.free_memory(self.vertex_buffer_memory, None);
                
                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    self.device.destroy_buffer(self.uniform_buffers[i], None);
                    self.device.free_memory(self.uniform_buffers_memory[i], None);
                }
                
                self.device.destroy_descriptor_pool(self.descriptor_pool, None);
                self.device.destroy_descriptor_set_layout(self.descriptor_set_layout, None);

                // Cleanup gizmo resources
                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    self.device.destroy_buffer(self.gizmo_uniform_buffers[i], None);
                    self.device.free_memory(self.gizmo_uniform_buffers_memory[i], None);
                }
                self.device.destroy_descriptor_set_layout(self.gizmo_descriptor_set_layout, None);
                self.device.destroy_pipeline(self.gizmo_pipeline, None);
                self.device.destroy_pipeline_layout(self.gizmo_pipeline_layout, None);
                self.device.destroy_buffer(self.gizmo_index_buffer, None);
                self.device.free_memory(self.gizmo_index_buffer_memory, None);
                self.device.destroy_buffer(self.gizmo_vertex_buffer, None);
                self.device.free_memory(self.gizmo_vertex_buffer_memory, None);

                // Cleanup widget resources
                // Cleanup custom meshes
                for (_path, (_mesh, vertex_buffer, vertex_memory, index_buffer, index_memory)) in self.custom_meshes.drain() {
                    self.device.destroy_buffer(vertex_buffer, None);
                    self.device.free_memory(vertex_memory, None);
                    self.device.destroy_buffer(index_buffer, None);
                    self.device.free_memory(index_memory, None);
                }

                // Cleanup directional light visualization
                self.device.destroy_buffer(self.dir_light_index_buffer, None);
                self.device.free_memory(self.dir_light_index_buffer_memory, None);
                self.device.destroy_buffer(self.dir_light_vertex_buffer, None);
                self.device.free_memory(self.dir_light_vertex_buffer_memory, None);

                // Cleanup depth sampler
                self.device.destroy_sampler(self.depth_sampler, None);

                // Cleanup SSAO resources
                self.device.destroy_descriptor_pool(self.ssao_blur_horizontal_descriptor_pool, None);
                self.device.destroy_descriptor_pool(self.ssao_blur_descriptor_pool, None);
                self.device.destroy_descriptor_pool(self.ssao_descriptor_pool, None);

                self.device.destroy_framebuffer(self.ssao_blur_horizontal_framebuffer, None);
                self.device.destroy_framebuffer(self.ssao_blur_framebuffer, None);
                self.device.destroy_framebuffer(self.ssao_framebuffer, None);

                self.device.destroy_render_pass(self.ssao_blur_horizontal_render_pass, None);
                self.device.destroy_render_pass(self.ssao_blur_render_pass, None);
                self.device.destroy_render_pass(self.ssao_render_pass, None);

                self.device.destroy_pipeline(self.ssao_blur_pipeline, None);
                self.device.destroy_pipeline(self.ssao_pipeline, None);
                self.device.destroy_pipeline_layout(self.ssao_blur_pipeline_layout, None);
                self.device.destroy_pipeline_layout(self.ssao_pipeline_layout, None);

                self.device.destroy_descriptor_set_layout(self.ssao_blur_descriptor_set_layout, None);
                self.device.destroy_descriptor_set_layout(self.ssao_descriptor_set_layout, None);

                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    self.device.destroy_buffer(self.ssao_uniform_buffers[i], None);
                    self.device.free_memory(self.ssao_uniform_buffers_memory[i], None);
                }

                self.device.destroy_sampler(self.ssao_sampler, None);

                self.device.destroy_image_view(self.ssao_blur_intermediate_image_view, None);
                self.device.destroy_image(self.ssao_blur_intermediate_image, None);
                self.device.free_memory(self.ssao_blur_intermediate_image_memory, None);

                self.device.destroy_image_view(self.ssao_blur_image_view, None);
                self.device.destroy_image(self.ssao_blur_image, None);
                self.device.free_memory(self.ssao_blur_image_memory, None);

                self.device.destroy_image_view(self.ssao_image_view, None);
                self.device.destroy_image(self.ssao_image, None);
                self.device.free_memory(self.ssao_image_memory, None);

                for i in 0..MAX_FRAMES_IN_FLIGHT {
                    self.device.destroy_semaphore(self.image_available_semaphores[i], None);
                    self.device.destroy_semaphore(self.render_finished_semaphores[i], None);
                    self.device.destroy_fence(self.in_flight_fences[i], None);
                }

                // Star shader resources now cleaned up by StarPass plugin

                self.device.destroy_command_pool(self.command_pool, None);
                self.device.destroy_pipeline(self.graphics_pipeline, None);
                self.device.destroy_pipeline(self.wireframe_pipeline, None);
                self.device.destroy_pipeline_layout(self.pipeline_layout, None);
                self.device.destroy_render_pass(self.render_pass, None);

                self.device.destroy_device(None);
                
                if let Some((debug_utils, messenger)) = self.debug_utils.take() {
                    debug_utils.destroy_debug_utils_messenger(messenger, None);
                }
                
                self.surface_loader.destroy_surface(self.surface, None);
                self.instance.destroy_instance(None);
            }
        }
    }
    
    #[derive(Default)]
    struct QueueFamilyIndices {
        graphics_family: Option<u32>,
        present_family: Option<u32>,
    }
    
    impl QueueFamilyIndices {
        fn is_complete(&self) -> bool {
            self.graphics_family.is_some() && self.present_family.is_some()
        }
    }
    
    unsafe extern "system" fn vulkan_debug_callback(
        message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        message_type: vk::DebugUtilsMessageTypeFlagsEXT,
        p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
        _user_data: *mut std::os::raw::c_void,
    ) -> vk::Bool32 {
        let callback_data = *p_callback_data;
        let message = if callback_data.p_message.is_null() {
            std::borrow::Cow::from("")
        } else {
            CStr::from_ptr(callback_data.p_message).to_string_lossy()
        };
        
        match message_severity {
            vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
                eprintln!("[Vulkan Error {:?}] {}", message_type, message);
            }
            vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
                eprintln!("[Vulkan Warning {:?}] {}", message_type, message);
            }
            _ => {
                println!("[Vulkan Info {:?}] {}", message_type, message);
            }
        }
        
        vk::FALSE
    }
    