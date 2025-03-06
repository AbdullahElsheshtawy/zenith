use anyhow::Context;
use ash::vk;
use std::sync::Arc;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct Renderer {
    window: Arc<Window>,
    entry: ash::Entry,
    instance: ash::Instance,
    device: ash::Device,
    loaders: Loaders,
    physical_device: vk::PhysicalDevice,

    surface: vk::SurfaceKHR,

    swapchain: Swapchain,
    graphics_queue: vk::Queue,
    graphics_queue_family: u32,
}

pub struct Loaders {
    swapchain: ash::khr::swapchain::Device,
    surface: ash::khr::surface::Instance,
}

impl Loaders {
    pub fn new(entry: &ash::Entry, instance: &ash::Instance, device: &ash::Device) -> Self {
        Self {
            swapchain: ash::khr::swapchain::Device::new(instance, device),
            surface: ash::khr::surface::Instance::new(entry, instance),
        }
    }
}

pub struct Swapchain {
    swapchain: vk::SwapchainKHR,
    images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
    extent: vk::Extent2D,
}

impl Swapchain {
    pub fn new(
        device: &ash::Device,
        physical_device: vk::PhysicalDevice,
        loaders: &Loaders,
        surface: vk::SurfaceKHR,
        present_mode: vk::PresentModeKHR,
        extent: vk::Extent2D,
        old_swapchain: Option<Swapchain>,
    ) -> anyhow::Result<Self> {
        let mut info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(2)
            .image_format(vk::Format::B8G8R8A8_UNORM)
            .image_color_space(vk::ColorSpaceKHR::SRGB_NONLINEAR)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
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
                            .subresource_range(
                                vk::ImageSubresourceRange::default()
                                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                                    .level_count(1)
                                    .layer_count(1),
                            ),
                        None,
                    )
                    .context("Creating swapchain image views")
            })
            .collect::<Result<_, _>>()?;

        let extent = find_swapchain_extent(
            unsafe {
                loaders
                    .surface
                    .get_physical_device_surface_capabilities(physical_device, surface)?
            },
            extent.width,
            extent.height,
        );
        Ok(Self {
            swapchain,
            images,
            views,
            extent,
        })
    }

    fn destroy(&self, device: &ash::Device, loaders: &Loaders) {
        unsafe { loaders.swapchain.destroy_swapchain(self.swapchain, None) };

        for view in &self.views {
            unsafe { device.destroy_image_view(*view, None) };
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
impl Renderer {
    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let window_size = window.inner_size();
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&window, &entry)?;
        let physical_device = pick_physical_device(&instance)?;
        let graphics_queue_family =
            select_queue_family(&instance, physical_device, vk::QueueFlags::GRAPHICS)?;
        let device = create_device(&instance, physical_device, graphics_queue_family)?;
        let loaders = Loaders::new(&entry, &instance, &device);
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )
        }?;

        let swapchain = Swapchain::new(
            &device,
            physical_device,
            &loaders,
            surface,
            vk::PresentModeKHR::FIFO,
            vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            },
            None,
        )?;

        Ok(Self {
            window,
            entry,
            instance,
            device,
            loaders,
            physical_device,
            surface,
            swapchain,
            graphics_queue,
            graphics_queue_family,
        })
    }
}

fn create_instance(window: &Window, entry: &ash::Entry) -> anyhow::Result<ash::Instance> {
    let app_info = vk::ApplicationInfo::default().api_version(vk::API_VERSION_1_3);
    let raw_display_handle = window.display_handle()?.as_raw();

    let enabled_extensions = ash_window::enumerate_required_extensions(raw_display_handle)?;
    Ok(unsafe {
        entry.create_instance(
            &vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .enabled_extension_names(enabled_extensions),
            None,
        )
    }?)
}

fn pick_physical_device(instance: &ash::Instance) -> anyhow::Result<vk::PhysicalDevice> {
    unsafe {
        let devices = instance.enumerate_physical_devices()?;

        devices
            .into_iter()
            .max_by_key(|device| {
                let properties = instance.get_physical_device_properties(*device);
                match properties.device_type {
                    vk::PhysicalDeviceType::DISCRETE_GPU => 100,
                    vk::PhysicalDeviceType::INTEGRATED_GPU => 75,
                    _ => 0,
                }
            })
            .context("No Suitable gpu!")
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();

            self.swapchain.destroy(&self.device, &self.loaders);
            self.loaders.surface.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_idx: u32,
) -> anyhow::Result<ash::Device> {
    let extensions = [vk::KHR_SWAPCHAIN_NAME.as_ptr()];

    let queue_create_infos = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_idx)
        .queue_priorities(&[1.0])];

    let mut features12 = vk::PhysicalDeviceVulkan12Features::default()
        .buffer_device_address(true)
        .descriptor_indexing(true);
    let mut features13 = vk::PhysicalDeviceVulkan13Features::default()
        .dynamic_rendering(true)
        .synchronization2(true);

    Ok(unsafe {
        instance.create_device(
            physical_device,
            &vk::DeviceCreateInfo::default()
                .queue_create_infos(&queue_create_infos)
                .enabled_extension_names(&extensions)
                .push_next(&mut features12)
                .push_next(&mut features13),
            None,
        )?
    })
}
fn select_queue_family(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    flags: vk::QueueFlags,
) -> anyhow::Result<u32> {
    unsafe {
        instance
            .get_physical_device_queue_family_properties(physical_device)
            .into_iter()
            .enumerate()
            .find(|(_, properties)| properties.queue_flags.contains(flags))
            .map(|(idx, _)| idx as u32)
            .context("The queue family requested does not exist")
    }
}
