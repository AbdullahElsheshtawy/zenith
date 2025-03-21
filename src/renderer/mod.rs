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
use ash::vk;
use bytemuck::{Pod, Zeroable};
use context::RenderContext;
use deletion_queue::DeletionQueue;
use descriptors::{DescriptorAllocator, DescriptorLayoutBuilder, PoolSizeRatio};
use framedata::FrameData;
use gpu_allocator::vulkan::{Allocator, AllocatorCreateDesc};
use image::Image;
use loaders::Loaders;
use std::{mem::ManuallyDrop, sync::Arc};
use swapchain::Swapchain;
use transitionable::Transitionable;
use ui::Ui;
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub struct Renderer<'a> {
    window: Arc<Window>,
    entry: ash::Entry,
    loaders: Loaders,

    ui: Ui,
    rcx: RenderContext,
    surface: vk::SurfaceKHR,

    swapchain: Swapchain,
    graphics_queue_family: u32,

    draw_image: Image,
    draw_image_descriptors: vk::DescriptorSet,
    draw_image_descriptor_layout: vk::DescriptorSetLayout,

    frames: [FrameData; Renderer::FRAMES_IN_FLIGHT],
    frame_number: usize,

    global_descriptor_allocator: DescriptorAllocator,

    background_effects: Vec<ComputeEffect<'a>>,
    current_background_effect: usize,
    deletion_queue: DeletionQueue<'a>,
}

impl Renderer<'_> {
    pub const FRAMES_IN_FLIGHT: usize = 2;

    pub fn new(window: Arc<Window>) -> anyhow::Result<Self> {
        let inner_size = window.inner_size();
        let window_size = inner_size;
        let entry = unsafe { ash::Entry::load() }?;
        let instance = create_instance(&window, &entry)?;
        let physical_device = util::pick_physical_device(&instance)?;
        let graphics_queue_family =
            util::select_queue_family(&instance, physical_device, vk::QueueFlags::GRAPHICS)?;
        let device = util::create_device(&instance, physical_device, graphics_queue_family)?;
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

        let compute_layout = unsafe {
            rcx.device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&[draw_image_descriptor_layout])
                    .push_constant_ranges(&[vk::PushConstantRange::default()
                        .size(std::mem::size_of::<ComputePushConstants>() as u32)
                        .stage_flags(vk::ShaderStageFlags::COMPUTE)]),
                None,
            )
        }?;

        let gradient_shader_module =
            util::load_shader_module("./shaders/gradient_color.comp.spv", &rcx.device)?;

        let sky_shader_module = util::load_shader_module("./shaders/sky.comp.spv", &rcx.device)?;
        let slide = util::load_shader_module("./shaders/slide.comp.spv", &rcx.device)?;
        let compute_info = vk::ComputePipelineCreateInfo::default()
            .layout(compute_layout)
            .stage(
                vk::PipelineShaderStageCreateInfo::default()
                    .stage(vk::ShaderStageFlags::COMPUTE)
                    .name(c"main"),
            );
        let sky_pipeline = {
            let mut info = compute_info;
            info.stage.module = sky_shader_module;
            unsafe {
                rcx.device
                    .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
            }
            .unwrap()[0]
        };
        let slide_pipeline = {
            let mut info = compute_info;
            info.stage.module = slide;
            unsafe {
                rcx.device
                    .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
            }
            .unwrap()[0]
        };

        let gradient_pipeline = {
            let mut info = compute_info;
            info.stage.module = gradient_shader_module;
            unsafe {
                rcx.device
                    .create_compute_pipelines(vk::PipelineCache::null(), &[info], None)
            }
            .unwrap()[0]
        };

        let background_effects = vec![
            ComputeEffect {
                name: "Gradient",
                pipeline: gradient_pipeline,
                layout: compute_layout,
                data: ComputePushConstants {
                    data1: glam::vec4(1.0, 0.0, 0.0, 1.0),
                    data2: glam::vec4(0.0, 0.0, 1.0, 1.0),
                    ..Default::default()
                },
            },
            ComputeEffect {
                name: "Sky",
                pipeline: sky_pipeline,
                layout: compute_layout,
                data: ComputePushConstants {
                    data1: glam::vec4(0.1, 0.2, 0.4, 0.97),
                    ..Default::default()
                },
            },
            ComputeEffect {
                name: "Slide",
                pipeline: slide_pipeline,
                layout: compute_layout,
                data: ComputePushConstants {
                    data1: glam::vec4(0.43, 0.54, 0.657, 0.5),
                    data2: glam::vec4(0.38, 1.24, 0.815, 0.8),
                    ..Default::default()
                },
            },
        ];
        unsafe {
            rcx.device.destroy_shader_module(slide, None);
            rcx.device.destroy_shader_module(sky_shader_module, None);
            rcx.device
                .destroy_shader_module(gradient_shader_module, None);
        }
        deletion_queue.push(Box::new(move |rcx: &mut RenderContext| unsafe {
            rcx.device.destroy_pipeline_layout(compute_layout, None);
            rcx.device.destroy_pipeline(gradient_pipeline, None);
            rcx.device.destroy_pipeline(sky_pipeline, None);
            rcx.device.destroy_pipeline(slide_pipeline, None);
        }));

        let ui = Ui::new(&window, &rcx, swapchain.format)?;
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
            deletion_queue,
            rcx,
            background_effects,
            current_background_effect: 0,
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

        let swapchain_image = Image::from_handle(
            &self.rcx,
            &mut self.deletion_queue,
            self.swapchain.get_image(swapchain_image_index),
            self.swapchain.format,
            self.swapchain.extent,
        )
        .unwrap();

        self.draw_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::GENERAL,
        );

        self.draw_background(cmd_buf);

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
            &self.draw_image,
            &swapchain_image,
            vk::Extent2D {
                width: self.draw_image.extent().width,
                height: self.draw_image.extent().height,
            },
            swapchain_image.extent(),
        );

        swapchain_image.transition(
            &self.rcx,
            cmd_buf,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            vk::ImageLayout::PRESENT_SRC_KHR,
        );
        self.draw_ui(cmd_buf, &swapchain_image);

        let frame = self.get_current_frame();
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
            let effect = self.background_effects[self.current_background_effect];
            self.rcx.device.cmd_bind_pipeline(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                effect.pipeline,
            );

            self.rcx.device.cmd_bind_descriptor_sets(
                cmd_buf,
                vk::PipelineBindPoint::COMPUTE,
                effect.layout,
                0,
                &[self.draw_image_descriptors],
                &[],
            );

            self.rcx.device.cmd_push_constants(
                cmd_buf,
                effect.layout,
                vk::ShaderStageFlags::COMPUTE,
                0,
                bytemuck::cast_slice(&[effect.data]),
            );
            self.rcx.device.cmd_dispatch(
                cmd_buf,
                f32::ceil(self.draw_image.extent().width as f32 / 16.0).round() as u32,
                f32::ceil(self.draw_image.extent().height as f32 / 16.0).round() as u32,
                1,
            );
        }
    }

    fn draw_ui(&mut self, cmd_buf: vk::CommandBuffer, image: &Image) {
        let raw_input = self.ui.egui_winit.take_egui_input(&self.window);
        let egui::FullOutput {
            platform_output,
            textures_delta,
            shapes,
            pixels_per_point,
            ..
        } = self.ui.ctx.run(raw_input, |ctx| {
            // Ui is here
            egui::Window::new("Background").show(ctx, |ui| {
                ui.label(format!(
                    "Selected effect: {}",
                    self.background_effects[self.current_background_effect].name
                ));

                ui.add(
                    egui::Slider::new(
                        &mut self.current_background_effect,
                        0..=self.background_effects.len() - 1,
                    )
                    .text("Effect index"),
                );
                let selected = &mut self.background_effects[self.current_background_effect];

                // Using a grid layout for the vector fields
                egui::Grid::new("data_grid").num_columns(4).show(ui, |ui| {
                    ui.label("data1");
                    ui.add(egui::DragValue::new(&mut selected.data.data1[0]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data1[1]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data1[2]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data1[3]).speed(0.01));
                    ui.end_row();

                    ui.label("data2");
                    ui.add(egui::DragValue::new(&mut selected.data.data2[0]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data2[1]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data2[2]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data2[3]).speed(0.01));
                    ui.end_row();

                    ui.label("data3");
                    ui.add(egui::DragValue::new(&mut selected.data.data3[0]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data3[1]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data3[2]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data3[3]).speed(0.01));
                    ui.end_row();

                    ui.label("data4");
                    ui.add(egui::DragValue::new(&mut selected.data.data4[0]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data4[1]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data4[2]).speed(0.01));
                    ui.add(egui::DragValue::new(&mut selected.data.data4[3]).speed(0.01));
                    ui.end_row();
                });
            });
        });

        self.ui
            .egui_winit
            .handle_platform_output(&self.window, platform_output);

        if !textures_delta.free.is_empty() {
            self.ui
                .renderer
                .free_textures(&textures_delta.free)
                .map_err(|err| println!("{err}"))
                .unwrap();
        }
        if !textures_delta.set.is_empty() {
            let cmd_pool = self.get_current_frame().cmd_pool;
            self.ui
                .renderer
                .set_textures(self.rcx.gfx_queue, cmd_pool, &textures_delta.set)
                .map_err(|err| println!("{err}"))
                .unwrap();
        }

        let primitives = self.ui.ctx.tessellate(shapes, pixels_per_point);
        unsafe {
            self.rcx.device.cmd_begin_rendering(
                cmd_buf,
                &util::rendering_info(
                    image.extent(),
                    &[util::rendering_attachment_info(
                        image,
                        None,
                        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                    )],
                    None,
                ),
            );

            self.rcx.device.cmd_set_viewport(
                cmd_buf,
                0,
                &[vk::Viewport {
                    width: image.extent().width as f32,
                    height: image.extent().height as f32,
                    max_depth: 1.0,
                    ..Default::default()
                }],
            );
            self.rcx.device.cmd_set_scissor(
                cmd_buf,
                0,
                &[vk::Rect2D {
                    offset: Default::default(),
                    extent: image.extent(),
                }],
            );
        }
        let _ = self
            .ui
            .renderer
            .cmd_draw(cmd_buf, image.extent(), pixels_per_point, &primitives)
            .map_err(|err| println!("{}", err));
        unsafe {
            self.rcx.device.cmd_end_rendering(cmd_buf);
        }
    }

    pub fn window_event(&mut self, window_event: &winit::event::WindowEvent) {
        let _ = self
            .ui
            .egui_winit
            .on_window_event(&self.window, window_event);
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

impl Drop for Renderer<'_> {
    fn drop(&mut self) {
        unsafe {
            self.rcx.device.device_wait_idle().unwrap();
            ManuallyDrop::drop(&mut self.ui.renderer);
            self.frames.iter_mut().for_each(|frame| {
                frame.destroy(&self.rcx.device);
            });

            self.deletion_queue.flush(&mut self.rcx);
            self.swapchain.destroy(&self.rcx.device, &self.loaders);
            self.loaders.surface.destroy_surface(self.surface, None);

            self.rcx.destroy();
        }
    }
}

#[derive(Default, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct ComputePushConstants {
    data1: glam::Vec4,
    data2: glam::Vec4,
    data3: glam::Vec4,
    data4: glam::Vec4,
}

#[derive(Clone, Copy)]
struct ComputeEffect<'a> {
    name: &'a str,
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    data: ComputePushConstants,
}
