use ash::vk;

use super::{context::RenderContext, util};
pub trait Transitionable {
    fn transition(
        &self,
        rcx: &RenderContext,
        cmd_buf: vk::CommandBuffer,
        current_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    );
}

impl Transitionable for vk::Image {
    fn transition(
        &self,
        rcx: &RenderContext,
        cmd_buf: vk::CommandBuffer,
        current_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
    ) {
        let aspect_mask = if new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };

        let image_barrier = [vk::ImageMemoryBarrier2::default()
            .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .src_access_mask(vk::AccessFlags2::MEMORY_WRITE)
            .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
            .dst_access_mask(vk::AccessFlags2::MEMORY_WRITE | vk::AccessFlags2::MEMORY_READ)
            .old_layout(current_layout)
            .new_layout(new_layout)
            .subresource_range(util::image_subresource_range(aspect_mask))
            .image(*self)];

        let dep_info = vk::DependencyInfo::default().image_memory_barriers(&image_barrier);
        unsafe { rcx.device.cmd_pipeline_barrier2(cmd_buf, &dep_info) };
    }
}
