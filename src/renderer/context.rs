use ash::vk;
use gpu_allocator::vulkan::Allocator;
pub struct RenderContext {
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub allocator: Allocator,
    pub gfx_queue: vk::Queue,
}

impl RenderContext {
    pub fn new(
        instance: ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        allocator: Allocator,
        gfx_queue: vk::Queue,
    ) -> Self {
        Self {
            device,
            allocator,
            instance,
            physical_device,
            gfx_queue,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        };
    }
}
