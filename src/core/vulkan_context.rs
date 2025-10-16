use ash::{vk, Entry};
use raw_window_handle::{HasDisplayHandle, HasWindowHandle};
use std::ffi::{CStr, CString};
use winit::window::Window;

/// Core Vulkan context containing the fundamental Vulkan objects needed for rendering.
/// This is the reusable engine component that handles Vulkan initialization and management.
pub struct VulkanContext {
    pub entry: Entry,
    pub instance: ash::Instance,
    pub debug_utils: Option<(ash::ext::debug_utils::Instance, vk::DebugUtilsMessengerEXT)>,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: ash::khr::surface::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub graphics_queue: vk::Queue,
    pub present_queue: vk::Queue,
    pub graphics_queue_family: u32,
    pub present_queue_family: u32,
}

impl VulkanContext {
    /// Create a new Vulkan context for the given window
    pub fn new(window: &Window, app_name: &str, engine_name: &str) -> anyhow::Result<Self> {
        unsafe {
            let entry = Entry::load()?;

            // Create instance
            let app_name_cstr = CString::new(app_name)?;
            let engine_name_cstr = CString::new(engine_name)?;
            let app_info = vk::ApplicationInfo::default()
                .application_name(&app_name_cstr)
                .application_version(vk::make_api_version(0, 1, 0, 0))
                .engine_name(&engine_name_cstr)
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
            let (device, graphics_queue, present_queue, graphics_queue_family, present_queue_family) =
                Self::create_logical_device(&instance, physical_device, &surface_loader, surface)?;

            Ok(Self {
                entry,
                instance,
                debug_utils,
                surface,
                surface_loader,
                physical_device,
                device,
                graphics_queue,
                present_queue,
                graphics_queue_family,
                present_queue_family,
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

        let device = devices
            .iter()
            .find(|&&device| Self::is_device_suitable(instance, device, surface_loader, surface))
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Failed to find suitable GPU"))?;

        Ok(device)
    }

    unsafe fn is_device_suitable(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> bool {
        let queue_families = Self::find_queue_families(instance, device, surface_loader, surface);
        let extensions_supported = Self::check_device_extension_support(instance, device);

        queue_families.graphics_family.is_some()
            && queue_families.present_family.is_some()
            && extensions_supported
    }

    unsafe fn find_queue_families(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> QueueFamilyIndices {
        let queue_families = instance.get_physical_device_queue_family_properties(device);

        let mut indices = QueueFamilyIndices::default();

        for (i, queue_family) in queue_families.iter().enumerate() {
            if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                indices.graphics_family = Some(i as u32);
            }

            let present_support = surface_loader
                .get_physical_device_surface_support(device, i as u32, surface)
                .unwrap_or(false);

            if present_support {
                indices.present_family = Some(i as u32);
            }

            if indices.is_complete() {
                break;
            }
        }

        indices
    }

    unsafe fn check_device_extension_support(
        instance: &ash::Instance,
        device: vk::PhysicalDevice,
    ) -> bool {
        let available_extensions = instance
            .enumerate_device_extension_properties(device)
            .unwrap_or_default();

        let required_extensions = [ash::khr::swapchain::NAME];

        for required in &required_extensions {
            let found = available_extensions.iter().any(|ext| {
                let name = CStr::from_ptr(ext.extension_name.as_ptr());
                required.to_bytes() == name.to_bytes()
            });

            if !found {
                return false;
            }
        }

        true
    }

    unsafe fn create_logical_device(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> anyhow::Result<(ash::Device, vk::Queue, vk::Queue, u32, u32)> {
        let indices = Self::find_queue_families(instance, physical_device, surface_loader, surface);

        let graphics_family = indices.graphics_family.unwrap();
        let present_family = indices.present_family.unwrap();

        let mut unique_queue_families = std::collections::HashSet::new();
        unique_queue_families.insert(graphics_family);
        unique_queue_families.insert(present_family);

        let queue_priority = 1.0f32;
        let queue_create_infos: Vec<_> = unique_queue_families
            .iter()
            .map(|&queue_family| {
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(queue_family)
                    .queue_priorities(std::slice::from_ref(&queue_priority))
            })
            .collect();

        let device_features = vk::PhysicalDeviceFeatures::default();

        let device_extension_names = [ash::khr::swapchain::NAME.as_ptr()];

        let device_create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extension_names)
            .enabled_features(&device_features);

        let device = instance.create_device(physical_device, &device_create_info, None)?;

        let graphics_queue = device.get_device_queue(graphics_family, 0);
        let present_queue = device.get_device_queue(present_family, 0);

        Ok((device, graphics_queue, present_queue, graphics_family, present_family))
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe {
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
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message = CStr::from_ptr((*p_callback_data).p_message);

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
            eprintln!("[Vulkan Error] {:?}: {:?}", message_type, message);
        }
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
            eprintln!("[Vulkan Warning] {:?}: {:?}", message_type, message);
        }
        _ => {
            println!("[Vulkan Info] {:?}: {:?}", message_type, message);
        }
    }

    vk::FALSE
}
