use ash::vk;
pub struct FrameData {
    pub swapchain_sem: vk::Semaphore,
    pub render_sem: vk::Semaphore,
    pub render_fence: vk::Fence,
    pub cmd_pool: vk::CommandPool,
    pub cmd_buf: vk::CommandBuffer,
}

impl FrameData {
    pub fn new(device: &ash::Device, queue_family_idx: u32) -> anyhow::Result<Self> {
        let sem_info = vk::SemaphoreCreateInfo::default();
        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);

        let swapchain_sem = unsafe { device.create_semaphore(&sem_info, None) }?;
        let render_sem = unsafe { device.create_semaphore(&sem_info, None) }?;

        let render_fence = unsafe { device.create_fence(&fence_info, None) }?;

        let cmd_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(queue_family_idx),
                None,
            )
        }?;

        let cmd_buf = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(cmd_pool)
                    .command_buffer_count(1)
                    .level(vk::CommandBufferLevel::PRIMARY),
            )
        }?[0];

        Ok(Self {
            cmd_pool,
            cmd_buf,
            swapchain_sem,
            render_sem,
            render_fence,
        })
    }

    pub fn destroy(&mut self, device: &ash::Device) {
        unsafe {
            device.destroy_command_pool(self.cmd_pool, None);
            device.destroy_fence(self.render_fence, None);
            device.destroy_semaphore(self.render_sem, None);
            device.destroy_semaphore(self.swapchain_sem, None);
        }
    }
}
