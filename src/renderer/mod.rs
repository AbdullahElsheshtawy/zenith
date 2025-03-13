mod framedata;
mod loaders;
mod swapchain;
mod transitionable;
mod util;
use anyhow::Context;
use ash::vk;
use framedata::FrameData;
use loaders::Loaders;
use std::sync::Arc;
use swapchain::Swapchain;
use transitionable::Transitionable;
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

    frames: [FrameData; Self::FRAMES_IN_FLIGHT],
    frame_number: usize,
}

impl Renderer {
    pub const FRAMES_IN_FLIGHT: usize = 2;

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
            surface,
            loaders.clone(),
            vk::PresentModeKHR::FIFO,
            vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            },
            None,
        )?;

        let frames: [FrameData; Self::FRAMES_IN_FLIGHT] = std::array::from_fn(|_| {
            FrameData::new(&device, graphics_queue_family)
                .expect("Failed creating per frame in flight data")
        });

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
            frames,
            frame_number: 0,
        })
    }
    pub fn get_current_frame(&self) -> &FrameData {
        &self.frames[self.frame_number % Self::FRAMES_IN_FLIGHT]
    }
    pub fn draw(&mut self) {
        let get_current_frame = self.get_current_frame();
        let frame = get_current_frame;
        let cmd_buf = frame.cmd_buf;

        unsafe {
            self.device
                .wait_for_fences(&[frame.render_fence], true, u64::MAX)
                .unwrap();
            self.device.reset_fences(&[frame.render_fence]).unwrap();
        }
        let (swapchain_image_index, _) = self
            .swapchain
            .acquire_next_image(frame.swapchain_sem)
            .unwrap();

        unsafe {
            self.device
                .reset_command_buffer(cmd_buf, vk::CommandBufferResetFlags::empty())
        }
        .unwrap();

        unsafe {
            self.device
                .begin_command_buffer(
                    cmd_buf,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap()
        };

        let swapchain_image = self.swapchain.get_image(swapchain_image_index);

        swapchain_image.transition(
            &self.device,
            cmd_buf,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        let clear_value = vk::ClearColorValue {
            float32: [
                0.0,
                0.0,
                f32::abs(f32::sin(self.frame_number as f32 / 120.0)),
                0.0,
            ],
        };

        unsafe {
            self.device.cmd_clear_color_image(
                cmd_buf,
                swapchain_image,
                vk::ImageLayout::GENERAL,
                &clear_value,
                &[util::image_subresource_range(vk::ImageAspectFlags::COLOR)],
            )
        };

        swapchain_image.transition(
            &self.device,
            cmd_buf,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        unsafe { self.device.end_command_buffer(cmd_buf) }.unwrap();

        // prepare the submission
        // wait on the present semaphore because it is signaled when the swapchain is ready
        // signal the render semaphore to signal that rendering is done
        let cmd_buf_submit_info = [util::command_buffer_submit_info(cmd_buf)];
        let wait_info = [util::semaphore_submit_info(
            vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT,
            frame.swapchain_sem,
        )];
        let signal_info = [util::semaphore_submit_info(
            vk::PipelineStageFlags2::ALL_GRAPHICS,
            frame.render_sem,
        )];

        let submit_info = [util::submit_info(
            &cmd_buf_submit_info,
            &signal_info,
            &wait_info,
        )];

        unsafe {
            self.device
                .queue_submit2(self.graphics_queue, &submit_info, frame.render_fence)
        }
        .unwrap();

        // Prepare presentation
        // make sure we finished all the drawing commands by waiting on the render semaphore
        self.swapchain
            .present(
                self.graphics_queue,
                &vk::PresentInfoKHR::default()
                    .swapchains(&[self.swapchain.swapchain])
                    .wait_semaphores(&[frame.render_sem])
                    .image_indices(&[swapchain_image_index]),
            )
            .unwrap();
        self.frame_number += 1;
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
            self.frames.iter().for_each(|frame| {
                frame.destroy(&self.device);
            });
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
