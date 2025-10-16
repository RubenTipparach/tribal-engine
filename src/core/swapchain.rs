use ash::vk;
use winit::window::Window;

/// Manages the Vulkan swapchain and associated resources
pub struct SwapchainManager {
    pub swapchain: vk::SwapchainKHR,
    pub swapchain_loader: ash::khr::swapchain::Device,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
    pub format: vk::Format,
    pub extent: vk::Extent2D,
}

impl SwapchainManager {
    /// Create a new swapchain
    pub unsafe fn new(
        window: &Window,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        graphics_family: u32,
        present_family: u32,
    ) -> anyhow::Result<Self> {
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);

        let (swapchain, images, format, extent) = Self::create_swapchain_internal(
            window,
            instance,
            physical_device,
            device,
            surface_loader,
            surface,
            &swapchain_loader,
            graphics_family,
            present_family,
        )?;

        let image_views = Self::create_image_views(device, &images, format)?;

        Ok(Self {
            swapchain,
            swapchain_loader,
            images,
            image_views,
            format,
            extent,
        })
    }

    unsafe fn create_swapchain_internal(
        window: &Window,
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        _device: &ash::Device,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        swapchain_loader: &ash::khr::swapchain::Device,
        graphics_family: u32,
        present_family: u32,
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

        let queue_family_indices = [graphics_family, present_family];

        let (image_sharing_mode, queue_family_index_count, p_queue_family_indices) =
            if graphics_family != present_family {
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

                device
                    .create_image_view(&create_info, None)
                    .map_err(|e| anyhow::anyhow!("Failed to create image view: {}", e))
            })
            .collect()
    }

    /// Cleanup swapchain resources (does not destroy the swapchain itself)
    pub unsafe fn cleanup_image_views(&mut self, device: &ash::Device) {
        for &image_view in &self.image_views {
            device.destroy_image_view(image_view, None);
        }
        self.image_views.clear();
    }
}

impl Drop for SwapchainManager {
    fn drop(&mut self) {
        // Note: image_views should be cleaned up before dropping
        // The swapchain itself will be destroyed here
        unsafe {
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
        }
    }
}
