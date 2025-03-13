use ash::vk;

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
