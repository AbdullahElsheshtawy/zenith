use ash::vk;
use gpu_allocator::vulkan::Allocator;
pub struct RenderContext {
    pub instance: ash::Instance,
    pub physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub allocator: Allocator,
}

impl RenderContext {
    pub fn new(
        instance: ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: ash::Device,
        allocator: Allocator,
    ) -> Self {
        Self {
            device,
            allocator,
            instance,
            physical_device,
        }
    }

    pub fn destroy(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
            self.device.destroy_device(None)
        };
    }
}
