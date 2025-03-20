mod context;
mod deletion_queue;
mod descriptors;
mod framedata;
mod image;
mod loaders;
mod swapchain;
mod transitionable;
mod ui;
mod util;
use anyhow::Context;
use ash::vk;
use context::RenderContext;
use deletion_queue::DeletionQueue;
use descriptors::{DescriptorAllocator, DescriptorLayoutBuilder, PoolSizeRatio};
use framedata::FrameData;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use image::Image;
use loaders::Loaders;
use std::{collections::VecDeque, f32::consts::TAU, sync::Arc};
use swapchain::Swapchain;
use transitionable::Transitionable;
use ui::Ui;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};
use yakui::{column, widgets::ColoredBox};

pub struct Renderer<'a> {
    window: Arc<Window>,
    entry: ash::Entry,
    loaders: Loaders,

    rcx: RenderContext,
    surface: vk::SurfaceKHR,

    swapchain: Swapchain,
    graphics_queue_family: u32,

    draw_image: Image,
    draw_image_descriptors: vk::DescriptorSet,
    draw_image_descriptor_layout: vk::DescriptorSetLayout,

    gradient_pipeline: vk::Pipeline,
    gradient_pipeline_layout: vk::PipelineLayout,

    frames: [FrameData; Renderer::FRAMES_IN_FLIGHT],
    frame_number: usize,

    global_descriptor_allocator: DescriptorAllocator,

    ui: Ui,
    deletion_queue: DeletionQueue<'a>,
}

impl Renderer<'_> {
    pub const FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let window_size = window.inner_size();
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&window, &entry)?;
        let physical_device = pick_physical_device(&instance)?;
        let graphics_queue_family =
            select_queue_family(&instance, physical_device, vk::QueueFlags::GRAPHICS)?;
        let device = create_device(&instance, physical_device, graphics_queue_family)?;
        let allocator = Allocator::new(&AllocatorCreateDesc {
            instance: instance.clone(),
            device: device.clone(),
            physical_device,
            debug_settings: Default::default(),
            buffer_device_address: true,
            allocation_sizes: Default::default(),
        })?;
        let gfx_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
        let mut rcx = RenderContext::new(instance, physical_device, device, allocator, gfx_queue);
        let loaders = Loaders::new(&entry, &rcx.instance, &rcx.device);
        let surface = unsafe {
            ash_window::create_surface(
                &entry,
                &rcx.instance,
                window.display_handle()?.as_raw(),
                window.window_handle()?.as_raw(),
                None,
            )
        }?;

        let swapchain = Swapchain::new(
            &rcx.device,
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

        let frames: [FrameData; Renderer::FRAMES_IN_FLIGHT] = std::array::from_fn(|_| {
            FrameData::new(&rcx.device, graphics_queue_family)
                .expect("Failed creating per frame in flight data")
        });

        let mut deletion_queue = DeletionQueue::new();
        let draw_image_usage_flags = vk::ImageUsageFlags::TRANSFER_SRC
            | vk::ImageUsageFlags::TRANSFER_DST
            | vk::ImageUsageFlags::STORAGE
            | vk::ImageUsageFlags::COLOR_ATTACHMENT;
        let draw_image = Image::new(
            &mut rcx,
            &mut deletion_queue,
            vk::Extent2D {
                width: window_size.width,
                height: window_size.height,
            },
            vk::Format::R16G16B16A16_SFLOAT,
            draw_image_usage_flags,
        )?;

        let global_descriptor_allocator = DescriptorAllocator::new(
            &rcx.device,
            10,
            &[PoolSizeRatio {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                ratio: 1.0,
            }; 1],
        )?;
        let draw_image_descriptor_layout = DescriptorLayoutBuilder::new()
            .add_binding(0, vk::DescriptorType::STORAGE_IMAGE)
            .build(
                &rcx.device,
                vk::ShaderStageFlags::COMPUTE,
                None,
                vk::DescriptorSetLayoutCreateFlags::empty(),
            )?;

        let draw_image_descriptors =
            global_descriptor_allocator.allocate(&rcx.device, draw_image_descriptor_layout)?;
        let img_info = [vk::DescriptorImageInfo::default()
            .image_layout(vk::ImageLayout::GENERAL)
            .image_view(draw_image.view())];

        let draw_image_write = [vk::WriteDescriptorSet::default()
            .dst_binding(0)
            .dst_set(draw_image_descriptors)
            .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
            .image_info(&img_info)];

        unsafe {
            rcx.device.update_descriptor_sets(&draw_image_write, &[]);
        }

        deletion_queue.push(Box::new(move |rcx: &mut RenderContext| {
            global_descriptor_allocator.destroy(&rcx.device);
            unsafe {
                rcx.device
                    .destroy_descriptor_set_layout(draw_image_descriptor_layout, None)
            };
        }));

        let gradient_pipeline_layout = unsafe {
            rcx.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[draw_image_descriptor_layout]),
                None,
            )
        }?;

        let shader_module = util::load_shader_module("./shaders/gradient.comp.spv", &rcx.device)?;

        let stage_info = vk::PipelineShaderStageCreateInfo::default()
            .stage(vk::ShaderStageFlags::COMPUTE)
            .module(shader_module)
            .name(c"main");

        let pipeline_info = [vk::ComputePipelineCreateInfo::default()
            .layout(gradient_pipeline_layout)
            .stage(stage_info)];

        let gradient_pipeline = match unsafe {
            rcx.device
                .create_compute_pipelines(vk::PipelineCache::null(), &pipeline_info, None)
        } {
            Ok(ps) => ps[0],
            Err((ps, err)) => {
                ps.iter().for_each(|pl| unsafe {
                    rcx.device.destroy_pipeline(*pl, None);
                });

                anyhow::bail!("Error Creating pipelines: {err}");
            }
        };

        deletion_queue.push(Box::new(move |rcx: &mut RenderContext| unsafe {
            rcx.device.destroy_shader_module(shader_module, None);
            rcx.device
                .destroy_pipeline_layout(gradient_pipeline_layout, None);
            rcx.device.destroy_pipeline(gradient_pipeline, None);
        }));

        let ui = Ui::new(&window, &rcx, draw_image.format());
        Ok(Self {
            window,
            entry,
            loaders,
            surface,
            swapchain,
            graphics_queue_family,
            frames,
            frame_number: 0,
            draw_image,
            draw_image_descriptors,
            draw_image_descriptor_layout,
            global_descriptor_allocator,
            gradient_pipeline,
            gradient_pipeline_layout,
            deletion_queue,
            rcx,
            ui,
        })
    }

    pub fn get_current_frame(&self) -> &FrameData {
        &self.frames[self.frame_number % Renderer::FRAMES_IN_FLIGHT]
    }
    pub fn draw(&mut self) {
        let frame = self.get_current_frame();
        let cmd_buf = frame.cmd_buf;

        unsafe {
            self.rcx
                .device
                .wait_for_fences(&[frame.render_fence], true, u64::MAX)
                .unwrap();
            self.rcx.device.reset_fences(&[frame.render_fence]).unwrap();
        }
        let (swapchain_image_index, _) = self
            .swapchain
            .acquire_next_image(frame.swapchain_sem)
            .unwrap();

        unsafe {
            self.rcx
                .device
                .reset_command_buffer(cmd_buf, vk::CommandBufferResetFlags::empty())
        }
        .unwrap();

        unsafe {
            self.rcx
                .device
                .begin_command_buffer(
                    cmd_buf,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .unwrap()
        };

        let swapchain_image = self.swapchain.get_image(swapchain_image_index);

        self.draw_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        self.draw_background(cmd_buf);
        self.draw_ui(cmd_buf);

        let viewports = [vk::Viewport {
            width: self.swapchain.extent.width as f32,
            height: self.swapchain.extent.height as f32,
            max_depth: 1.0,
            ..Default::default()
        }];
        unsafe {
            self.rcx.device.cmd_begin_rendering(
                cmd_buf,
                &vk::RenderingInfo::default()
                    .render_area(vk::Rect2D {
                        offset: Default::default(),
                        extent: self.draw_image.extent(),
                    })
                    .layer_count(1)
                    .color_attachments(&[vk::RenderingAttachmentInfo::default()
                        .image_view(self.draw_image.view())
                        .image_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
                        .load_op(vk::AttachmentLoadOp::LOAD)
                        .store_op(vk::AttachmentStoreOp::STORE)]),
            );
            self.rcx.device.cmd_set_viewport(cmd_buf, 0, &viewports);
            self.rcx
                .device
                .cmd_set_scissor(cmd_buf, 0, &[self.draw_image.extent().into()]);
        }
        self.ui.render(&self.rcx, cmd_buf, self.draw_image.extent());
        unsafe {
            self.rcx.device.cmd_end_rendering(cmd_buf);
        }
        let frame = self.get_current_frame();
        self.draw_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::GENERAL,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        );

        swapchain_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        );

        util::copy_image_to_image(
            &self.rcx,
            cmd_buf,
            self.draw_image.image(),
            swapchain_image,
            vk::Extent2D {
                width: self.draw_image.extent().width,
                height: self.draw_image.extent().height,
            },
            self.swapchain.extent,
        );

        swapchain_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );

        unsafe { self.rcx.device.end_command_buffer(cmd_buf) }.unwrap();

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
            self.rcx
                .device
                .queue_submit2(self.rcx.gfx_queue, &submit_info, frame.render_fence)
        }
        .unwrap();

        // Prepare presentation
        // make sure we finished all the drawing commands by waiting on the render semaphore
        self.swapchain
            .present(
                self.rcx.gfx_queue,
                &vk::PresentInfoKHR::default()
                    .swapchains(&[self.swapchain.swapchain])
                    .wait_semaphores(&[frame.render_sem])
                    .image_indices(&[swapchain_image_index]),
            )
            .unwrap();
        self.frame_number += 1;
    }

    fn draw_background(&self, cmd_buf: vk::CommandBuffer) {
        unsafe {
            self.rcx.device.cmd_bind_pipeline(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                self.gradient_pipeline,
            );

            self.rcx.device.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                self.gradient_pipeline_layout,
                0,
                &[self.draw_image_descriptors],
                &[],
            );

            self.rcx.device.cmd_dispatch(
                cmd_buf,
                f32::ceil(self.draw_image.extent().width as f32 / 16.0).round() as u32,
                f32::ceil(self.draw_image.extent().height as f32 / 16.0).round() as u32,
                1,
            );
        }
    }

    fn draw_ui(&mut self, cmd_buf: vk::CommandBuffer) {
        self.ui.start();
        ui::fps_counter();
        self.ui.finish(cmd_buf, &self.rcx);
    }

    pub fn window_event(&mut self, window_event: &winit::event::WindowEvent) {
        self.ui.window_event(window_event);
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

impl Drop for Renderer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.rcx.device.device_wait_idle().unwrap();
            self.frames.iter_mut().for_each(|frame| {
                frame.destroy(&self.rcx.device);
            });

            self.deletion_queue.flush(&mut self.rcx);
            self.ui.destroy(&self.rcx);
            self.swapchain.destroy(&self.rcx.device, &self.loaders);
            self.loaders.surface.destroy_surface(self.surface, None);

            self.rcx.destroy();
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
        .descriptor_indexing(true)
        // these features are required for yakui-vulkan
        .descriptor_binding_partially_bound(true)
        .descriptor_binding_sampled_image_update_after_bind(true);
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
