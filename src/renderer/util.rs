use anyhow::Context;
use ash::vk;

use super::{context::RenderContext, image::Image};

pub fn rendering_attachment_info(
    image: &Image,
    clear: Option<vk::ClearValue>,
    layout: vk::ImageLayout,
) -> vk::RenderingAttachmentInfo<'static> {
    let mut info = vk::RenderingAttachmentInfo::default()
        .image_view(image.view())
        .image_layout(layout)
        .load_op(if clear.is_some() {
            vk::AttachmentLoadOp::CLEAR
        } else {
            vk::AttachmentLoadOp::LOAD
        })
        .store_op(vk::AttachmentStoreOp::STORE);
    if let Some(clear) = clear {
        info = info.clear_value(clear);
    }
    info
}

pub fn rendering_info<'a>(
    render_extent: vk::Extent2D,
    color_attachment_info: &'a [vk::RenderingAttachmentInfo<'a>],
    depth_attachment_info: Option<&'a vk::RenderingAttachmentInfo>,
) -> vk::RenderingInfo<'a> {
    let mut info = vk::RenderingInfo::default()
        .render_area(vk::Rect2D {
            offset: Default::default(),
            extent: render_extent,
        })
        .layer_count(1)
        .color_attachments(color_attachment_info);
    if let Some(depth_info) = depth_attachment_info {
        info = info.depth_attachment(depth_info);
    }
    info
}

pub fn load_shader_module(
    file_path: &str,
    device: &ash::Device,
) -> anyhow::Result<vk::ShaderModule> {
    let mut file = std::fs::File::open(file_path)?;

    let code = ash::util::read_spv(&mut file)?;

    Ok(unsafe {
        device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(&code), None)
    }?)
}
pub fn copy_image_to_image(
    rcx: &RenderContext,
    cmd_buf: vk::CommandBuffer,
    src: &Image,
    dst: &Image,
    src_size: vk::Extent2D,
    dst_size: vk::Extent2D,
) {
    let blit_region = [vk::ImageBlit2::default()
        .src_offsets([
            vk::Offset3D::default(),
            vk::Offset3D {
                x: src_size.width as _,
                y: src_size.height as _,
                z: 1,
            },
        ])
        .dst_offsets([
            vk::Offset3D::default(),
            vk::Offset3D {
                x: dst_size.width as _,
                y: dst_size.height as _,
                z: 1,
            },
        ])
        .src_subresource(
            vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1),
        )
        .dst_subresource(
            vk::ImageSubresourceLayers::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .layer_count(1),
        )];

    let blit_info = vk::BlitImageInfo2::default()
        .src_image(src.image())
        .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
        .dst_image(dst.image())
        .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .filter(vk::Filter::LINEAR)
        .regions(&blit_region);

    unsafe { rcx.device.cmd_blit_image2(cmd_buf, &blit_info) };
}
pub fn image_create_info(
    format: vk::Format,
    usage_flags: vk::ImageUsageFlags,
    extent: vk::Extent2D,
) -> vk::ImageCreateInfo<'static> {
    vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .usage(usage_flags)
        .format(format)
        .extent(vk::Extent3D {
            width: extent.width,
            height: extent.height,
            depth: 1,
        })
        .array_layers(1)
        .mip_levels(1)
        .samples(vk::SampleCountFlags::TYPE_1)
}

pub fn image_view_create_info(
    format: vk::Format,
    image: vk::Image,
    aspect_flags: vk::ImageAspectFlags,
) -> vk::ImageViewCreateInfo<'static> {
    vk::ImageViewCreateInfo::default()
        .view_type(vk::ImageViewType::TYPE_2D)
        .image(image)
        .format(format)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: aspect_flags,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        })
}
pub fn semaphore_submit_info(
    stage_mask: vk::PipelineStageFlags2,
    semaphore: vk::Semaphore,
) -> vk::SemaphoreSubmitInfo<'static> {
    vk::SemaphoreSubmitInfo::default()
        .semaphore(semaphore)
        .stage_mask(stage_mask)
        .value(1)
}

pub fn command_buffer_submit_info(
    cmd_buf: vk::CommandBuffer,
) -> vk::CommandBufferSubmitInfo<'static> {
    vk::CommandBufferSubmitInfo::default().command_buffer(cmd_buf)
}

pub fn submit_info<'a>(
    cmd_buf_submit_info: &'a [vk::CommandBufferSubmitInfo<'a>],
    signal_semaphore_submit_info: &'a [vk::SemaphoreSubmitInfo<'a>],
    wait_semaphore_submit_info: &'a [vk::SemaphoreSubmitInfo<'a>],
) -> vk::SubmitInfo2<'a> {
    vk::SubmitInfo2::default()
        .wait_semaphore_infos(wait_semaphore_submit_info)
        .signal_semaphore_infos(signal_semaphore_submit_info)
        .command_buffer_infos(cmd_buf_submit_info)
}

pub fn image_subresource_range(aspect_mask: vk::ImageAspectFlags) -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .level_count(vk::REMAINING_MIP_LEVELS)
        .layer_count(vk::REMAINING_ARRAY_LAYERS)
}

pub fn pick_physical_device(instance: &ash::Instance) -> anyhow::Result<vk::PhysicalDevice> {
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

pub fn create_device(
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

pub fn select_queue_family(
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
