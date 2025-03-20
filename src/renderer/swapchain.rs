use crate::renderer::Loaders;
use anyhow::Context;
use ash::vk;

use super::util;

pub struct Swapchain {
    loaders: Loaders,
    pub swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
    pub extent: vk::Extent2D,
    pub format: vk::Format,
}

impl Swapchain {
    pub fn new(
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        surface: vk::SurfaceKHR,
        loaders: Loaders,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
        old_swapchain: Option<Swapchain>,
    ) -> anyhow::Result<Self> {
        let caps = unsafe {
            loaders
                .surface
                .get_physical_device_surface_capabilities(physical_device, surface)?
        };
        let format = vk::Format::B8G8R8A8_UNORM;
        let mut info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(caps.min_image_count)
            .image_format(format)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(vk::SurfaceTransformFlagsKHR::IDENTITY)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);

        if let Some(old_swapchain) = old_swapchain {
            info.old_swapchain = old_swapchain.swapchain;
        }

        let swapchain = unsafe { loaders.swapchain.create_swapchain(&info, None) }?;
        let images = unsafe { loaders.swapchain.get_swapchain_images(swapchain) }?;
        let views = images
            .iter()
            .map(|&image| unsafe {
                device
                    .create_image_view(
                        &vk::ImageViewCreateInfo::default()
                            .image(image)
                            .view_type(vk::ImageViewType::TYPE_2D)
                            .format(vk::Format::B8G8R8A8_UNORM)
                            .components(vk::ComponentMapping {
                                r: vk::ComponentSwizzle::IDENTITY,
                                g: vk::ComponentSwizzle::IDENTITY,
                                b: vk::ComponentSwizzle::IDENTITY,
                                a: vk::ComponentSwizzle::IDENTITY,
                            })
                            .subresource_range(util::image_subresource_range(
                                vk::ImageAspectFlags::COLOR,
                            )),
                        None,
                    )
                    .context("Creating swapchain image views")
            })
            .collect::<Result<_, _>>()?;

        let extent = find_swapchain_extent(caps, extent.width, extent.height);
        Ok(Self {
            swapchain,
            images,
            views,
            extent,
            loaders,
            format,
        })
    }

    pub fn destroy(&self, device: &ash::Device, loaders: &Loaders) {
        unsafe { loaders.swapchain.destroy_swapchain(self.swapchain, None) };

        for view in &self.views {
            unsafe { device.destroy_image_view(*view, None) };
        }
    }

    pub fn acquire_next_image(&self, semaphore: vk::Semaphore) -> Result<(u32, bool), vk::Result> {
        unsafe {
            self.loaders.swapchain.acquire_next_image(
                self.swapchain,
                u64::MAX,
                semaphore,
                vk::Fence::null(),
            )
        }
    }

    pub fn get_image(&self, index: u32) -> vk::Image {
        self.images[index as usize]
    }

    pub fn present(
        &self,
        graphics_queue: vk::Queue,
        present_info: &vk::PresentInfoKHR,
    ) -> Result<bool, vk::Result> {
        unsafe {
            self.loaders
                .swapchain
                .queue_present(graphics_queue, present_info)
        }
    }
}

fn find_swapchain_extent(
    caps: vk::SurfaceCapabilitiesKHR,
    desired_width: u32,
    desired_height: u32,
) -> vk::Extent2D {
    if caps.current_extent.width != u32::MAX {
        return caps.current_extent;
    }

    vk::Extent2D {
        width: u32::max(
            caps.min_image_extent.width,
            u32::min(caps.max_image_extent.width, desired_width),
        ),
        height: u32::max(
            caps.min_image_extent.height,
            u32::min(caps.max_image_extent.height, desired_height),
        ),
    }
}
