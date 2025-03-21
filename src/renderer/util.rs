use std::io::{Read, Seek};

use ash::vk;

use super::{context::RenderContext, image::Image};

pub fn load_shader_module(
    file_path: &str,
    device: &ash::Device,
) -> anyhow::Result<vk::ShaderModule> {
    let mut file = std::fs::File::open(file_path)?;

    let size = file.seek(std::io::SeekFrom::End(0))? as usize;
    file.seek(std::io::SeekFrom::Start(0))?;
    let mut code = vec![0u32; size / std::mem::size_of::<u32>()];

    file.read_exact(unsafe { std::slice::from_raw_parts_mut(code.as_mut_ptr().cast(), size) })?;

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
